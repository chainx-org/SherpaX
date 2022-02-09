# Deploy erc20 to SherpaX

This is a Hardhat setup to deploy the necessary contracts of Uniswap.
Forked from [moonbeam-uniswap](https://github.com/PureStake/moonbeam-uniswap)

## require "@openzeppelin/contracts": "4.4.1"

## Get Started

Clone repo:

```
git clone https://github.com/chainx-org/SherpaX.git
cd deploy/erc20-contracts
```

Set PRIVKEY env:
```
export PRIVKEY=0x.....
```

Update hardhat.config.js:
default url
```
url: 'http://127.0.0.1:8546'
```

Install packages:

```
npm i
```

Modify the private keys as you wish in the `hardhat.config.js` file.

### Deploy the contracts (Standalone)

To deploy the contracts in a Standalone node you can run:

```
npx hardhat run --network dev scripts/deploy-erc20.js 
```
or
```
npx hardhat run --network dev scripts/deploy-erc20-admin.js 
```
