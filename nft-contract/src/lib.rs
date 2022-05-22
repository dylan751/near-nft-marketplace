use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedSet};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{collections::LookupMap, AccountId};
use near_sdk::{env, init, near_bindgen, Balance, CryptoHash, Promise};

pub type TokenId = String;

use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::utils::*;

mod internal;
mod metadata;
mod mint;
mod utils;

// State cơ bản của NFT contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct Contract {
    pub owner_id: AccountId,

    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>, // Lưu danh sách token mà user sở hữu

    pub tokens_by_id: LookupMap<TokenId, Token>, // Mapping token id với các data mở rộng của Token

    pub token_metadata_by_id: LookupMap<TokenId, TokenMetadata>, // Mapping token id với token metadata

    pub metadata: LazyOption<NFTContractMetadata>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum StorageKey {
    TokenPerOwnerKey,
    ContractMetadataKey,
    TokenByIdKey,
    TokenMetadataByIdKey,
    TokenPerOwnerInnerKey {
        account_id_hash: CryptoHash, // Để đảm bảo các account_id không trùng nhau
    },
}

impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, token_metadata: NFTContractMetadata) -> Self {
        Self {
            owner_id,
            metadata: LazyOption::new(
                StorageKey::ContractMetadataKey.try_to_vec().unwrap(),
                Some(&token_metadata),
            ),
            tokens_per_owner: LookupMap::new(StorageKey::TokenPerOwnerKey.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokenByIdKey.try_to_vec().unwrap()),
            token_metadata_by_id: LookupMap::new(
                StorageKey::TokenMetadataByIdKey.try_to_vec().unwrap(),
            ),
        }
    }

    #[init]
    pub fn new_default_metadata(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: "zng-nft-2.0.0".to_string(),
                name: "ZNG NFT".to_string(),
                symbol: "ZNGT".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }
}

#[cfg(all(test, not(target_arch = "wasm-32")))]
mod tests {
    use super::*;

    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_sdk::MockedBlockchain;

    const MINT_STORAGE_COST: u128 = 58_700_000_000_000_000_000_000;

    fn get_context(is_view: bool) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .is_view(is_view);

        builder
    }

    fn get_sample_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("TOKEN TEST".to_owned()),
            description: Some("Description".to_owned()),
            media: None,
            media_hash: None,
            copies: None,
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_mint_token() {
        let mut context = get_context(false);
        testing_env!(context.build());

        // Init contract
        let mut contract = Contract::new_default_metadata(accounts(0).to_string());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "ZNG_NFT".to_string();
        contract.nft_mint(
            token_id.clone(),
            get_sample_metadata(),
            accounts(0).to_string(),
        );

        let token = contract.ntf_token(token_id.clone()).unwrap();

        // Test người sở hữu token vừa mint có đúng là accounts(0) không
        assert_eq!(accounts(0).to_string(), token.owner_id);
        assert_eq!(token_id.clone(), token.token_id);
        assert_eq!(token.metadata, get_sample_metadata());
    }
}
