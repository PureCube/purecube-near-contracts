# PureBlock Near Contracts

Smart contracts for PureBlock platform at Near Protocol.

## Runner

Near Runner play-to-earn in-game NFT. 

### Implementation details

* follow all [NonFungibleToken standards](https://nomicon.io/Standards/Tokens/NonFungibleToken/)
* `new_default_meta` initializes with
  * `owner_id` contract owner account
  * `treasury_id` receives funds from mint and can mint until `max_supply` is reached
  * `max_supply` of tokens
  * `base_uri` endpoint of centralized gateway which stores media, off-chain attributes and uses revealing system
  * `mint_price` of new token
  * `mint_start` timestamp milliseconds when anyone can mint new token by `mint_price`
  * `mint_end` timestamp milliseconds when anyone cant mint
  * `perpetual_royalties` up to 6 trade fee receivers
* `nft_mint` require `receiver_id` only
* `set_meta` can change `name`, `base_uri`, `icon` by `owner_id`

## Marketplace

Simple NFT marketplace contract

## Develop Quick-Start

### Explore Near NFT contracts
* *[near-examples](https://github.com/near-examples)* **[NFT](https://github.com/near-examples/NFT)** built on *[near-contract-standards](https://github.com/near/near-sdk-rs/tree/master/near-contract-standards)* :: *[non_fungible_token](https://github.com/near/near-sdk-rs/tree/master/near-contract-standards/src/non_fungible_token)* implementation and [Master NFTs on NEAR](https://docs.near.org/tutorials/nfts/introduction#) *[nft-tutorial](https://github.com/near-examples/nft-tutorial)* **[nft-contract](https://github.com/near-examples/nft-tutorial/tree/main/nft-contract)**, **[nft-series](https://github.com/near-examples/nft-tutorial/tree/main/nft-series)** and **[market-contract](https://github.com/near-examples/nft-tutorial/tree/main/market-contract)** built from scratch without it. Last option is based for our NFT contracts.
* [paras.id](https://paras.id): [paras-nft-contract](https://github.com/ParasHQ/paras-nft-contract), [integration requirements](https://docs.paras.id/nft-smart-contract-integration)
* [mintbase.io](https://mintbase.io): [mintbase-core](https://github.com/Mintbase/mintbase-core), [list a token](https://docs.mintbase.io/dev/smart-contracts/core-addresses/marketplace-2.0) 

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
MAIN_ACCOUNT=your-account.testnet
NFT_CONTRACT_ID=runner.your-account.testnet
BASE_URI=https://gateway.pureblock.io/runner-testnet
echo $NFT_CONTRACT_ID
echo $MAIN_ACCOUNT
echo $BASE_URI
near create-account $NFT_CONTRACT_ID --masterAccount $MAIN_ACCOUNT --initialBalance 10
near deploy --accountId $NFT_CONTRACT_ID --wasmFile out/main.wasm
near call $NFT_CONTRACT_ID new_default_meta '{"owner_id": "'$NFT_CONTRACT_ID'","treasury_id": "'$MAIN_ACCOUNT'", "max_supply": "100", 
"base_uri": "'$BASE_URI'", "mint_price": "5000000000000000000000000","mint_start": "1", "mint_end": "1690000000000000000", "perpetual_royalties": {"'$MAIN_ACCOUNT'": 200}}' --accountId $NFT_CONTRACT_ID
near view $NFT_CONTRACT_ID nft_metadata
near call $NFT_CONTRACT_ID nft_mint '{"receiver_id": "'$MAIN_ACCOUNT'"}' --accountId $MAIN_ACCOUNT --amount 5.1
near view $NFT_CONTRACT_ID nft_token '{"token_id": "0"}'
```

### Transfering NFTs

```bash=
MAIN_ACCOUNT_2=your-second-wallet-account.testnet
echo $NFT_CONTRACT_ID
echo $MAIN_ACCOUNT
echo $MAIN_ACCOUNT_2
near call $NFT_CONTRACT_ID nft_transfer '{"receiver_id": "$MAIN_ACCOUNT_2", "token_id": "0", "memo": "Go Team :)"}' --accountId $MAIN_ACCOUNT --depositYocto 1
```

### Change contract metadata
```bash=
echo $NFT_CONTRACT_ID
echo $BASE_URI
near call $NFT_CONTRACT_ID set_meta '{"name": "Near Runner", "base_uri": "'$BASE_URI'", "icon": ""}' --accountId $NFT_CONTRACT_ID --depositYocto 1
near view $NFT_CONTRACT_ID nft_metadata
```

### List NFT on marketplaces

#### Mintbase
```
echo $NFT_CONTRACT_ID
echo $MAIN_ACCOUNT
near call market-v2-beta.mintspace2.testnet deposit_storage '{}' --deposit 0.01 --accountId $MAIN_ACCOUNT
near call $NFT_CONTRACT_ID nft_approve '{"account_id": "market-v2-beta.mintspace2.testnet", "token_id": "0", "msg": "{\"price\":\"1000000000000000000000000\"}"}' --depositYocto 450000000000000000000 --accountId $MAIN_ACCOUNT
```
