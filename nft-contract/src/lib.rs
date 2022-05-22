use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{collections::LookupMap, AccountId};
use near_sdk::{init, near_bindgen};

pub type TokenId = String;

use crate::metadata::*;

mod metadata;

// State cơ bản của NFT contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct Contract {
    pub owner_id: AccountId,

    pub tokens_per_owner: LookupMap<AccountId, TokenId>, // Lưu danh sách token mà user sở hữu

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
