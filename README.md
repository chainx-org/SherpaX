How to use/test locally

### 1. Supported Polkadot dependencies

It should at least work until the following commits

-   polkadot.js.org (apps apps v0.95.2-81)
-   Polkadot release-v0.9.9 @ `0f4a7663c605d4b002ccae02df22a7fc0819d582`
-   Cumulus polkadot-v0.9.9 @ `fd80849dde5c209c20a996cfcc5aaacd4666dcbe`
-   Substrate polkadot-v0.9.9  @ `91061a7d925b5bc597804293da283477512ba4ff`
-   PureStake/frontier polkadot-v0.9.9 @ `4c91772259f5d1436274af24a9fd9304e54aefe6`

### 2. How to use

1. Spin up Polkadot validators (number of parachains + 1)
2. Spin up Collator(s)

Recommend checking out the [cumulus-workshop](https://substrate.dev/cumulus-workshop/#/3-parachains/1-launch) and following most of the steps described there, mainly 3.
Unfortunately, some commands there are outdated as the workshop has not been updated to the newest Rococo version, yet.
The following code is basically copied from there and updated to the new version to have a one-page-overview for all commands and steps.
Please check out the workshop for explanations.

### 3. Launch a local setup including a Relay Chain and a Parachain

- polkadot & sherpax
```bash
# Compile Polkadot
git clone https://github.com/paritytech/polkadot
git checkout 0f4a7663c605d4b002ccae02df22a7fc0819d582
cargo build --release

# Compile SherpaX
cargo build --release
```

- by docker(only support linux)
```bash
cd docker
./build.sh
./docker-compose up
```

- or by shell script
```bash
cd docker
# copy polkadot & sherpax to current directory
./regenerateConfig-rococo-local.sh
./start-all.sh
```

### 4. custom type

```json
{
  "Address": "MultiAddress",
  "LookupSource": "MultiAddress",
  "Cid": "u64",
  "BondType": "u16",
  "BondData": {
    "bond_type": "BondType",
    "data": "Bytes"
  },
  "CidDetails": {
    "owner": "AccountId",
    "bonds": "Vec<BondData>",
    "card":  "Bytes"
  }
}
```

# EVM Support
## 0. development accounts have 1000 KSX
```
Alith:
Public Address: 0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac
Private Key: 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133

Baltathar:
Public Address: 0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0
Private Key: 0x8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b

Charleth:
Public Address: 0x798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc
Private Key: 0x0b6e18cafb6ed99687ec547bd28139cafdd2bffe70e6b688025de6b445aa5c5b

Dorothy:
Public Address: 0x773539d4Ac0e786233D90A233654ccEE26a613D9
Private Key: 0x39539ab1876910bbf3a223d84a29e28f1cb4e2e456503e7e91ed39b2e7223d68

Ethan:
Public Address: 0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB
Private Key: 0x7dce9bc8babb68fec1409be38c8e1a52650206a7ed90ff956ae8a6d15eeaaef4

Faith:
Public Address: 0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d
Private Key: 0xb9d2ea9a615f3165812e8d44de0d24da9bbd164b65c4f0573e1ce2c8dbd9c8df

Goliath:
Public Address: 0x7BF369283338E12C90514468aa3868A551AB2929
Private Key: 0x96b8a38e12e1a31dee1eab2fffdf9d9990045f5b37e44d8cc27766ef294acf18

Heath:
Public Address: 0x931f3600a299fd9B24cEfB3BfF79388D19804BeA
Private Key: 0x0d6dcaaef49272a5411896be8ad16c01c35d6f8c18873387b71fbc734759b0ab

Ida:
Public Address: 0xC41C5F1123ECCd5ce233578B2e7ebd5693869d73
Private Key: 0x4c42532034540267bf568198ccec4cb822a025da542861fcb146a5fab6433ff8

Judith:
Public Address: 0x2898FE7a42Be376C8BC7AF536A940F7Fd5aDd423
Private Key: 0x94c49300a58d576011096bcb006aa06f5a91b34b4383891e8029c21dc39fbb8b

Gerald:
Public Address: 0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b
Private Key: 0x99b3c12287537e38c90a9219d4cb074a89a16e9cdb20bf85728ebd97c343e342
```

## 1. metamask (for account)

```txt
Network Name: mini-test
New RPC URL: http://127.0.0.1:8545
Chain ID: 1500
Currency Symbol: MINI
Block Explorer URL:
```

Refer [Connect MetaMask to Moonbase Alpha](https://docs.moonbeam.network/getting-started/moonbase/metamask/)

## 2. Remix (for contract)

Refer [Interacting with Moonbeam Using Remix](https://docs.moonbeam.network/getting-started/local-node/using-remix/)
