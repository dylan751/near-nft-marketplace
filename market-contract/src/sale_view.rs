/**
 * Để hiển thị trạng thái sale của Marpketlace
*/
use crate::*;

#[near_bindgen]
impl Contract {
    // Lấy tổng số sale đang đăng bán trên Market
    pub fn get_supply_sales(&self) -> U128 {
        U128(self.sales.len() as u128)
    }

    // Lấy tổng số sale đang đăng bán theo owner_id
    pub fn get_supply_by_owner_id(&self, account_id: AccountId) -> U128 {
        let sales_by_owner_id = self.by_owner_id.get(&account_id);
        if let Some(sales_by_owner_id) = sales_by_owner_id {
            U128(sales_by_owner_id.len() as u128)
        } else {
            U128(0)
        }
    }

    // Lấy tổng số sale đang đăng bán theo contract_id
    pub fn get_supply_by_contract_id(&self, contract_id: NFTContractId) -> U128 {
        let tokens_by_contract_id = self.by_contract_id.get(&contract_id);
        if let Some(tokens_by_contract_id) = tokens_by_contract_id {
            U128(tokens_by_contract_id.len() as u128)
        } else {
            U128(0)
        }
    }

    // Lấy tất cả thông tin của sale hiện tại (có pagination)
    pub fn get_sales(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Sale> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.sales
            .values()
            .skip(start as usize)
            .take(limit.unwrap_or(0) as usize)
            .collect()
    }

    // Lấy tất cả thông tin sale của owner_id (có pagination)
    pub fn get_sale_by_owner_id(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Sale> {
        // Lấy tất cả token của account_id
        let by_owner_id = self.by_owner_id.get(&account_id);
        let contract_token_ids = if let Some(by_owner_id) = by_owner_id {
            by_owner_id
        } else {
            return vec![];
        };

        // Lấy danh sách thông tin các token đang sale
        let start = u128::from(from_index.unwrap_or(U128(0)));

        contract_token_ids
            .as_vector()
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(0) as usize)
            .map(|contract_token_ids| self.sales.get(&contract_token_ids).unwrap())
            .collect()
    }

    // Lấy tất cả thông tin sale của contract_id (có pagination)
    pub fn get_sale_by_contract_id(
        &self,
        contract_id: NFTContractId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Sale> {
        // Lấy tất cả token của contract_id
        let tokens_by_contract_id = self.by_contract_id.get(&contract_id);

        let token_ids = if let Some(tokens_by_contract_id) = tokens_by_contract_id {
            tokens_by_contract_id
        } else {
            return vec![];
        };

        let start = u128::from(from_index.unwrap_or(U128(0)));
        token_ids
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(0) as usize)
            .map(|token_id| {
                self.sales
                    // format để chuyển từ dạng TokenId sang ContractAndTokenId: <contract_id>.<token_id>
                    // TokenId: ZNG_NFT#01
                    // ContractTokenId: nft.duongnh.testnet.ZNG_NFT#01
                    .get(&format!("{}{}{}", contract_id, ".", token_id))
                    .unwrap()
            })
            .collect()
    }
}
