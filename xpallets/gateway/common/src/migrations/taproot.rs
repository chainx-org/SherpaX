// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use crate::{Config, TrusteeAdmin, TrusteeMultiSigAddr};
use frame_support::{log::info, traits::Get, weights::Weight};

/// Apply all of the migrations due to AccountId.
///
/// ### Warning
///
/// Use with care and run at your own risk.
pub fn apply<T: Config>() -> Weight {
    info!(
        target: "runtime::gateway::common",
        "Running migration for gateway common pallet"
    );

    migrate_trustee_admin::<T>().saturating_add(migrate_trustee_multisig_addr::<T>())
}

/// Migrate from the old trustee session info.
pub fn migrate_trustee_multisig_addr<T: Config>() -> Weight {
    TrusteeMultiSigAddr::<T>::translate::<T::AccountId, _>(|_, n| Some(n));
    let count = TrusteeMultiSigAddr::<T>::iter_values().count();
    info!(
        target: "runtime::gateway::common",
        "migrated {} trustee multisig addr.",
        count,
    );
    <T as frame_system::Config>::DbWeight::get().reads_writes(count as Weight, count as Weight)
}

/// Migrate from the old trustee intention properties.
pub fn migrate_trustee_admin<T: Config>() -> Weight {
    if TrusteeAdmin::<T>::translate::<T::AccountId, _>(|n| n).is_ok() {
        info!(
            target: "runtime::gateway::common",
            "migrated trustee admin success."
        );
    } else {
        info!(
            target: "runtime::gateway::common",
            "migrated trustee admin fail."
        );
    }

    <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1)
}
