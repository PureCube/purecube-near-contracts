import { BN, NearAccount } from "near-workspaces";

export const DEFAULT_GAS: string = "30000000000000";
export const DEFAULT_DEPOSIT: string = "9050000000000000000000";
export const DEFAULT_DEPOSIT_FOR_MINT: string = "5009050000000000000000000";
export const DEFAULT_BASE_URI: string = "https://gateway.pureblock.io/runner-testnet/";
export const MINT_PRICE: string = "5000000000000000000000000";
export const MINT_START: string = "1600000000000000000";
export const MINT_END: string = "1690000000000000000"; // Sat Jul 22 2023 04:26:40

export async function purchaseListedNFT(
  nft_contract: NearAccount,
  bidder_account: NearAccount,
  market_contract: NearAccount,
  bid_price: string
) {
  const offer_payload = {
    nft_contract_id: nft_contract,
    token_id: "0",
  };
  await bidder_account.callRaw(
    market_contract,
    "offer",
    offer_payload,
    defaultCallOptions(DEFAULT_GAS + "0", bid_price)
  );
}

export async function placeNFTForSale(
  market_contract: NearAccount,
  owner: NearAccount,
  nft_contract: NearAccount,
  ask_price: string // sale price string in yoctoNEAR
) {
  await approveNFT(
    market_contract,
    owner,
    nft_contract,
    '{"sale_conditions": ' + `"${ask_price}"` + " }" // msg string trigger XCC
  );
}

export function defaultCallOptions(
  gas: string = DEFAULT_GAS,
  deposit: string = DEFAULT_DEPOSIT
) {
  return {
    gas: new BN(gas),
    attachedDeposit: new BN(deposit),
  };
}
export async function approveNFT(
  account_to_approve: NearAccount,
  owner: NearAccount,
  nft_contract: NearAccount,
  message?: string
) {
  const approve_payload = {
    token_id: "0",
    account_id: account_to_approve,
    msg: message,
  };
  await owner.call(
    nft_contract,
    "nft_approve",
    approve_payload,
    defaultCallOptions()
  );
}

export async function mintNFT(
  user: NearAccount,
  nft_contract: NearAccount,
  royalties?: object
) {
  const mint_payload = {
    receiver_id: user,
  };
  await user.call(nft_contract, "nft_mint", mint_payload, defaultCallOptions(DEFAULT_GAS, DEFAULT_DEPOSIT_FOR_MINT));
}

export async function payForStorage(
  alice: NearAccount,
  market_contract: NearAccount
) {
  await alice.call(
    market_contract,
    "storage_deposit",
    {},
    defaultCallOptions(DEFAULT_GAS, "10000000000000000000000") // Requires minimum deposit of 10000000000000000000000
  );
}

export async function transferNFT(
  receiver: NearAccount,
  sender: NearAccount,
  nft_contract: NearAccount
) {
  const transfer_payload = {
    receiver_id: receiver,
    token_id: "0",
    approval_id: 0, // first and only approval done in line 224
  };
  await sender.call(
    nft_contract,
    "nft_transfer",
    transfer_payload,
    defaultCallOptions(DEFAULT_GAS, "1")
  );
}
