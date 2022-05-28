use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::serde::{self, Deserialize, Serialize};
use near_sdk::serde_json::{Deserializer, Serializer};
use near_sdk::{env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise};

use crate::nft_callback::*;
use crate::sale_view::*;
use crate::utils::*;

// Coi như sau mỗi lần bán qua lại thì tăng storage lên 1000 bytes
const STORAGE_PER_SALE: u128 = 1000 * env::STORAGE_PRICE_PER_BYTE;

mod nft_callback;
mod sale_view;
mod utils;

pub type TokenId = String;
pub type NFTContractId = String;
pub type SalePriceInYoctoNear = U128;
// Để nếu có 2 Contract khác nhau cùng sử dụng market-contract này thì nếu trùng token id cũng ko sao
// Có dạng nft.duongnh.testnet.ZNG_NFT#01
pub type ContractAndTokenId = String;

// Struct cho việc mua bán
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    pub owner_id: AccountId,
    pub approval_id: u64,
    pub nft_contract_id: NFTContractId,
    pub token_id: TokenId,
    // Các điều kiện của sales (Giá, ...)
    pub sale_conditions: SalePriceInYoctoNear,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // Owner of contract
    pub owner_id: AccountId,
    // Danh sách sales của token
    pub sales: UnorderedMap<ContractAndTokenId, Sale>,
    // Danh sách token_id đang được đăng bán của 1 account_id
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>,
    // Danh sách token_id đang được đăng bán của 1 nft contract
    pub by_contract_id: LookupMap<NFTContractId, UnorderedSet<TokenId>>,
    // Danh sách account deposit để cover storage
    pub storage_deposit: LookupMap<AccountId, Balance>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum StorageKey {
    SaleKey,
    ByOwnerIdKey,
    InnerByOwnerIdKey {
        // Trong UnorderedSet của ContractAndTokenId, muốn mỗi phần tử có 1 key riêng để đảm bảo ko trùng nhau
        account_id_hash: CryptoHash,
    },
    InnerByContractIdKey {
        // Trong UnorderedSet của TokenId, muốn mỗi phần tử có 1 key riêng để đảm bảo ko trùng nhau
        account_id_hash: CryptoHash,
    },
    ByContractIdKey,
    StorageDepositKey,
}

#[near_bindgen]
impl Contract {
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            sales: UnorderedMap::new(StorageKey::SaleKey.try_to_vec().unwrap()),
            by_owner_id: LookupMap::new(StorageKey::ByOwnerIdKey.try_to_vec().unwrap()),
            by_contract_id: LookupMap::new(StorageKey::ByContractIdKey.try_to_vec().unwrap()),
            storage_deposit: LookupMap::new(StorageKey::StorageDepositKey.try_to_vec().unwrap()),
        }
    }

    // Cho phép user deposit 1 lượng Near vào contract để cover phí storage
    // User có thể deposit cho account khác
    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        // Nếu có gắn account_id -> deposit cho account_id
        // Nếu không có account_id -> deposit cho người gọi hàm
        let storage_account_id = account_id.unwrap_or(env::predecessor_account_id());
        let deposit = env::attached_deposit();

        assert!(
            deposit >= STORAGE_PER_SALE,
            "Required deposit minimum of {}",
            STORAGE_PER_SALE
        );

        // Cộng thêm số tiền deposit vào storage_deposit của account_id
        let mut balance = self.storage_deposit.get(&storage_account_id).unwrap_or(0);
        balance += deposit;

        // Update dữ liệu
        self.storage_deposit.insert(&storage_account_id, &balance);
    }

    // Cho phép người dùng rút lại tiền đã deposit mà đang ko dùng để lưu trữ data gì cả
    #[payable]
    pub fn storage_withdraw(&mut self) {
        assert_one_yocto();
        let owner_id = env::predecessor_account_id();

        // Lấy ra lượng tiền đã deposit của user, đồng thời xoá user khỏi list đã deposit luôn
        let mut amount = self.storage_deposit.remove(&owner_id).unwrap_or(0);

        // Tính tổng tiền cần để cover storage của user
        // Lượng tiền đã deposit thừa ra thì refund lại cho user
        let sales = self.by_owner_id.get(&owner_id); // Danh sách các token đang đăng bán của user

        let len = sales.map(|s| s.len()).unwrap_or_default();

        // VD: user đang đăng bán 3 tokens
        // -> lượng tiền để cover data storage = 3 * lượng tiền cần cho mỗi tokens
        let storage_required = u128::from(len) * STORAGE_PER_SALE;

        // Check xem lượng deposit hiện tại có cover được data storage ko
        assert!(amount >= storage_required);

        // Tính lượng tiền thừa ra để cover storage của user
        let diff = amount - storage_required;

        // Nếu thừa -> transfer lại cho user
        if diff > 0 {
            Promise::new(owner_id.clone()).transfer(diff);
        }

        // Nếu user còn lưu trữ data -> Cập nhật lại thông tin trong list storage_deposit
        if storage_required > 0 {
            self.storage_deposit.insert(&owner_id, &storage_required);
        }
    }

    pub fn storage_minimun_balance(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }

    // Check lượng storage đã deposit của account_id
    pub fn storage_balance_of(&self, account_id: Option<AccountId>) -> U128 {
        let owner_id = account_id.unwrap_or(env::predecessor_account_id());

        U128(self.storage_deposit.get(&owner_id).unwrap_or(0))
    }
}
