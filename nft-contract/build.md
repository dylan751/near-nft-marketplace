# How to build and test this contract

1. Create nft_contract_id -> deploy out/nft-contract.wasm to nft_contract_id (`nft.duongnh.testnet`)

   ```
   - ./build.sh
   - cd ..
   - near create-account nft.duongnh.testnet --masterAccount duongnh.testnet --initialBalance 20
   - near deploy --wasmFile out/nft-contract.wasm --accountId nft.duongnh.testnet --initFunction new_default_metadata --initArgs '{"owner_id": "duongnh.testnet"}'
   ```

2. View total supply in `nft.duongnh.testnet`

   ```
   near view nft.duongnh.testnet nft_total_supply
   ```

3. View total supply of an account `duongnh.testnet`

   ```
   near view nft.duongnh.testnet nft_supply_for_owner '{"account_id": "duongnh.testnet"}'
   ```

4. Mint an NFT _(Note: token_id must be unique)_

   ```
   near call nft.duongnh.testnet nft_mint '{"token_id": "ZNG_NFT#03", "receiver_id": "duongnh.testnet", "metadata": {"title": "NEAR LOGO", "description": "NEAR LOGO", "media": "https://bafkreibhsxpr4qbjqure75n6q6ywulozmb6e2tnedloq6v5em24f6nhmgm.ipfs.dweb.link/"}}' --deposit 0.1 --accountId duongnh.testnet
   ```

5. View the token we just minted

   ```
   near view nft.duongnh.testnet nft_token '{"token_id": "ZNG_NFT#03"}'
   ```

6. View total supply again in `nft.duongnh.testnet`

   ```
   near view nft.duongnh.testnet nft_total_supply
   ```

7. Transfer NFT from `duongnh.tesnet` to `zuongnh.testnet`

   ```
   near call nft.duongnh.testnet nft_transfer '{"receiver_id": "zuongnh.testnet", "token_id": "ZNG_NFT#03", "approval_id": 0}' --accountId duongnh.testnet --depositYocto 1
   ```

8. `zuongnh.testnet` add approval for `duongnh.testnet` to transfer his token

   ```
   near call nft.duongnh.testnet nft_approve '{"token_id": "ZNG_NFT#03", "account_id": "duongnh.testnet"}' --deposit 0.01 --accountId zuongnh.testnet
   ```

9. View the token we just add approval

   ```
   near view nft.duongnh.testnet nft_token '{"token_id": "ZNG_NFT#03"}'
   ```

10. Use `duongnh.testnet` to transfer token back to `duongnh.testnet` (although token's owner is `zuongnh.testnet`, but since `duongnh.testnet` has been approved to transfer the NFT so it can transfer the NFT)

   ```
   near call nft.duongnh.testnet nft_transfer '{"receiver_id": "duongnh.testnet", "token_id": "ZNG_NFT#02", "approval_id": 0}' --accountId duongnh.testnet --depositYocto 1
   ```
