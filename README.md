# Singleton SherpaX 
Based on Substrate framework, 
Singleton SherpaX Chain is a blockchain adopting `aura + grandpa` consensus algorithm.


How to use/test locally

## 1. Supported dependencies

It should at least work until the following commits

-   polkadot.js.org (apps v0.98.2-106)
-   Polkadot release-v0.9.11 @ `eb9f107e3a04687dcf80111563f2bcea7d5b15d3`
-   Substrate polkadot-v0.9.11  @ `57346f6b24875f8935280dba51fa8ab0a9ba1e39`

## 2. Local Test

### 2.1 quick start
```bash
 ./target/release/sherpax --dev --tmp --rpc-port 8546 --rpc-cors=all -levm=trace
```

### 2.2 full start

#### 2.2.1 alice

```bash
./target/release/sherpax \
    --chain=local \
    -d ./data/alice \
    --alice \
    --ws-port 9944 \
    --port 30331
```

#### 2.2.2 bob

```bash
./target/release/sherpax \
    --chain=local \
    -d ./data/bob \
    --bob \
    --ws-port 9945 \
    --port 30332
```

#### 2.2.3 full

```bash
 ./target/release/sherpax \
    --chain=local \
    -d ./data/sherpax-full \
    --execution=wasm \
    --state-cache-size=0 \
    --prometheus-external \
    --pruning=archive \
    --rpc-cors=all \
    --rpc-external \
    --rpc-port 8546 \
    --ws-external \
    --ws-port 9977 \
    --ws-max-connections 10000 \
    --port 30333 \
    -levm=trace
```

## 3. Evm support

### 3.1 metamask (for account)

```txt
Network Name: sherpax
New RPC URL: http://127.0.0.1:8546
Chain ID: 1506
Currency Symbol: KSX
Block Explorer URL:
```

Refer [Connect MetaMask to Moonbase Alpha](https://docs.moonbeam.network/getting-started/moonbase/metamask/)

### 3.2 Remix (for contract)

Refer [Interacting with Moonbeam Using Remix](https://docs.moonbeam.network/getting-started/local-node/using-remix/)

### 3.3  Ethereum apis
- [Ethereum JSON-RPC.postman_collection](https://github.com/chainx-org/SherpaX/blob/singleton/develop_docs/Ethereum-JSON-RPC.postman_collection.json)
- [Ethereum RPC Support](https://github.com/PureStake/moonbeam-docs-cn/blob/master/builders/get-started/eth-compare/rpc-support.md)
- [QuickNode Ethereum RPC](https://www.quicknode.com/docs)
- [Ethereum JSON-RPC Wiki](https://eth.wiki/json-rpc/API#)

### 3.4 frointier账户体系
[frontier是如何管理substrate账户和ethereum账户的](https://github.com/chainx-org/chainx-technical-archive/blob/main/ZhaoJianBing/substrate_account_and_ethereum_account_zh.md)

## 4. Assets Bridge
Refer [AssetsBridge](./xpallets/assets-bridge/README.md)
