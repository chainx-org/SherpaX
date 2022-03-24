// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

//! Weights for xpallet_gateway_common
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-21, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("benchmarks"), DB CACHE: 128

// Executed Command:
// ./target/release/sherpax
// benchmark
// --chain=benchmarks
// --steps=50
// --repeat=20
// --pallet=xpallet_gateway_common
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./xpallets/gateway/common/src/weights.rs
// --template=./scripts/xpallet-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for xpallet_gateway_common.
pub trait WeightInfo {
    fn withdraw() -> Weight;
    fn cancel_withdrawal() -> Weight;
    fn setup_trustee() -> Weight;
    fn set_trustee_info_config() -> Weight;
    fn change_trustee_transition_duration() -> Weight;
    fn set_relayer() -> Weight;
    fn set_trustee_admin() -> Weight;
    fn set_trustee_admin_multiply() -> Weight;
    fn claim_trustee_reward() -> Weight;
}

/// Weights for xpallet_gateway_common using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn withdraw() -> Weight {
        (105_877_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    fn cancel_withdrawal() -> Weight {
        (36_626_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn setup_trustee() -> Weight {
        (83_092_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_trustee_info_config() -> Weight {
        (3_875_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn change_trustee_transition_duration() -> Weight {
        (2_447_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_relayer() -> Weight {
        (2_750_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_trustee_admin() -> Weight {
        (11_897_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_trustee_admin_multiply() -> Weight {
        (2_489_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn claim_trustee_reward() -> Weight {
        (167_065_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn withdraw() -> Weight {
        (105_877_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(8 as Weight))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
    }
    fn cancel_withdrawal() -> Weight {
        (36_626_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn setup_trustee() -> Weight {
        (83_092_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_trustee_info_config() -> Weight {
        (3_875_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn change_trustee_transition_duration() -> Weight {
        (2_447_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_relayer() -> Weight {
        (2_750_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_trustee_admin() -> Weight {
        (11_897_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_trustee_admin_multiply() -> Weight {
        (2_489_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn claim_trustee_reward() -> Weight {
        (167_065_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
    }
}
