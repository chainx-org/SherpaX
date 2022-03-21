// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

//! Weights for xpallet_gateway_bitcoin
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-21, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("benchmarks"), DB CACHE: 128

// Executed Command:
// ./target/release/sherpax
// benchmark
// --chain=benchmarks
// --steps=50
// --repeat=20
// --pallet=xpallet_gateway_bitcoin
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./xpallets/gateway/bitcoin/src/weights.rs
// --template=./scripts/xpallet-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for xpallet_gateway_bitcoin.
pub trait WeightInfo {
    fn push_header() -> Weight;
    fn push_transaction() -> Weight;
    fn create_taproot_withdraw_tx() -> Weight;
    fn set_best_index() -> Weight;
    fn set_confirmed_index() -> Weight;
    fn remove_pending() -> Weight;
    fn remove_proposal() -> Weight;
    fn set_btc_withdrawal_fee() -> Weight;
    fn set_btc_deposit_limit() -> Weight;
}

/// Weights for xpallet_gateway_bitcoin using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn push_header() -> Weight {
        (101_871_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(10 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    fn push_transaction() -> Weight {
        (240_105_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(20 as Weight))
            .saturating_add(T::DbWeight::get().writes(9 as Weight))
    }
    fn create_taproot_withdraw_tx() -> Weight {
        (141_186_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(14 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn set_best_index() -> Weight {
        (3_029_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_confirmed_index() -> Weight {
        (3_125_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn remove_pending() -> Weight {
        (189_661_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn remove_proposal() -> Weight {
        (47_824_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn set_btc_withdrawal_fee() -> Weight {
        (2_311_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_btc_deposit_limit() -> Weight {
        (2_399_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn push_header() -> Weight {
        (101_871_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(10 as Weight))
            .saturating_add(RocksDbWeight::get().writes(5 as Weight))
    }
    fn push_transaction() -> Weight {
        (240_105_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(20 as Weight))
            .saturating_add(RocksDbWeight::get().writes(9 as Weight))
    }
    fn create_taproot_withdraw_tx() -> Weight {
        (141_186_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(14 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn set_best_index() -> Weight {
        (3_029_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_confirmed_index() -> Weight {
        (3_125_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn remove_pending() -> Weight {
        (189_661_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn remove_proposal() -> Weight {
        (47_824_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn set_btc_withdrawal_fee() -> Weight {
        (2_311_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_btc_deposit_limit() -> Weight {
        (2_399_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
}
