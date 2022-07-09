use crate::*;
use near_sdk::promise_result_as_success;
use std::collections::HashMap;

// GAS constants to attach to calls
const GAS_FOR_ROYALTIES: Gas = 115_000_000_000_000;
const GAS_FOR_NFT_TRANSFER: Gas = 15_000_000_000_000;

// Constant useds to attch 0 NEAR to a call
const NO_DEPOSIT: Balance = 0;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

#[ext_contract(ext_nft_contract)]
pub trait NFTContract {
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: String,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout;
}

#[ext_contract(ext_self)]
pub trait MarketContract {
    fn resolve_purchase(&mut self, buyer_id: AccountId, price: U128) -> Promise;
    fn ft_resolve_purchase(&mut self, buyer_id: AccountId, price: SalePrice) -> Promise;
}

#[near_bindgen]
impl Contract {
    // Xoá sale
    #[payable]
    pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: TokenId) {
        assert_one_yocto();

        // Xoá sale
        let sale = self.internal_remove_sale(nft_contract_id, token_id);

        assert_eq!(
            env::predecessor_account_id(),
            sale.owner_id,
            "Must be owner id"
        );
    }

    // Update giá của Sale
    pub fn update_price(&mut self, nft_contract_id: AccountId, token_id: TokenId, price: SalePrice) {
        assert_one_yocto();

        let contract_and_token_id =
            format!("{}{}{}", nft_contract_id.clone(), ".", token_id.clone());

        let mut sale = self
            .sales
            .get(&contract_and_token_id)
            .expect("Not found sale");

        // Check xem có phải người update giá là chủ của Sale ko
        assert_eq!(
            env::predecessor_account_id(),
            sale.owner_id,
            "Must be sale owner"
        );

        sale.sale_conditions = price;

        // Update lại thông tin
        self.sales.insert(&contract_and_token_id, &sale);
    }

    // Cho phép user mua nft
    #[payable]
    pub fn offer(&mut self, nft_contract_id: AccountId, token_id: TokenId) {
        let deposit = env::attached_deposit();
        assert!(deposit > 0, "Attached deposit must be greater than 0");

        let contract_and_token_id =
            format!("{}{}{}", nft_contract_id.clone(), ".", token_id.clone());

        let sale = self
            .sales
            .get(&contract_and_token_id)
            .expect("Not found sale");

        let buyer_id = env::predecessor_account_id();
        // Buyer và owner của NFT phải khác nhau (không thể tự mua NFT của chính mình được)
        assert_ne!(buyer_id, sale.owner_id, "Can not bid on your own sale");

        let price = sale.sale_conditions.amount.0;
        assert!(
            deposit >= price,
            "Attached deposit must be grater than or equal current price: {}",
            price
        );

        self.process_purchase(nft_contract_id, token_id, U128(deposit), buyer_id);
    }

    #[private]
    pub fn process_purchase(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
        price: U128,
        buyer_id: AccountId,
    ) -> Promise {
        // Mua hàng -> Xoá sản phẩm đi
        let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());

        // Cross-contract Call
        ext_nft_contract::nft_transfer_payout(
            buyer_id.clone(),
            token_id,
            sale.approval_id,
            "Payout from market contract".to_string(),
            price,
            10,
            &nft_contract_id,
            1,
            GAS_FOR_NFT_TRANSFER,
        )
        .then(ext_self::resolve_purchase(
            buyer_id,
            price,
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_ROYALTIES,
        ))
    }

    // Chuyển tiền bản quyền cho các payouts ở trong payout object
    pub fn resolve_purchase(&mut self, buyer_id: AccountId, price: U128) -> U128 {
        let payout_option = promise_result_as_success().and_then(|value| {
            let payout_object =
                near_sdk::serde_json::from_slice::<Payout>(&value).expect("Invalid payout object");

            // Giới hạn xử lý max 10 account
            if payout_object.payout.len() > 10 || payout_object.payout.is_empty() {
                env::log("Cannot have more than 10 royalties".as_bytes());
                None
            } else {
                let mut remainder = price.0;

                for &value in payout_object.payout.values() {
                    remainder = remainder.checked_sub(value.0)?;
                }

                if remainder == 0 || remainder == 1 {
                    Some(payout_object.payout)
                } else {
                    None
                }
            }
        });

        // Transfer payout
        let payout = if let Some(payout_option) = payout_option {
            payout_option
        } else {
            // Nếu ko có payout -> ko phải trả tiền
            // Toàn bộ tiền của buyer sẽ được chuyển lại
            Promise::new(buyer_id).transfer(u128::from(price));
            return price;
        };

        for (receiver_id, amount) in payout {
            Promise::new(receiver_id).transfer(u128::from(amount));
        };

        price
    }
}
