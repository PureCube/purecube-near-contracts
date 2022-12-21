import anyTest, { TestFn } from "ava";
import { NEAR, NearAccount, Worker, BN } from "near-workspaces";
import path from "path";
import {
  approveNFT,
  defaultCallOptions,
  DEFAULT_GAS,
  mintNFT,
  payForStorage,
  placeNFTForSale,
  purchaseListedNFT,
  transferNFT,
  MINT_PRICE,
  DEFAULT_BASE_URI,
  MINT_START,
  MINT_END,
  DEFAULT_DEPOSIT_FOR_MINT
} from "./utils";

const test = anyTest as TestFn<{
  worker: Worker;
  accounts: Record<string, NearAccount>;
}>;

test.beforeEach(async (t) => {
  const worker = await Worker.init();
  const root = worker.rootAccount;

  const treasury = await root.createSubAccount("treasury", {
    initialBalance: NEAR.parse("1 N").toJSON(),
  });

  const nftContractLocation = path.join(__dirname, "../../../out/main.wasm");

  const royalties_string = `{"${treasury.accountId}":2000}`; // 20%
  const royalties = JSON.parse(royalties_string);

  const nft_contract = await root.devDeploy(
    nftContractLocation,
    {
      method: "new_default_meta",
      args: {
        owner_id: root,
        treasury_id: treasury,
        max_supply: "10",
        base_uri: DEFAULT_BASE_URI,
        mint_price: MINT_PRICE,
        mint_start: MINT_START,
        mint_end: MINT_END,
        perpetual_royalties: royalties,
      },
      initialBalance: NEAR.parse("100 N").toJSON()
    }
  );

  const marketContractLocation = path.join(__dirname, "../../../out/market.wasm");
  const market_contract = await root.devDeploy(
    marketContractLocation,
    {
      method: "new",
      args: { owner_id: root },
      initialBalance: NEAR.parse("100 N").toJSON()
    }
  );

  const alice = await root.createSubAccount("alice", {
    initialBalance: NEAR.parse("100 N").toJSON(),
  });

  const bob = await root.createSubAccount("bob", {
    initialBalance: NEAR.parse("100 N").toJSON(),
  });

  const charlie = await root.createSubAccount("charlie", {
    initialBalance: NEAR.parse("100 N").toJSON(),
  });

  t.context.worker = worker;
  t.context.accounts = { root, nft_contract, market_contract, alice, bob, charlie, treasury, };
});

test.afterEach.always(async (t) => {
  await t.context.worker.tearDown().catch((error) => {
    console.log("Failed to tear down the worker:", error);
  });
});

