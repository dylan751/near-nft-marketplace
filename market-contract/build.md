# How to build and test this contract

1. Build nft_market_contract -> deploy out/market-contract.wasm to nft_market_contract (`nft-market.duongnh.testnet`)

   ```
   - ./build.sh
   - cd ..
   - near create-account nft-market.duongnh.testnet --masterAccount duongnh.testnet --initialBalance 20
   - near deploy --wasmFile out/market-contract.wasm --accountId nft-market.duongnh.testnet --initFunction new --initArgs '{"owner_id": "duongnh.testnet"}'
   ```

2. View total supply in `nft-market.duongnh.testnet`

   ```
   near view nft-market.duongnh.testnet get_supply_sales
   ```

3. Deposit into Market Contract to cover storage

   ```
   near call nft-market.duongnh.testnet storage_deposit '{"account_id": "duongnh.testnet"}' --accountId duongnh.testnet --deposit 0.1
   ```

4. Call approve to transfer token (`duongnh.testnet` gives approve to `nft-market.duongnh.testnet` with `price = 1 NEAR`)
    ```
    near call nft.duongnh.testnet nft_approve '{"token_id": "ZNG_NFT#02", "account_id": "nft-market.duongnh.testnet", "msg": "{\"sale_conditions\": \"1000000000000000000000000\"}"}' --accountId duongnh.testnet --deposit 0.01
    ```

5. Get Sales information on Market
    ```
    near view nft-market.duongnh.testnet get_sales '{"from_index": "0", "limit": 10}'
    ```