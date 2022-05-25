use crate::*;

const GAS_FOR_RESOLVE_TRANSFER: Gas = 10_000_000_000_000;
const GAS_FOR_NFT_TRANSFER_CALL: Gas = 25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;
const NO_DEPOSIT: Balance = 0;

// Các trait, interface, function theo chuẩn NEP-171 của NEAR - Core Functionality
// Xem thêm tại: https://nomicon.io/Standards/Tokens/NonFungibleToken/Core
pub trait NonFungibleTokenCore {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: Option<String>,
    );

    // Return true nếu transfer NFT được thực hiện thành công
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        memo: Option<String>,
        approval_id: u64,
        msg: String,
    ) -> PromiseOrValue<bool>;
}

#[ext_contract(ext_non_fungible_token_receiver)] // Macro cross contract call
trait NonFungibleTokenReceiver {
    // Method này được lưu trên Contract B, A thực hiện Cross Contract Call nft_on_transfer
    // Return true nếu như NFT cần được rollback lại cho owner cũ (transfer lỗi)
    // Dùng để check NFT transfer đã thành công hay cần rollback (thất bại)
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolver {
    // Nếu Contract B yêu cầu rollback lại cho owner cũ -> A sẽ rollback lại data trong nft_resolve_transfer
    // Chứa logic để thực hiện rollback
    fn nft_resolve_transfer(
        &mut self,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: HashMap<AccountId, u64>,
    ) -> bool;
}

trait NonFungibleTokenResolver {
    fn nft_resolve_transfer(
        &mut self,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: HashMap<AccountId, u64>,
    ) -> bool;
}

// Triển khai code cho những interface trên
#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    // Yêu cầu deposit 1 yoctoNear để bảo mật cho user
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let previous_token =
            self.internal_transfer(&sender_id, &receiver_id, &token_id, Some(approval_id), memo);

        // Refund nếu deposit thừa
        refund_approved_account_ids(sender_id, &previous_token.approved_account_ids);
    }

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        memo: Option<String>,
        approval_id: u64,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let previous_token =
            self.internal_transfer(&sender_id, &receiver_id, &token_id, Some(approval_id), memo);

        // Thực hiện Cross Contract Call sang Contract của người nhận
        // -> Gọi hàm nft_on_transfer
        ext_non_fungible_token_receiver::nft_on_transfer(
            sender_id.clone(),
            previous_token.owner_id.clone(),
            token_id.clone(),
            msg,
            &receiver_id,
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_NFT_TRANSFER_CALL,
        )
        .then(ext_self::nft_resolve_transfer(
            previous_token.owner_id,
            receiver_id,
            token_id,
            previous_token.approved_account_ids,
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }

    // Vì nft_transfer và nft_transfer_call đều sử dụng logic transfer token giống nhau -> code 1 hàm vào internal.rs
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    // Xử lý call back của nft_on_transfer khi contract nhận gọi lại
    fn nft_resolve_transfer(
        &mut self,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: HashMap<AccountId, u64>,
    ) -> bool {
        if let PromiseResult::Successful(value) = env::promise_result(0) {
            // Thành công, chỉ có 1 promise
            if let Ok(is_rollback_token) = near_sdk::serde_json::from_slice::<bool>(&value) {
                return is_rollback_token;
            }
        }

        // Xử lý các case không thể rollback lại được
        let mut token = if let Some(token) = self.tokens_by_id.get(&token_id) {
            // Nếu người nhận ko phải là owner -> Không thực hiện được -> rollback
            if token.owner_id != receiver_id {
                refund_approved_account_ids(owner_id, &approved_account_ids);
                return true;
            }
            token
        } else {
            // Nếu không tìm thấy token -> Không thực hiện được -> rollback
            refund_approved_account_ids(owner_id, &approved_account_ids);
            return true;
        };

        // Xử lý các case rollback được
        log!(
            "Rollback {} from @{} to @{}",
            token_id,
            receiver_id,
            owner_id
        );

        self.internal_remove_token_from_owner(&token_id, &receiver_id); // Xoá token của người vừa nhận
        self.internal_add_token_to_owner(&token_id, &owner_id); // Trả lại token cho owner cũ

        // Lấy lại các giá trị của token
        token.owner_id = owner_id;

        refund_approved_account_ids(receiver_id, &token.approved_account_ids);
        token.approved_account_ids = approved_account_ids;
        
        self.tokens_by_id.insert(&token_id, &token);

        false // Cho front-end biết là giao dịch thất bại -> Rollback toàn bộ data
    }
}
