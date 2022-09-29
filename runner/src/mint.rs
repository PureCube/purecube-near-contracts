use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: AccountId,
    ) {
        let supply = self.nft_total_supply();
        assert!(supply < self.max_supply, "All tokens minted");

        let next_token_id: TokenId = supply.0.to_string();

        //measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        let mut need_to_attach: U128 = self.mint_price;

        let is_treasury = env::predecessor_account_id() == self.treasury_id;
        let attached_deposit: U128 = U128(env::attached_deposit());

        if is_treasury {
            need_to_attach = U128(1);
        } else {
            assert!(near_sdk::env::block_timestamp() > self.mint_start.0, "Minting will start {}, now {}", self.mint_start.0, near_sdk::env::block_timestamp());
            assert!(near_sdk::env::block_timestamp() < self.mint_end.0, "Minting is over {}, now {}", self.mint_end.0, near_sdk::env::block_timestamp());
        }

        // check attached amount of NEAR
        assert!(
                need_to_attach <= attached_deposit,
                "Must attach {} yoctoNEAR to mint new token, you attached {}",
                self.mint_price.0,
                attached_deposit.0
            );

        if !is_treasury {
            Promise::new(self.treasury_id.clone()).transfer(self.mint_price.0);
        }

        let new_metadata = TokenMetadata {
            title: Some(format!("A-Runner #{}", next_token_id).to_string()),
            description: Some("Near Runner play-to-earn game NFT".to_string()),
            media: Some(format!("img/{}.jpg", next_token_id).to_string()),
            reference: Some(format!("data/{}.json", next_token_id).to_string()),
            copies: Some(1u64),
            media_hash: None,
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference_hash: None,
        };

        //specify the token struct that contains the owner ID 
        let token = Token {
            //set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            //we set the approved account IDs to the default value (an empty map)
            approved_account_ids: Default::default(),
            //the next approval ID is set to 0
            next_approval_id: 0,
        };

        //insert the token ID and token struct and make sure that the token doesn't exist
        assert!(
            self.tokens_by_id.insert(&next_token_id, &token).is_none(),
            "Token already exists"
        );

        //insert the token ID and metadata
        self.token_metadata_by_id.insert(&next_token_id, &new_metadata);

        //call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&token.owner_id, &next_token_id);

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftMint(vec![NftMintLog {
                // Owner of the token.
                owner_id: token.owner_id.to_string(),
                // Vector of token IDs that were minted.
                token_ids: vec![next_token_id.to_string()],
                // An optional memo to include.
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());

        //calculate the required storage which was the used - initial
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.

        //get how much it would cost to store the information and mint cost
        let mut required_cost = env::storage_byte_cost() * Balance::from(required_storage_in_bytes);
        if !is_treasury {
            required_cost += self.mint_price.0;
        }

        //make sure that the attached deposit is greater than or equal to the required cost
        assert!(
            required_cost <= attached_deposit.0,
            "Must attach {} yoctoNEAR to cover storage and mint price",
            required_cost,
        );

        //get the refund amount from the attached deposit - required cost
        let refund = attached_deposit.0 - required_cost;

        //if the refund is greater than 1 yocto NEAR, we refund the predecessor that amount
        if refund > 1 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }
    }
}