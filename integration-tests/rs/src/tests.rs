use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{network::Sandbox, Account, Contract, Worker};

mod helpers;

const NFT_WASM_FILEPATH: &str = "../../out/main.wasm";
const MARKET_WASM_FILEPATH: &str = "../../out/market.wasm";
const DEFAULT_BASE_URI: &str = "https://gateway.purecube.io/cubby-runners-testnet";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initiate environemnt
    let worker = workspaces::sandbox().await?;

    // deploy contracts
    let nft_wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let nft_contract = worker.dev_deploy(&nft_wasm).await?;
    let market_wasm = std::fs::read(MARKET_WASM_FILEPATH)?;
    let market_contract = worker.dev_deploy(&market_wasm).await?;

    // create accounts
    let owner = worker.root_account();
    let alice = owner
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("31 N"))
        .transact()
        .await?
        .into_result()?;
    let bob = owner
        .create_subaccount(&worker, "bob")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let charlie = owner
        .create_subaccount(&worker, "charlie")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let treasury = owner
            .create_subaccount(&worker, "treasury")
//             .initial_balance(parse_near!("30 N"))
            .transact()
            .await?
            .into_result()?;

    // Initialize contracts
    nft_contract
        .call(&worker, "new_default_meta")
        .args_json(serde_json::json!({"owner_id": owner.id(), "treasury_id": treasury.id(), "max_supply": "9", "base_uri": DEFAULT_BASE_URI, "mint_price": "5000000000000000000000000","mint_start": "1", "mint_end": "1690000000000000000", "perpetual_royalties": {treasury.id().to_string(): 2000}}))?
        .transact()
        .await?;
    market_contract
        .call(&worker, "new")
        .args_json(serde_json::json!({"owner_id": owner.id()}))?
        .transact()
        .await?;

    // begin tests
    test_nft_metadata_view(&owner, &nft_contract, &worker).await?;
    test_nft_mint_call(&owner, &alice, &nft_contract, &treasury, &worker).await?;
    test_nft_approve_call(&bob, &nft_contract, &market_contract, &worker).await?;
    test_nft_approve_call_long_msg_string(&alice, &nft_contract, &market_contract, &worker).await?;
    test_sell_nft_listed_on_marketplace(&alice, &nft_contract, &market_contract, &bob, &worker).await?;
    test_transfer_nft_when_listed_on_marketplace(&alice, &bob, &charlie, &nft_contract, &market_contract, &worker).await?;
    test_approval_revoke(&alice, &bob, &nft_contract, &market_contract, &worker).await?;
    test_reselling_and_royalties(&alice, &bob, &charlie, &nft_contract, &market_contract, &treasury, &worker).await?;

    Ok(())
}

async fn test_nft_metadata_view(
    owner: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let expected = json!({
        "base_uri": DEFAULT_BASE_URI,
        "icon": "data:image/svg+xml,%3Csvg id='Layer_2' data-name='Layer 2' xmlns='http://www.w3.org/2000/svg' viewBox='0 0 566.1 555.59'%3E%3Cdefs%3E%3Cstyle%3E.cls-1%7Bfill:%23f15a29;%7D%3C/style%3E%3C/defs%3E%3Cpath class='cls-1' d='M708.73,315.12c-51.79-57-126-92.91-208.8-92.91s-157,36-208.7,92.91A284.3,284.3,0,0,0,217,506.91c.05,135,28.75,205.94,75.67,243.24a147.36,147.36,0,0,0,27.31,17.25c-3.78-3-7.42-6.31-10.84-9.66a153.64,153.64,0,0,1-46.47-108.63c-1.75-43.71-2.58-73-3.57-107-.45-17.91-1-37-1.75-60.44,0-88.38,92-154.43,174.26-154.43H568.42c99.77,0,174.26,81.5,174.26,154.83-.71,23-1.23,42-1.78,60-1,34-1.79,63.3-3.54,106.65A156.09,156.09,0,0,1,679,768.18a153.42,153.42,0,0,0,30-18.88c46-37.58,74.06-108.51,74.06-242.39a284.62,284.62,0,0,0-74.32-191.79Z' transform='translate(-216.95 -222.21)'/%3E%3Cpath class='cls-1' d='M564.92,352H435c-74,0-152.76,60-152.76,134,2,63,2.46,94.93,5,158.36.38,48.13,26.18,90,64.43,113.4a134.15,134.15,0,0,0,70.73,20.08H577.51a134.43,134.43,0,0,0,70.78-20.08c38.18-23.46,64-65.23,64.38-113.4,2.58-63.45,3.07-95.39,5.11-158.36C717.75,412,638.92,352,564.92,352ZM422.46,587.21a49.52,49.52,0,0,1-30.31,10.42,51.07,51.07,0,1,1,30.31-10.42ZM607.8,597.65a51.05,51.05,0,0,1-50.46-50.44,50.46,50.46,0,1,1,50.46,50.44Z' transform='translate(-216.95 -222.21)'/%3E%3C/svg%3E",
        "name": "A-Runner",
        "reference": serde_json::Value::Null,
        "reference_hash": serde_json::Value::Null,
        "spec": "nft-1.0.0",
        "symbol": "RUNNER",
    });
    let res: serde_json::Value = owner
        .call(&worker, contract.id(), "nft_metadata")
        .args_json(json!({  "account_id": owner.id()  }))?
        .transact()
        .await?
        .json()?;
    assert_eq!(res, expected);
    println!("      Passed ✅ test_nft_metadata_view");
    Ok(())
}

