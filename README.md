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
    --port 30333
    -levm=trace
```

## 3. Evm support

### 3.1 metamask (for account)

```txt
Network Name: sherpax
New RPC URL: http://127.0.0.1:8546
Chain ID: 1505
Currency Symbol: KSX
Block Explorer URL:
```

Refer [Connect MetaMask to Moonbase Alpha](https://docs.moonbeam.network/getting-started/moonbase/metamask/)

### 3.2 Remix (for contract)

Refer [Interacting with Moonbeam Using Remix](https://docs.moonbeam.network/getting-started/local-node/using-remix/)
