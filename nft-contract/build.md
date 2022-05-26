# How to build and test this contract

2. Create nft_contract_id -> deploy out/nft-contract.wasm to nft_contract_id (`nft.duongnh.testnet`)

   ```
   - ./build.sh
   - cd ..
   - near create-account nft.duongnh.testnet --masterAccount duongnh.testnet --initialBalance 20
   - near deploy --wasmFile out/nft-contract.wasm --accountId nft.duongnh.testnet --initFunction new_default_metadata --initArgs '{"owner_id": "duongnh.testnet"}'
   ```

3. View total supply in `nft.duongnh.testnet`

   ```
   near view nft.duongnh.testnet nft_total_supply
   ```

4. Mint an NFT _(Note: token_id must be unique)_

   ```
   near call nft.duongnh.testnet nft_mint '{"token_id": "ZNG_NFT#01", "receiver_id": "duongnh.testnet", "metadata": {"title": "NEAR LOGO", "description": "NEAR LOGO", "media": "https://bafkreibhsxpr4qbjqure75n6q6ywulozmb6e2tnedloq6v5em24f6nhmgm.ipfs.dweb.link/"}}' --deposit 0.1 --accountId duongnh.testnet
   ```

5. View the token we just minted

   ```
   near view nft.duongnh.testnet nft_token '{"token_id": "ZNG_NFT#01"}'
   ```

6. View total supply again in `nft.duongnh.testnet`

   ```
   near view nft.duongnh.testnet nft_total_supply
   ```

7. Transfer NFT from `duongnh.tesnet` to `zuongnh.testnet`

   ```
   near call nft.duongnh.testnet nft_transfer '{"receiver_id": "zuongnh.testnet", "token_id": "ZNG_NFT#01", "approval_id": 0}' --accountId duongnh.testnet --depositYocto 1
   ```

8. `zuongnh.testnet` add approval for `duongnh.testnet` to transfer his token

   ```
   near call nft.duongnh.testnet nft_approve '{"token_id": "ZNG_NFT#01", "account_id": "duongnh.testnet"}' --deposit 0.01 --accountId zuongnh.testnet
   ```

5. View the token we just add approval

   ```
   near view nft.duongnh.testnet nft_token '{"token_id": "ZNG_NFT#01"}'
   ```

9. Use `duongnh.testnet` to transfer token back to `duongnh.testnet` (although token's owner is `zuongnh.testnet`, but since `duongnh.testnet` has been approved to transfer the NFT so it can transfer the NFT)

   ```
   near call nft.duongnh.testnet nft_transfer '{"receiver_id": "duongnh.testnet", "token_id": "ZNG_NFT#02", "approval_id": 0}' --accountId duongnh.testnet --depositYocto 1
   ```

10. Transfer: (call contract `ft_transfer_call` in `ft.duongnh.testnet`)

    ```
    near call ft.duongnh.testnet ft_transfer_call '{"receiver_id": "staking.duongnh.testnet", "amount": "1000000000000000000000000", "msg": ""}' --accountId duongnh.testnet --depositYocto 1 --gas 60000000000000
    ```

11. Check account info again: `duongnh.testnet`

    ```
    near view staking.duongnh.testnet get_account_info '{"account_id": "duongnh.testnet"}'
    ```

12. Harvest reward:

    ```
    near call staking.duongnh.testnet harvest --accountId duongnh.testnet --depositYocto 1 --gas 60000000000000
    ```

13. Unstake:

    ```
    near call staking.duongnh.testnet unstake '{"amount": "1000000000000000000"}' --accountId duongnh.testnet --depositYocto 1
    ```

14. Withdraw (User can only withdraw after 1 epoch since unstake (~12 hours))

    ```
    near call staking.duongnh.testnet withdraw --accountId duongnh.testnet --depositYocto 1 --gas 300000000000000
    ```

15. Run simulation tests:

    ```

    ```