async fn test_nft_mint_call(
    owner: &Account,
    user: &Account,
    contract: &Contract,
    treasury: &Account,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let request_payload = json!({
        "token_id": "1",
        "receiver_id": user.id(),
        "metadata": {
            "title": "LEEROYYYMMMJENKINSSS",
            "description": "Alright time's up, let's do this.",
            "media": "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1"
        },
    });

    user.call(&worker, contract.id(), "nft_mint")
        .args_json(request_payload)?
        .deposit(parse_near!("5.008 N"))
        .transact()
        .await?;

    let tokens: serde_json::Value = owner
        .call(&worker, contract.id(), "nft_tokens")
        .args_json(serde_json::json!({}))?
        .transact()
        .await?
        .json()?;

    let expected = json!([
        {   
            "approved_account_ids": {},
            "royalty": {
                treasury.id().to_string(): 2000,
            },
            "token_id": "0",
            "owner_id": user.id(),
            "metadata": {
                "expires_at": serde_json::Value::Null, 
                "extra": serde_json::Value::Null, 
                "issued_at": serde_json::Value::Null, 
                "copies": 1,
                "media_hash": serde_json::Value::Null,
                "reference": "data/0.json",
                "reference_hash": serde_json::Value::Null,
                "starts_at": serde_json::Value::Null,
                "updated_at": serde_json::Value::Null,
                "title": "Chubby Runner #0",
                "description": "Chubby Runners are designed to provide the ultimate play & earn experience. We believe in rewarding players for their effort, skill, and loyalty.",
                "media": "img/0.png"
            }
        }
    ]);

    assert_eq!(tokens, expected);
    println!("      Passed ✅ test_nft_mint_call");
    Ok(())
}

async fn test_nft_approve_call(
    user: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "1";
    helpers::mint_nft(user, nft_contract, worker).await?;
    helpers::approve_nft(market_contract, user, nft_contract, worker, token_id).await?;

    let view_payload = json!({
        "token_id": token_id,
        "approved_account_id": market_contract.id(),
    });
    let result: bool = user
        .call(&worker, nft_contract.id(), "nft_is_approved")
        .args_json(view_payload)?
        .transact()
        .await?
        .json()?;
    
    assert_eq!(result, true);
    println!("      Passed ✅ test_nft_approve_call");
    Ok(())
}

async fn test_nft_approve_call_long_msg_string(
    user: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "2";
    helpers::mint_nft(user, nft_contract, worker).await?;
    helpers::pay_for_storage(user, market_contract, worker, 10000000000000000000000).await?;

    let approve_payload  = json!({
        "token_id": token_id,
        "account_id": market_contract.id(),
        "msg": "sample message".repeat(10240),
    });

    match user.call(&worker, nft_contract.id(), "nft_approve")
        .args_json(approve_payload)?
        .deposit(helpers::DEFAULT_DEPOSIT)
        .gas(helpers::DEFAULT_GAS as u64)
        .transact()
        .await
    {
        Ok(_result) => {
            panic!("test_nft_approve_call_long_msg_string worked despite insufficient gas")
        }
        Err(e) => {
            let e_string = e.to_string();
            if !e_string
                .contains("Not valid SaleArgs")
            {
                panic!("test_nft_approve_call_long_msg_string displays unexpected error message: {:?}", e_string);
            }

            let view_payload = json!({
                "token_id": token_id,
                "approved_account_id": market_contract.id(),
            });
            let result: bool = user
                .call(&worker, nft_contract.id(), "nft_is_approved")
                .args_json(view_payload)?
                .transact()
                .await?
                .json()?;
            
            assert_eq!(result, true);
            println!("      Passed ✅ test_nft_approve_call_long_msg_string");
        }
    }
    Ok(())
}

