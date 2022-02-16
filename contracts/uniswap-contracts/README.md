# Deploy Uniswap V2 to SherpaX

This is a Hardhat setup to deploy the necessary contracts of Uniswap.
Forked from [moonbeam-uniswap](https://github.com/PureStake/moonbeam-uniswap)

## Get Started

Clone repo:

```
git clone https://github.com/chainx-org/SherpaX.git
cd deploy/uniswap-contracts
```

Install packages:

```
npm i
```

Modify the private keys as you wish in the `hardhat.config.js` file.

### Deploy the contracts (Standalone)

To deploy the contracts in a Standalone node you can run:

#### Script

```
export PRIVKEY= Your privateKey
npx hardhat run --network dev scripts/deploy-factory.js 
```

#### Remix

To collect to localhost

```
bash ./script/remix.sh 
```

setting
* evmVersion: istanbul
* Enable optimization: true

