// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

extern crate alloc;
use crate::{AssetChainOf, Config};
use frame_support::{log::info, traits::Get, weights::Weight};
use sp_runtime::SaturatedConversion;
use xp_assets_registrar::Chain;

/// Initialize the new module by storing migration
pub fn apply<T: Config>() -> Weight {
    info!(
        target: "runtime::gateway::record",
        "✅ Running migration for gateway record of dogecoin"
    );
    dogecoin_genesis::<T>()
}

pub fn dogecoin_genesis<T: Config>() -> Weight {
    let dogecoin_asset_id: T::AssetId = 9u32.saturated_into();
    AssetChainOf::<T>::insert(dogecoin_asset_id, Chain::Dogecoin);

    info!(
        target: "runtime::gateway::record",
        "✅ Migration for record of genesis done"
    );
    <T as frame_system::Config>::DbWeight::get().writes(1)
}
