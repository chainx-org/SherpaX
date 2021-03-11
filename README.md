How to use/test locally

### 1. Supported Polkadot dependencies

It should at least work until the following commits

-   rustc 1.52.0-nightly (d1206f950 2021-02-15)
-   polkadot.js.org (apps v0.80.2-19)
-   Polkadot rococo-v1 @ `72243baaedf3ded4226032949a23f8478f5565d9`
-   Cumulus rococo-v1 @ `24b1ee6bd1d96f255889f167e59ef9c9399a6305`
-   Substrate rococo-v1 (newer than 2.0.0) @ `645299e8b23ec5fa52935b1a6edbf36886e80141`

### 2. How to use

1. Spin up Polkadot validators (number of parachains + 1)
2. Spin up Collator(s)

Recommend checking out the [cumulus-workshop](https://substrate.dev/cumulus-workshop/#/3-parachains/1-launch) and following most of the steps described there, mainly 3.
Unfortunately, some commands there are outdated as the workshop has not been updated to the newest Rococo version, yet.
The following code is basically copied from there and updated to the new version to have a one-page-overview for all commands and steps.
Please check out the workshop for explanations.

### 3. Launch a local setup including a Relay Chain and a Parachain

#### Launch the Relay Chain

```bash
# Compile Polkadot with the real overseer feature
git clone https://github.com/paritytech/polkadot
git checkout rococo-v1
cargo build --release --features=real-overseer

# Generate a raw chain spec
./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode > rococo-custom-local.json
sed -i 's/"validation_upgrade_frequency": 600/"validation_upgrade_frequency": 10/g' rococo-custom-local.json
sed -i 's/"validation_upgrade_delay": 300/"validation_upgrade_delay": 5/g' rococo-custom-local.json
./target/release/polkadot build-spec --chain rococo-custom-local.json --disable-default-bootnode --raw > rococo-local-cfde-real-overseer.json

# Alice
./target/release/polkadot --chain rococo-local-cfde-real-overseer.json --alice --tmp

# Bob (In a separate terminal)
./target/release/polkadot --chain rococo-local-cfde-real-overseer.json --bob --tmp --port 30334
```

#### Launch the Parachain
`parachain-id` is u32 type. 
e.g `parachain-id=59`

```bash
# Compile
cargo build --release

# Export genesis state
# --parachain-id 59 as an example that can be chosen freely. Make sure to everywhere use the same parachain id
./target/release/sherpax export-genesis-state --parachain-id 59 > genesis-state

# Export genesis wasm
./target/release/sherpax export-genesis-wasm > genesis-wasm

# Collator1
./target/release/sherpax --collator --tmp --parachain-id 59 --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-cfde-real-overseer.json --port 30335

# Collator2
./target/release/sherpax --collator --tmp --parachain-id 59 --port 40336 --ws-port 9947 -- --execution wasm --chain ../polkadot/rococo-local-cfde-real-overseer.json --port 30336

# Parachain Full Node 1
./target/release/sherpax --tmp --parachain-id 59 --port 40337 --ws-port 9948 -- --execution wasm --chain ../polkadot/rococo-local-cfde-real-overseer.json --port 30337
```

### 4. Register the parachain
![image](https://user-images.githubusercontent.com/2915325/99548884-1be13580-2987-11eb-9a8b-20be658d34f9.png)

