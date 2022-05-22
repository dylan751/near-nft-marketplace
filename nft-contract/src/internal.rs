use crate::*;

#[near_bindgen]
impl Contract {
    // Add 1 token vào danh sách sở hữu bởi owner
    pub(crate) fn internal_add_token_to_owner(
        &mut self,
        token_id: &TokenId,
        account_id: &AccountId,
    ) {
        // Nếu account_id đã có danh sách token rồi, thì sẽ lấy danh sách token đang có
        // Nếu account_id chưa có danh sách token (account_id chưa có trong tokens_per_owner) thì tạo mới tokens_set
        let mut tokens_set = self.tokens_per_owner.get(account_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::TokenPerOwnerInnerKey {
                    account_id_hash: hash_account_id(account_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        // Thêm token vào danh sách sở hữu của account_id
        tokens_set.insert(&token_id);

        // Update dữ liệu on-chain
        self.tokens_per_owner.insert(account_id, &tokens_set);
    }
}
