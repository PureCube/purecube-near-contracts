use std::collections::HashMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, PromiseOrValue,
};

use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::approval::*;
pub use crate::royalty::*;
pub use crate::events::*;

mod internal;
mod approval; 
mod enumeration; 
mod metadata; 
mod mint; 
mod nft_core; 
mod royalty; 
mod events;

/// This spec can be treated like a version of the standard.
pub const NFT_METADATA_SPEC: &str = "1.0.0";
/// This is the name of the NFT standard we're using
pub const NFT_STANDARD_NAME: &str = "nep171";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    //contract owner
    pub owner_id: AccountId,

    //keeps track of all the token IDs for a given account
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    //keeps track of the token struct for a given token ID
    pub tokens_by_id: UnorderedMap<TokenId, Token>,

    //keeps track of the metadata for the contract
    pub metadata: LazyOption<NFTContractMetadata>,

    //treasury address
    pub treasury_id: AccountId,

    //price of mint new token
    pub mint_price: U128,

    //max total supply
    pub max_supply: U128,

    pub mint_start: U64,

    pub mint_end: U64,

    pub perpetual_royalties: HashMap<AccountId, u32>,
}

const DATA_IMAGE_SVG_ICON: &str = "data:image/svg+xml,%3Csvg id='Layer_2' data-name='Layer 2' xmlns='http://www.w3.org/2000/svg' viewBox='0 0 566.1 555.59'%3E%3Cdefs%3E%3Cstyle%3E.cls-1%7Bfill:%23f15a29;%7D%3C/style%3E%3C/defs%3E%3Cpath class='cls-1' d='M708.73,315.12c-51.79-57-126-92.91-208.8-92.91s-157,36-208.7,92.91A284.3,284.3,0,0,0,217,506.91c.05,135,28.75,205.94,75.67,243.24a147.36,147.36,0,0,0,27.31,17.25c-3.78-3-7.42-6.31-10.84-9.66a153.64,153.64,0,0,1-46.47-108.63c-1.75-43.71-2.58-73-3.57-107-.45-17.91-1-37-1.75-60.44,0-88.38,92-154.43,174.26-154.43H568.42c99.77,0,174.26,81.5,174.26,154.83-.71,23-1.23,42-1.78,60-1,34-1.79,63.3-3.54,106.65A156.09,156.09,0,0,1,679,768.18a153.42,153.42,0,0,0,30-18.88c46-37.58,74.06-108.51,74.06-242.39a284.62,284.62,0,0,0-74.32-191.79Z' transform='translate(-216.95 -222.21)'/%3E%3Cpath class='cls-1' d='M564.92,352H435c-74,0-152.76,60-152.76,134,2,63,2.46,94.93,5,158.36.38,48.13,26.18,90,64.43,113.4a134.15,134.15,0,0,0,70.73,20.08H577.51a134.43,134.43,0,0,0,70.78-20.08c38.18-23.46,64-65.23,64.38-113.4,2.58-63.45,3.07-95.39,5.11-158.36C717.75,412,638.92,352,564.92,352ZM422.46,587.21a49.52,49.52,0,0,1-30.31,10.42,51.07,51.07,0,1,1,30.31-10.42ZM607.8,597.65a51.05,51.05,0,0,1-50.46-50.44,50.46,50.46,0,1,1,50.46,50.44Z' transform='translate(-216.95 -222.21)'/%3E%3C/svg%3E";

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
}

#[near_bindgen]
impl Contract {
    /*
        initialization function (can only be called once).
        this initializes the contract with default metadata so the
        user doesn't have to manually type metadata.
    */
    #[init]
    pub fn new_default_meta(
        owner_id: AccountId,
        treasury_id: AccountId,
        max_supply: U128,
        base_uri: String,
        mint_price: U128,
        mint_start: U64,
        mint_end: U64,
        perpetual_royalties: Option<HashMap<AccountId, u32>>
      ) -> Self {
        //calls the other function "new: with some default metadata and the owner_id passed in 
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: format!("nft-{}", NFT_METADATA_SPEC).to_string(),
                name: "Chubby Runners".to_string(),
                symbol: "RUNNER".to_string(),
                icon: Some(DATA_IMAGE_SVG_ICON.to_string()),
                base_uri: Some(base_uri.to_string()),
                reference: None,
                reference_hash: None,
            },
            treasury_id,
            max_supply,
            mint_price,
            mint_start,
            mint_end,
            perpetual_royalties,
        )
    }

    /*
        initialization function (can only be called once).
        this initializes the contract with metadata that was passed in and
        the owner_id. 
    */
    #[init]
    pub fn new(
        owner_id: AccountId,
        metadata: NFTContractMetadata,
        treasury_id: AccountId,
        max_supply: U128,
        mint_price: U128,
        mint_start: U64,
        mint_end: U64,
        perpetual_royalties: Option<HashMap<AccountId, u32>>
    ) -> Self {
            // create a royalty map to store in the contract
            let mut royalty = HashMap::new();

            // if perpetual royalties were passed into the function:
            if let Some(perpetual_royalties) = perpetual_royalties {
                //make sure that the length of the perpetual royalties is below 7 since we won't have enough GAS to pay out that many people
                assert!(perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");

                //iterate through the perpetual royalties and insert the account and amount in the royalty map
                for (account, amount) in perpetual_royalties {
                    royalty.insert(account, amount);
                }
            }

        //create a variable of type Self with all the fields initialized. 
        let this = Self {
            //Storage keys are simply the prefixes used for the collections. This helps avoid data collision
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: UnorderedMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            //set the owner_id field equal to the passed in owner_id.
            owner_id,
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
            treasury_id,
            mint_price,
            mint_start,
            mint_end,
            max_supply,
            perpetual_royalties: royalty,
        };

        //return the Contract object
        this
    }

    #[payable]
    pub fn set_meta(
        &mut self,
        name: String,
        base_uri: String,
        icon: Option<String>,
        max_supply: U128,
    ) {
        assert_eq!(
              &env::predecessor_account_id(),
              &self.owner_id,
              "Predecessor must be contract owner."
        );
        assert!(
            base_uri.len() <= 100,
            "Base URI must be less then 100 chars"
        );

        self.max_supply = max_supply;

        self.metadata = LazyOption::new(
           StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
           Some(&NFTContractMetadata {
                   spec: "nft-1.0.0".to_string(),
                   name: name.to_string(),
                   symbol: "RUNNER".to_string(),
                   icon,
                   base_uri: Some(base_uri.to_string()),
                   reference: None,
                   reference_hash: None,
           }),
        )

        //todo: event log
    }
}

#[cfg(test)]
mod tests;