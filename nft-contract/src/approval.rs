/**
 * Quản lý các account approved thông qua các function
 * Cập nhật lại transfer để cho phép các tài khoản trong list account_approved sẽ được phép transfer token thay owner
 */
use crate::*;

const GAS_FOR_NFT_APPROVE: Gas = 10_000_000_000_000;
const NO_DEPOSIT: Balance = 0;

pub trait NonFungibleTokenApproval {
    // Cho phép account khác (Marketplace) quyền chuyển token của mình cho người khác
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>);
    // Check xem account đã có quyền chuyển Token chưa
    // Nếu approve account_id hợp lệ -> return true, else return false
    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool;
    // Xoá quyền của 1 account đối với 1 token
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId);
    // Xoá toàn bộ các contract đã aprroved khỏi 1 token nào đó
    // (Xoá toàn bộ quyền transfer token của tất cả contract đối vói 1 token nào đó)
    fn nft_revoke_all(&mut self, token_id: TokenId);
}

#[ext_contract(ext_non_fungible_token_approval_receiver)]
pub trait NonFungibleTokenApprovalReceiver {
    // Market Contract: A
    // NFT Contract: B
    // Khi Contract A call approve trên Contract B, bên B sẽ gọi lại hàm
    // nft_on_approve trên Contract A để phía A xử lý ngược lại (đăng bán, cập nhật thông tin trạng thái, ...)
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

#[near_bindgen]
impl NonFungibleTokenApproval for Contract {
    // Thêm quyền chuyển token cho account_id
    // Bổ sung account_id vào list approved_account_ids của Token
    // Note: Vì function này sẽ làm tăng data trong Contract -> Thêm payable để user deposit thêm
    // Account ID => market contract id
    #[payable]
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>) {
        assert_at_least_one_yocto();

        // Kiểm tra xem token có tồn tại hay không
        let mut token = self.tokens_by_id.get(&token_id).expect("Not found token");

        // Check xem sender có phải token owner không
        // Chỉ owner mới có quyền approved cho account khác
        assert_eq!(
            &env::predecessor_account_id(),
            &token.owner_id,
            "Predecessor must be the token owner"
        );

        // Thực hiện approve
        let approval_id = token.next_approval_id;
        // Check xem account này đã tồn tại trong list approved_account_ids chưa
        // Add account vào list các tài khoản có thể transfer Token này
        let is_new_approval = token
            .approved_account_ids
            .insert(account_id.clone(), approval_id)
            .is_none();

        // Nếu approve cho account mới -> Tăng dung lượng data -> tính phí cho user
        let storage_used = if is_new_approval {
            bytes_for_approved_account_id(&account_id)
        } else {
            0
        };

        token.next_approval_id += 1;
        self.tokens_by_id.insert(&token_id, &token);

        // Refund nếu user nạp vào thừa phí lưu trữ
        refund_deposit(storage_used);

        // Nếu có gắn msg -> Thực hiện Cross Contract Call sang market contract
        // msg chứa thông tin: giá, hành động, hàm, ...
        if let Some(msg) = msg {
            ext_non_fungible_token_approval_receiver::nft_on_approve(
                token_id,
                token.owner_id,
                approval_id,
                msg,
                &account_id,
                NO_DEPOSIT,
                env::prepaid_gas() - GAS_FOR_NFT_APPROVE,
            )
            .as_return();
        }
    }

    // Kiểm tra account có tồn tại trong list approve ko
    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        let token = self.tokens_by_id.get(&token_id).expect("Token not found");
        let approval = token.approved_account_ids.get(&approved_account_id);

        // Nếu tồn tại account trong list approved_account_ids -> Check tiếp xem approval_id có đúng ko
        if let Some(approval) = approval {
            if approval == &approval_id.unwrap() {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // Note: Khi xoá 1 account khỏi approved_list_ids -> Refund phí lưu trữ data mà user đã trả trước đó
    #[payable]
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
        assert_one_yocto();

        let mut token = self.tokens_by_id.get(&token_id).expect("Not found token");
        let sender_id = env::predecessor_account_id();
        // Check xem người gọi hàm revoke() có phải owner của token hay không
        assert_eq!(
            &sender_id, &token.owner_id,
            "Only owner of the NFT can call revoke function"
        );

        // Nếu xoá quyền thành công
        if token.approved_account_ids.remove(&account_id).is_some() {
            // Refund lại số tiền đã deposit để lưu trữ data của user
            refund_approved_account_ids_iter(sender_id, [account_id].iter());
            // Cập nhật lại danh sách tokens
            self.tokens_by_id.insert(&token_id, &token);
        }
    }

    #[payable]
    fn nft_revoke_all(&mut self, token_id: TokenId) {
        assert_one_yocto();

        let mut token = self.tokens_by_id.get(&token_id).expect("Not found token");
        let sender_id = env::predecessor_account_id();
        // Check xem người gọi hàm revoke() có phải owner của token hay không
        assert_eq!(
            &sender_id, &token.owner_id,
            "Only owner of the NFT can call revoke function"
        );

        if !token.approved_account_ids.is_empty() {
            // Refund lại số tiền mọi người đã deposit khi gọi hàm revoke_all()
            refund_approved_account_ids(sender_id, &token.approved_account_ids);
            // Xoá toàn bộ list account đã approved cho token
            token.approved_account_ids.clear();
            // Cập nhật lại danh sách tokens
            self.tokens_by_id.insert(&token_id, &token);
        }
    }
}
