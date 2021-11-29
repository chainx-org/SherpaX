How to use/test locally

### 1. Supported Polkadot dependencies

It should at least work until the following commits

-   polkadot.js.org (apps v0.98.2-106)
-   Polkadot release-v0.9.11 @ `eb9f107e3a04687dcf80111563f2bcea7d5b15d3`
-   Cumulus polkadot-v0.9.11 @ `ede4d527c4fc5d84c43216b408a873625488574b`
-   Substrate polkadot-v0.9.11  @ `57346f6b24875f8935280dba51fa8ab0a9ba1e39`

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
git checkout eb9f107e3a04687dcf80111563f2bcea7d5b15d3
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
