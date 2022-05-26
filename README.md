# Singleton SherpaX 
Based on Substrate framework, 
Singleton SherpaX Chain is a blockchain adopting `aura + grandpa` consensus algorithm.


How to use/test locally

## 1. Supported dependencies

It should at least work until the following commits
- Substrate: polkadot-v0.9.18  @ `fc3fd073d3a0acf9933c3994b660ebd7b5833f65`
- ChainX frontier: polkadot-v0.9.18 @ `4e91f788ba02a0add42f09cae21bf96e3d346330`

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
- [Ethereum JSON-RPC.postman_collection](./develop_docs/Ethereum-JSON-RPC.postman_collection.json)
- [Ethereum RPC Support](https://github.com/PureStake/moonbeam-docs-cn/blob/master/builders/get-started/eth-compare/rpc-support.md)
- [QuickNode Ethereum RPC](https://www.quicknode.com/docs)
- [Ethereum JSON-RPC Wiki](https://eth.wiki/json-rpc/API#)

### 3.4 frontier account 
[How does frontier manage the substrate account and ethereum account](https://github.com/chainx-org/chainx-technical-archive/blob/main/ZhaoJianBing/substrate_account_and_ethereum_account.md)

## 4. Assets Bridge
Refer [AssetsBridge](./xpallets/assets-bridge/README.md)

## 5. SherpaX Mainnet
[shrepax mainnet chainspec(76MB, tar.gz)](./node/res/sherpax-raw.json.tar.gz)
```bash
tar zxvf ./node/res/sherpax-raw.json.tar.gz -C ./node/res/
```
or
[sherpax_mainnet_chainspec(276MB)](https://github.com/chainx-org/SherpaX/releases/download/v1.0.0/sherpax-raw.json)
```bash
./target/release/sherpax --chain=./node/res/sherpax-raw.json --tmp
```
