use crate::*;

#[near_bindgen]
impl Contract {
    // Thêm 1 token vào danh sách sở hữu bởi owner
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

    // Xoá token khỏi owner
    pub(crate) fn internal_remove_token_from_owner(&mut self, token_id: &TokenId, account_id: &AccountId) {
        let mut tokens_set = self.tokens_per_owner.get(account_id).expect("Token should be owned by sender");

        // Xoá token_id khỏi tokens_set
        tokens_set.remove(token_id);
        // Nếu xoá token xong tokens_set của account rỗng -> Xoá luôn account_id khỏi tokens_per_owner
        // Ngược lại -> Cập nhật list tokens_per_owner
        if tokens_set.is_empty() {
            self.tokens_per_owner.remove(account_id);
        } else {
            self.tokens_per_owner.insert(account_id, &tokens_set);
        }
    }

    // Return data token cũ trước khi thực hiện transfer
    /**
     * - Kiểm tra token_id có tồn tại không?
     * - sender_id có phải là owner của token hay không?
     * - sender_id và receiver_id trùng nhau (gửi cho chính mình) không?
     * - Xoá token khỏi owner cũ
     * - Thêm token cho receiver_id
     */
    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &TokenId,
        memo: Option<String>,
    ) -> Token {
        // Kiểm tra token_id có tồn tại không?
        let token = self.tokens_by_id.get(token_id).expect("Not found token");
        // sender_id có phải là owner của token hay không?
        if sender_id != &token.owner_id {
            env::panic("Sender must be the token's owner".as_bytes());
        };
        // sender_id và receiver_id trùng nhau (gửi cho chính mình) không?
        assert_ne!(&token.owner_id, receiver_id, "The token owner and the receiver should be different");

        // Xoá token khỏi owner cũ
        self.internal_remove_token_from_owner(&token_id, &token.owner_id);
        // Thêm token cho receiver_id
        self.internal_add_token_to_owner(&token_id, receiver_id);

        let new_token = Token {
            owner_id: receiver_id.clone(),
        };

        // Thêm token mới vào list tất cả tokens
        self.tokens_by_id.insert(token_id, &new_token);

        // Nếu có memo thì in ra memo
        if let Some(memo) = memo {
            log!("Memo: {}", memo);
        }

        // Return token cũ
        token
    }
}
