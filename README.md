# PureBlock Near Contracts

Smart contracts for PureBlock platform at Near Protocol.

## Runner

Near Runner play-to-earn in-game NFT. 

### Requirements

* [x] Follow all [NonFungibleToken standards](https://nomicon.io/Standards/Tokens/NonFungibleToken/)
* [x] Initialize with
  * `owner_id` contract owner account
  * `treasury_id` receives funds from mint and can mint until `max_supply` is reached
  * `max_supply` of tokens
  * `base_uri` endpoint of centralized gateway which stores media, off-chain attributes and uses revealing system
  * `mint_price` of new token
  * `mint_start` timestamp milliseconds when anyone can mint new token by `mint_price`
  * `mint_end` timestamp milliseconds when anyone cant mint
  * `perpetual_royalties` up to 6 trade fee receivers
* [ ] Pass [Paras](https://paras.id) [integration requirements](https://docs.paras.id/nft-smart-contract-integration)


## Marketplace

Simple NFT marketplace contract

## Develop Quick-Start

### Explore Near NFT contracts
* *[near-examples](https://github.com/near-examples)* **[NFT](https://github.com/near-examples/NFT)** built on *[near-contract-standards](https://github.com/near/near-sdk-rs/tree/master/near-contract-standards)* :: *[non_fungible_token](https://github.com/near/near-sdk-rs/tree/master/near-contract-standards/src/non_fungible_token)* implementation and [Master NFTs on NEAR](https://docs.near.org/tutorials/nfts/introduction#) *[nft-tutorial](https://github.com/near-examples/nft-tutorial)* **[nft-contract](https://github.com/near-examples/nft-tutorial/tree/main/nft-contract)**, **[nft-series](https://github.com/near-examples/nft-tutorial/tree/main/nft-series)** and **[market-contract](https://github.com/near-examples/nft-tutorial/tree/main/market-contract)** built from scratch without it. Last option is based for our NFT contracts.
* [paras-nft-contract](https://github.com/ParasHQ/paras-nft-contract)


### Prerequisites

* [NEAR Wallet Account](wallet.testnet.near.org)
* [Rust Toolchain](https://docs.near.org/develop/prerequisites)
* [NEAR-CLI](https://docs.near.org/tools/near-cli#setup)
* [yarn](https://classic.yarnpkg.com/en/docs/install#mac-stable)

### Build
```=bash
yarn build
yarn test
```

### Mint An NFT

```=bash
near login
near create-account nft-example.your-account.testnet --masterAccount your-account.testnet --initialBalance 10
NFT_CONTRACT_ID=nft-example.your-account.testnet
MAIN_ACCOUNT=your-account.testnet
echo $NFT_CONTRACT_ID
echo $MAIN_ACCOUNT
near deploy --accountId $NFT_CONTRACT_ID --wasmFile out/main.wasm
near call $NFT_CONTRACT_ID new_default_meta '{"owner_id": "'$NFT_CONTRACT_ID'"}' --accountId $NFT_CONTRACT_ID
near view $NFT_CONTRACT_ID nft_metadata
near call $NFT_CONTRACT_ID nft_mint '{"token_id": "token-1", "metadata": {"title": "My Non Fungible Team Token", "description": "The Team Most Certainly Goes :)", "media": "https://bafybeiftczwrtyr3k7a2k4vutd3amkwsmaqyhrdzlhvpt33dyjivufqusq.ipfs.dweb.link/goteam-gif.gif"}, "receiver_id": "'$MAIN_ACCOUNT'"}' --accountId $MAIN_ACCOUNT --amount 0.1
near call $NFT_CONTRACT_ID nft_mint '{"token_id": "token-1", "metadata": {"title": "My Non Fungible Team Token", "description": "The Team Most Certainly Goes :)", "media": "https://bafybeiftczwrtyr3k7a2k4vutd3amkwsmaqyhrdzlhvpt33dyjivufqusq.ipfs.dweb.link/goteam-gif.gif"}, "receiver_id": "'$MAIN_ACCOUNT'"}' --accountId $MAIN_ACCOUNT --amount 0.1
```

### View NFT Information

```bash=
near view $NFT_CONTRACT_ID nft_token '{"token_id": "token-1"}'
```

### Transfering NFTs

```bash=
MAIN_ACCOUNT_2=your-second-wallet-account.testnet
echo $NFT_CONTRACT_ID
echo $MAIN_ACCOUNT
echo $MAIN_ACCOUNT_2
near call $NFT_CONTRACT_ID nft_transfer '{"receiver_id": "$MAIN_ACCOUNT_2", "token_id": "token-1", "memo": "Go Team :)"}' --accountId $MAIN_ACCOUNT --depositYocto 1
```
