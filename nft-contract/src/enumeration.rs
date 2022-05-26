// Lấy danh sách token của User
use crate::*;

// Các hàm theo chuẩn NEP-181 của NEAR - Enumeration
// Xem thêm tại: https://nomicon.io/Standards/Tokens/NonFungibleToken/Enumeration
#[near_bindgen]
impl Contract {
    // Lấy tổng số token đang có trong contract
    pub fn nft_total_supply(&self) -> U128 {
        // Đếm tổng số lượng id đang có trong token_metadata_by_id
        U128(self.token_metadata_by_id.len() as u128)
    }

    // Lấy tổng số token đang có của account nào đó
    pub fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        let token_for_owner_set = self.tokens_per_owner.get(&account_id);

        if let Some(token_for_owner_set) = token_for_owner_set {
            U128(token_for_owner_set.len() as u128)
        } else {
            U128(0)
        }
    }

    // Lấy danh sách token (có pagination)
    pub fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<JsonToken> {
        let token_keys = self.token_metadata_by_id.keys_as_vector();

        let start = u128::from(from_index.unwrap_or(U128(0)));

        // Duyệt tất cả các keys -> Trả về JsonToken
        token_keys
            .iter()
            .skip(start as usize) // Pagination
            .take(limit.unwrap_or(0) as usize) // Pagination
            .map(|token_id| self.nft_token(token_id.clone()).unwrap())
            .collect()
    }

    // Lấy danh sách token của account nào đó (có pagination)
    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonToken> {
        let token_keys = self.tokens_per_owner.get(&account_id);

        let keys = if let Some(token_keys) = token_keys {
            token_keys
        } else {
            return vec![];
        };

        let start = u128::from(from_index.unwrap_or(U128(0)));

        // Duyệt tất cả các keys -> Trả về JsonToken
        keys.as_vector()
            .iter()
            .skip(start as usize) // Pagination
            .take(limit.unwrap_or(0) as usize) // Pagination
            .map(|token_id| self.nft_token(token_id.clone()).unwrap())
            .collect()
    }
}