test("nft contract: nft metadata view", async (t) => {
  const { root, nft_contract } = t.context.accounts;
  const expected = {
    base_uri: DEFAULT_BASE_URI,
    icon: "data:image/svg+xml,%3Csvg id='Layer_2' data-name='Layer 2' xmlns='http://www.w3.org/2000/svg' viewBox='0 0 566.1 555.59'%3E%3Cdefs%3E%3Cstyle%3E.cls-1%7Bfill:%23f15a29;%7D%3C/style%3E%3C/defs%3E%3Cpath class='cls-1' d='M708.73,315.12c-51.79-57-126-92.91-208.8-92.91s-157,36-208.7,92.91A284.3,284.3,0,0,0,217,506.91c.05,135,28.75,205.94,75.67,243.24a147.36,147.36,0,0,0,27.31,17.25c-3.78-3-7.42-6.31-10.84-9.66a153.64,153.64,0,0,1-46.47-108.63c-1.75-43.71-2.58-73-3.57-107-.45-17.91-1-37-1.75-60.44,0-88.38,92-154.43,174.26-154.43H568.42c99.77,0,174.26,81.5,174.26,154.83-.71,23-1.23,42-1.78,60-1,34-1.79,63.3-3.54,106.65A156.09,156.09,0,0,1,679,768.18a153.42,153.42,0,0,0,30-18.88c46-37.58,74.06-108.51,74.06-242.39a284.62,284.62,0,0,0-74.32-191.79Z' transform='translate(-216.95 -222.21)'/%3E%3Cpath class='cls-1' d='M564.92,352H435c-74,0-152.76,60-152.76,134,2,63,2.46,94.93,5,158.36.38,48.13,26.18,90,64.43,113.4a134.15,134.15,0,0,0,70.73,20.08H577.51a134.43,134.43,0,0,0,70.78-20.08c38.18-23.46,64-65.23,64.38-113.4,2.58-63.45,3.07-95.39,5.11-158.36C717.75,412,638.92,352,564.92,352ZM422.46,587.21a49.52,49.52,0,0,1-30.31,10.42,51.07,51.07,0,1,1,30.31-10.42ZM607.8,597.65a51.05,51.05,0,0,1-50.46-50.44,50.46,50.46,0,1,1,50.46,50.44Z' transform='translate(-216.95 -222.21)'/%3E%3C/svg%3E",
    name: "A-Runner",
    reference: null,
    reference_hash: null,
    spec: "nft-1.0.0",
    symbol: "RUNNER",
  };
  t.deepEqual(
    await nft_contract.view("nft_metadata", { account_id: root }),
    expected
  );
});

test("nft contract: nft mint call", async (t) => {
  const { alice, nft_contract } = t.context.accounts;
  const request_payload = {
    receiver_id: alice,
  };
  await alice.call(
    nft_contract,
    "nft_mint",
    request_payload,
    defaultCallOptions(DEFAULT_GAS, DEFAULT_DEPOSIT_FOR_MINT)
  );

  const tokens = await nft_contract.view("nft_tokens");
  const expected = [
    {
      approved_account_ids: {},
      metadata: {
        copies: 1,
        description: "Chubby Runners are designed to provide the ultimate play & earn experience. We believe in rewarding players for their effort, skill, and loyalty.",
        expires_at: null,
        extra: null,
        issued_at: null,
        media: "img/0.png",
        media_hash: null,
        reference: "data/0.json",
        reference_hash: null,
        starts_at: null,
        title: "Chubby Runner #0",
        updated_at: null,
      },
      owner_id: alice.accountId,
      royalty: {
          "treasury.test.near": 2000,
      },
      token_id: "0",
    },
  ];
  t.deepEqual(tokens, expected, "Expected to find one minted NFT");
});

test("nft contract: nft approve call", async (t) => {
  const { alice, nft_contract, market_contract } = t.context.accounts;
  await mintNFT(alice, nft_contract);
  await approveNFT(market_contract, alice, nft_contract);

  // test if approved
  const view_payload = {
    token_id: "0",
    approved_account_id: market_contract,
  };
  const approved = await nft_contract.view("nft_is_approved", view_payload);
  t.true(approved, "Failed to approve NFT");
});

test("nft contract: nft approve call long msg string", async (t) => {
  const { alice, nft_contract, market_contract } = t.context.accounts;
  await mintNFT(alice, nft_contract);
  await payForStorage(alice, market_contract);

  // approve NFT
  const approve_payload = {
    token_id: "0",
    account_id: market_contract,
    msg: "sample message".repeat(10 * 1024),
  };
  const result = await alice.callRaw(
    nft_contract,
    "nft_approve",
    approve_payload,
    defaultCallOptions()
  );
  t.regex(result.receiptFailureMessages.join("\n"), /Not valid SaleArgs+/);

  // test if approved
  const view_payload = {
    token_id: "0",
    approved_account_id: market_contract,
  };
  const approved = await nft_contract.view("nft_is_approved", view_payload);
  t.true(approved, "NFT approval apss without sale args");
});