async fn test_sell_nft_listed_on_marketplace(
    seller: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    buyer: &Account,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "3";
    let sale_price = 300000000000000000000000 as u128;  // 0.3 NEAR in yoctoNEAR
    let fee = 60000000000000000000000 as u128;  // 0.06 NEAR in yoctoNEAR
    helpers::mint_nft(seller, nft_contract, worker).await?;
    helpers::pay_for_storage(seller, market_contract, worker, 10000000000000000000000 as u128).await?;
    helpers::place_nft_for_sale(seller, market_contract, nft_contract, worker, token_id, sale_price).await?;

    let before_seller_balance: u128 = helpers::get_user_balance(seller, worker).await?;
    let before_buyer_balance: u128 = helpers::get_user_balance(buyer, worker).await?;
    helpers::purchase_listed_nft(buyer, market_contract, nft_contract, worker, token_id, sale_price).await?;
    let after_seller_balance: u128 = helpers::get_user_balance(seller, worker).await?;
    let after_buyer_balance: u128 = helpers::get_user_balance(buyer, worker).await?;

    let dp = 1;  // being exact requires keeping track of gas usage 
    assert_eq!(helpers::round_to_near_dp(after_seller_balance, dp), helpers::round_to_near_dp(before_seller_balance + sale_price - fee, dp), "seller did not receive the sale price without trade fee");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance, dp), helpers::round_to_near_dp(before_buyer_balance - sale_price, dp), "buyer did not receive the sale price");

    println!("      Passed ✅ test_sell_nft_listed_on_marketplace");
    Ok(())
}

async fn test_transfer_nft_when_listed_on_marketplace(
    seller: &Account,
    first_buyer: &Account,
    second_buyer: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "4";
    let sale_price = 3000000000000000000000000 as u128;  // 3 NEAR in yoctoNEAR
    helpers::mint_nft(seller, nft_contract, worker).await?;
    helpers::pay_for_storage(seller, market_contract, worker, 10000000000000000000000 as u128).await?;
    helpers::place_nft_for_sale(seller, market_contract, nft_contract, worker, token_id, sale_price).await?;

    helpers::transfer_nft(seller, first_buyer, nft_contract, worker, token_id).await?;

    // attempt purchase NFT
    let before_seller_balance: u128 = helpers::get_user_balance(seller, worker).await?;
    let before_buyer_balance: u128 = helpers::get_user_balance(second_buyer, worker).await?;
    helpers::purchase_listed_nft(second_buyer, market_contract, nft_contract, worker, token_id, sale_price).await?;
    let after_seller_balance: u128 = helpers::get_user_balance(seller, worker).await?;
    let after_buyer_balance: u128 = helpers::get_user_balance(second_buyer, worker).await?;

    // assert owner remains first_buyer
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, worker, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, first_buyer.id().to_string(), "token owner is not first_buyer");

    // assert balances remain equal
    let dp = 1;     
    assert_eq!(helpers::round_to_near_dp(after_seller_balance, dp), helpers::round_to_near_dp(before_seller_balance, dp), "seller balance changed");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance, dp), helpers::round_to_near_dp(before_buyer_balance, dp), "buyer balance changed");

    println!("      Passed ✅ test_transfer_nft_when_listed_on_marketplace");

    Ok(())
}

async fn test_approval_revoke(
    first_user: &Account,
    second_user: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "5";
    let sale_price = 3000000000000000000000000 as u128;  // 3 NEAR in yoctoNEAR
    helpers::mint_nft(first_user, nft_contract, worker).await?;
    helpers::pay_for_storage(first_user, market_contract, worker, 10000000000000000000000 as u128).await?;
    helpers::place_nft_for_sale(first_user, market_contract, nft_contract, worker, token_id, sale_price).await?;

    // nft_revoke market_contract call
    let revoke_payload = json!({
        "account_id": market_contract.id(),
        "token_id": token_id,
    });
    first_user.call(&worker, nft_contract.id(), "nft_revoke")
        .args_json(revoke_payload)?
        .deposit(1)
        .transact()
        .await?;

    // market_contract attempts to nft_transfer, when second_user tries to purchase NFT on market
    let before_seller_balance: u128 = helpers::get_user_balance(first_user, worker).await?;
    let before_buyer_balance: u128 = helpers::get_user_balance(second_user, worker).await?;
    helpers::purchase_listed_nft(
        second_user, market_contract, nft_contract, worker, token_id, sale_price
    ).await?;
    let after_seller_balance: u128 = helpers::get_user_balance(first_user, worker).await?;
    let after_buyer_balance: u128 = helpers::get_user_balance(second_user, worker).await?;

    // assert owner remains first_user
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, worker, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, first_user.id().to_string(), "token owner is not first_user");

    // assert balances unchanged
    assert_eq!(helpers::round_to_near_dp(after_seller_balance, 0), helpers::round_to_near_dp(before_seller_balance, 0), "seller balance changed");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance, 0), helpers::round_to_near_dp(before_buyer_balance, 0), "buyer balance changed");    

    println!("      Passed ✅ test_approval_revoke");
    Ok(())
}

