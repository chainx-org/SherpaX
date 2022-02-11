#!/usr/bin/env bash

set -e

cd "$(dirname "$0")"

cd ..

# cargo build --release --features=runtime-benchmarks

bench_run() {
  pallet=$1
  output=$2
  ./target/release/sherpax benchmark \
    --chain=benchmarks \
    --steps=50 \
    --repeat=20 \
    --pallet="$pallet" \
    --extrinsic="*" \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output="$output" \
    --template=./scripts/xpallet-weight-template.hbs

  rustfmt "$output"
}

bench_run xpallet_gateway_bitcoin   ./xpallets/gateway/bitcoin/src/weights.rs
bench_run xpallet_gateway_common    ./xpallets/gateway/common/src/weights.rs
bench_run xpallet_gateway_records   ./xpallets/gateway/records/src/weights.rs