test("cross contract: sell NFT listed on marketplace", async (t) => {
  const { alice, nft_contract, market_contract, bob } = t.context.accounts;
  await mintNFT(alice, nft_contract);
  await payForStorage(alice, market_contract);

  const sale_price = "300000000000000000000000"; // sale price string in yoctoNEAR is 0.3 NEAR
  await placeNFTForSale(market_contract, alice, nft_contract, sale_price);

  const alice_balance_before = await alice.availableBalance();
  const bob_balance_before = await bob.availableBalance();
  await purchaseListedNFT(nft_contract, bob, market_contract, sale_price);
  const alice_balance_after = await alice.availableBalance();
  const bob_balance_after = await bob.availableBalance();

  // assert alice balance increased by sale price
  const test_precision_dp_near = 1;
  const slice_val = test_precision_dp_near - 24;
  t.is(
    alice_balance_after.toString().slice(0, slice_val),
    alice_balance_before.add(NEAR.from(sale_price)).toString().slice(0, slice_val),
    "Alice balance should increase by sale price"
  );
  // bob balance should decrease by sale price
  t.is(
    bob_balance_after.toString().slice(0, slice_val),
    bob_balance_before.sub(NEAR.from(sale_price)).toString().slice(0, slice_val),
    "Bob balance should decrease by sale price"
  );

  // NFT has new owner
  const view_payload = {
    token_id: "0",
  };
  const token_info: any = await nft_contract.view("nft_token", view_payload);
  t.is(token_info.owner_id, bob.accountId, "NFT should have been sold");
  // nothing left for sale on market
  const sale_supply = await market_contract.view("get_supply_sales");
  t.is(sale_supply, "0", "Expected no sales to be left on market");
});

test("cross contract: transfer NFT when listed on marketplace", async (t) => {
  const { alice, nft_contract, market_contract, bob, charlie } = t.context.accounts;
  await mintNFT(alice, nft_contract);
  await payForStorage(alice, market_contract);

  const sale_price = "300000000000000000000000"; // sale price string in yoctoNEAR is 0.3 NEAR
  await placeNFTForSale(market_contract, alice, nft_contract, sale_price);

  await transferNFT(bob, market_contract, nft_contract);

  // purchase NFT
  const offer_payload = {
    nft_contract_id: nft_contract,
    token_id: "0",
  };
  const result = await charlie.callRaw(
    market_contract,
    "offer",
    offer_payload,
    defaultCallOptions(
      DEFAULT_GAS + "0", // 10X default amount for XCC
      sale_price // Attached deposit must be greater than or equal to the current price
    )
  );

  // assert expectations
  // NFT has same owner
  const view_payload = {
    token_id: "0",
  };
  const token_info: any = await nft_contract.view("nft_token", view_payload);
  t.is(
    token_info.owner_id,
    bob.accountId, // NFT was transferred to bob
    "NFT should have bob as owner"
  );
  // Unauthorized error should be found
  t.regex(result.receiptFailureMessages.join("\n"), /Unauthorized+/);
});

test("cross contract: approval revoke", async (t) => {
  const { alice, nft_contract, market_contract, bob } = t.context.accounts;
  await mintNFT(alice, nft_contract);
  await payForStorage(alice, market_contract);
  await placeNFTForSale(
    market_contract,
    alice,
    nft_contract,
    "300000000000000000000000"
  );

  // revoke approval
  const revoke_payload = {
    token_id: "0",
    account_id: market_contract, // revoke market contract authorization
  };
  await alice.call(
    nft_contract,
    "nft_revoke",
    revoke_payload,
    defaultCallOptions(DEFAULT_GAS, "1") // Requires attached deposit of exactly 1 yoctoNEAR
  );

  // transfer NFT
  const transfer_payload = {
    receiver_id: bob,
    token_id: "0",
    approval_id: 1,
  };
  const result = await market_contract.callRaw(
    nft_contract,
    "nft_transfer",
    transfer_payload,
    defaultCallOptions(DEFAULT_GAS, "1")
  );

  // assert expectations
  // Unauthorized error should be found
  t.regex(result.receiptFailureMessages.join("\n"), /Unauthorized+/);
});

