use crate::*;

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    // get the default hash algorithm
    let mut hash = CryptoHash::default();
    // We hash the account Id and return it as
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

pub(crate) fn refund_deposit(storage_used: u64) {
    // Tính lượng tiền cần nạp để cover storage
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    // Nếu người dùng deposit lượng tiền ít hơn lượng cần thiết để lưu data -> Báo lỗi
    assert!(
        attached_deposit >= required_cost,
        "Must attach {} yoctoNear to cover storage",
        required_cost
    );

    let refund_amount = attached_deposit - required_cost;

    // Thực hiện refund
    if refund_amount > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund_amount);
    }
}

pub(crate) fn assert_one_yocto() {
    assert_eq!(env::attached_deposit(), 1, "Required attached deposit of exact 1 yoctoNear");
}
