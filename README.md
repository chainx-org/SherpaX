How to use/test locally

### 1. Supported Polkadot dependencies

It should at least work until the following commits

-   polkadot.js.org (apps v0.86.3-54)
-   Polkadot rococo-v1 @ `127eb17a25bbe2a9f2731ff11a65d7f8170f2373`
-   Cumulus rococo-v1 @ `da4c3bac6e9584e65740ef5db4dbd2c31c1a91db`
-   Substrate rococo-v1  @ `2be8fcc4236d32786c62f6f27a98e7fe7e550807`

### 2. How to use

1. Spin up Polkadot validators (number of parachains + 1)
2. Spin up Collator(s)

Recommend checking out the [cumulus-workshop](https://substrate.dev/cumulus-workshop/#/3-parachains/1-launch) and following most of the steps described there, mainly 3.
Unfortunately, some commands there are outdated as the workshop has not been updated to the newest Rococo version, yet.
The following code is basically copied from there and updated to the new version to have a one-page-overview for all commands and steps.
Please check out the workshop for explanations.

### 3. Launch a local setup including a Relay Chain and a Parachain by cmd

#### Launch the Relay Chain

```bash
# Compile Polkadot
git clone https://github.com/paritytech/polkadot
git checkout 127eb17a25bbe2a9f2731ff11a65d7f8170f2373
cargo build --release

# Generate a raw chain spec
./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode > rococo-custom-local.json
./target/release/polkadot build-spec --chain rococo-custom-local.json --disable-default-bootnode --raw > rococo-local-raw.json

# Alice
./target/release/polkadot --chain rococo-local-raw.json --alice --tmp

# Bob (In a separate terminal)
./target/release/polkadot --chain rococo-local-raw.json --bob --tmp --port 30334
```

#### Launch the Parachain
`parachain-id` is u32 type. 
e.g `parachain-id=1059`

```bash
# Compile
cargo build --release

# Export genesis state
# --parachain-id 1059 as an example that can be chosen freely. Make sure to everywhere use the same parachain id
./target/release/sherpax export-genesis-state --parachain-id 1059 > genesis-state

# Export genesis wasm
./target/release/sherpax export-genesis-wasm > genesis-wasm

# Collator
./target/release/sherpax --validator --tmp --parachain-id 1059 --port 40335 --ws-port 9977 -- --execution wasm --chain ../polkadot/rococo-local-raw.json --port 30335

# Parachain Full Node
./target/release/sherpax --tmp --parachain-id 1059 --port 40337 --ws-port 9988 -- --execution wasm --chain ../polkadot/rococo-local-raw.json --port 30337
```

#### Register the parachain
![image](https://user-images.githubusercontent.com/2915325/99548884-1be13580-2987-11eb-9a8b-20be658d34f9.png)