async fn test_reselling_and_royalties(
    user: &Account,
    first_buyer: &Account,
    second_buyer: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    treasury: &Account,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "6";
    let sale_price = 3000000000000000000000000 as u128;  // 3 NEAR in yoctoNEAR
    let fee = 600000000000000000000000 as u128;  // 0.6 NEAR in yoctoNEAR

    // mint
    let request_payload = json!({
        "receiver_id": user.id(),
    });
    user.call(&worker, nft_contract.id(), "nft_mint")
        .args_json(request_payload)?
        .deposit(parse_near!("5.1 N"))
        .transact()
        .await?;

    helpers::pay_for_storage(user, market_contract, worker, 10000000000000000000000 as u128).await?;
    helpers::place_nft_for_sale(user, market_contract, nft_contract, worker, token_id, sale_price).await?;

    // first_buyer purchases NFT
    let mut before_seller_balance: u128 = helpers::get_user_balance(user, worker).await?;
    let mut before_buyer_balance: u128 = helpers::get_user_balance(first_buyer, worker).await?;
    helpers::purchase_listed_nft(first_buyer, market_contract, nft_contract, worker, token_id, sale_price).await?;
    let mut after_seller_balance: u128 = helpers::get_user_balance(user, worker).await?;
    let mut after_buyer_balance: u128 = helpers::get_user_balance(first_buyer, worker).await?;

    // assert owner becomes first_buyer
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, worker, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, first_buyer.id().to_string(), "token owner is not first_buyer");

    // assert balances changed
    assert_eq!(helpers::round_to_near_dp(after_seller_balance, 0), helpers::round_to_near_dp(before_seller_balance + sale_price - fee, 0), "seller balance unchanged");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance, 0), helpers::round_to_near_dp(before_buyer_balance - sale_price, 0), "buyer balance unchanged");

    // first buyer lists nft for sale
    helpers::pay_for_storage(first_buyer, market_contract, worker, 10000000000000000000000 as u128).await?;
    helpers::place_nft_for_sale(first_buyer, market_contract, nft_contract, worker, token_id, sale_price).await?;

    // second_buyer purchases NFT
    let resale_price = sale_price * 5;  // 15 NEAR
    before_seller_balance = helpers::get_user_balance(first_buyer, worker).await?;
    before_buyer_balance = helpers::get_user_balance(second_buyer, worker).await?;
    let before_user_balance: u128 = helpers::get_user_balance(treasury, worker).await?;
    helpers::purchase_listed_nft(second_buyer, market_contract, nft_contract, worker, token_id, resale_price).await?;
    let after_user_balance: u128 = helpers::get_user_balance(treasury, worker).await?;
    after_seller_balance = helpers::get_user_balance(first_buyer, worker).await?;
    after_buyer_balance = helpers::get_user_balance(second_buyer, worker).await?;

    // assert owner changes to second_buyer
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, worker, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, second_buyer.id().to_string(), "token owner is not second_buyer");

    // assert balances changed
    let royalty_fee = resale_price / 5;
    assert_eq!(helpers::round_to_near_dp(after_seller_balance, 0), helpers::round_to_near_dp(before_seller_balance + resale_price - royalty_fee, 0), "seller balance unchanged");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance, 0), helpers::round_to_near_dp(before_buyer_balance - resale_price, 0), "buyer balance unchanged");
    assert_eq!(helpers::round_to_near_dp(after_user_balance, 0), helpers::round_to_near_dp(before_user_balance + royalty_fee, 0), "user balance unchanged");

    println!("      Passed ✅ test_reselling_and_royalties");
    Ok(())
}