test("cross contract: reselling and royalties", async (t) => {
  const { alice, nft_contract, market_contract, bob, charlie, treasury } = t.context.accounts;
  const royalties_string = `{"${alice.accountId}":2000}`;
  const royalties = JSON.parse(royalties_string);
  await mintNFT(alice, nft_contract, royalties);
  await payForStorage(alice, market_contract);
  const ask_price = "300000000000000000000000";

  await placeNFTForSale(market_contract, alice, nft_contract, ask_price);

  const bid_price = ask_price + "0";

  const fee_amount = "600000000000000000000000";

  const alice_balance_before = await alice.availableBalance();
  const bob_balance_before = await bob.availableBalance();
  await purchaseListedNFT(nft_contract, bob, market_contract, bid_price);
  const alice_balance_after = await alice.availableBalance();
  const bob_balance_after = await bob.availableBalance();

  const test_precision_dp_near = 1;
  const slice_val = test_precision_dp_near - 24;
  t.is(
    alice_balance_after.toString().slice(0, slice_val),
    alice_balance_before.add(NEAR.from(bid_price)).sub(NEAR.from(fee_amount)).toString().slice(0, slice_val),
    "Alice balance should increase by sale price and decreased by 20% fee"
  );
  t.is(
    bob_balance_after.toString().slice(0, slice_val),
    bob_balance_before.sub(NEAR.from(bid_price)).toString().slice(0, slice_val),
    "Bob balance should decrease by sale price"
  );

  // bob relists NFT for higher price
  await payForStorage(bob, market_contract);
  const resell_ask_price = bid_price + "0";
  await placeNFTForSale(market_contract, bob, nft_contract, resell_ask_price);

  // bob updates price to lower ask
  const lowered_resell_ask_price = "600000000000000000000000";
  const update_price_payload = {
    nft_contract_id: nft_contract,
    token_id: "0",
    price: lowered_resell_ask_price,
  };
  await bob.call(
    market_contract,
    "update_price",
    update_price_payload,
    defaultCallOptions(DEFAULT_GAS, "1")
  );

  const treasury_balance_before_2 = await treasury.availableBalance();
  const bob_balance_before_2 = await bob.availableBalance();
  const charlie_balance_before_2 = await charlie.availableBalance();
  await purchaseListedNFT(nft_contract, charlie, market_contract, resell_ask_price);
  const treasury_balance_after_2 = await treasury.availableBalance();
  const bob_balance_after_2 = await bob.availableBalance();
  const charlie_balance_after_2 = await charlie.availableBalance();

  t.is(
    treasury_balance_after_2.sub(treasury_balance_before_2).toHuman(),
    "6 N",
    "Treasury balance should increase by royalty fee of 20% of sale price"
  )
  t.is(
    bob_balance_after_2.sub(bob_balance_before_2).toHuman(),
    "24.00031 N",
    "Bob balance should decrease by sale price minus royalty fee of 20% of sale price"
  )
  t.is(
    charlie_balance_before_2.sub(charlie_balance_after_2).toHuman().slice(0, 2),
    "30",
    "Charlie balance should decrease by sale price"
  )
});

test("nft contract: change contract meta", async (t) => {
  const { root, nft_contract } = t.context.accounts;
  const expected = {
    base_uri: "https://channged",
    icon: null,
    name: "Near Runner2",
    reference: null,
    reference_hash: null,
    spec: "nft-1.0.0",
    symbol: "RUNNER",
  };

  await root.call(
      nft_contract,
      "set_meta",
      {
        name: "Near Runner2",
        base_uri: 'https://channged',
        icon: null,
      },
      defaultCallOptions(DEFAULT_GAS,  "1")
  );

  t.deepEqual(
      await nft_contract.view("nft_metadata", { account_id: root }),
      expected
  );
});