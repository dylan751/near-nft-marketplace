use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap, UnorderedSet};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{collections::LookupMap, AccountId};
use near_sdk::{
    env, ext_contract, init, log, near_bindgen, Balance, CryptoHash, Gas, Promise, PromiseOrValue,
    PromiseResult,
};
use std::collections::HashMap;

pub type TokenId = String;

pub use crate::approval::*;
pub use crate::enumeration::*;
use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::utils::*;

mod approval;
mod enumeration;
mod internal;
mod metadata;
mod mint;
mod nft_core;
mod utils;

// State cơ bản của NFT contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct Contract {
    pub owner_id: AccountId,

    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>, // Lưu danh sách token mà user sở hữu

    pub tokens_by_id: LookupMap<TokenId, Token>, // Mapping token id với các data mở rộng của Token (Danh sách tất cả token đang có trong contract)

    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>, // Mapping token id với token metadata

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
            token_metadata_by_id: UnorderedMap::new(
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

        let token = contract.nft_token(token_id.clone()).unwrap();

        // Test người sở hữu token vừa mint có đúng là accounts(0) không
        assert_eq!(accounts(0).to_string(), token.owner_id);
        assert_eq!(token_id.clone(), token.token_id);
        assert_eq!(token.metadata, get_sample_metadata());
    }

    #[test]
    fn test_transfer_nft() {
        // --- Init test environment ---
        let mut context = get_context(false);
        testing_env!(context.build());

        // --- Init Contract ---
        let mut contract = Contract::new_default_metadata(accounts(0).to_string());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        // --- Tạo 1 token có id là zng_nft ---
        let token_id = "zng_nft".to_owned();
        contract.nft_mint(
            token_id.clone(),
            get_sample_metadata(),
            accounts(0).to_string(),
        );

        let token = contract.nft_token(token_id.clone()).unwrap();
        // --- Kiểm tra các thông tin về owner ---
        assert_eq!(token.owner_id, accounts(0).to_string());
        assert_eq!(token.token_id, token_id);
        assert_eq!(get_sample_metadata(), token.metadata);

        testing_env!(context.attached_deposit(1).build());

        // --- Transfer nft từ accounts(0) sang accounts(1)
        contract.nft_transfer(accounts(1).to_string(), token_id.clone(), 1, None);

        let new_token = contract.nft_token(token_id.clone()).unwrap();
        // --- Kiểm tra các thông tin về owner
        assert_eq!(new_token.owner_id, accounts(1).to_string());
        assert_eq!(new_token.token_id, token_id);
        assert_eq!(get_sample_metadata(), new_token.metadata);
    }
}
