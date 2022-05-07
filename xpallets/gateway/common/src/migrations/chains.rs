// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use crate::{
    traits::BytesLike, Config, GenericTrusteeIntentionProps, LittleBlackHouse,
    TrusteeIntentionPropertiesOf, TrusteeIntentionProps, TrusteeSigRecord, TrusteeTransitionStatus,
};
use codec::{Decode, Encode};
use frame_support::{
    log::info,
    migration::{storage_key_iter, take_storage_value},
    traits::Get,
    weights::Weight,
    RuntimeDebug, Twox64Concat,
};
use musig2::PublicKey;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sherpax_primitives::Text;
use sp_std::prelude::*;
use xp_assets_registrar::Chain;

/// The trustee intention properties.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct OldTrusteeIntentionProps<TrusteeEntity: BytesLike> {
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_text"))]
    pub about: Text,
    pub hot_entity: TrusteeEntity,
    pub cold_entity: TrusteeEntity,
}
/// The generic trustee intention properties.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct OldGenericTrusteeIntentionProps(pub OldTrusteeIntentionProps<Vec<u8>>);

/// Apply all of the migrations due to taproot.
///
/// ### Warning
///
/// Use with care and run at your own risk.
pub fn apply<T: Config>() -> Weight {
    info!(
        target: "runtime::gateway::common",
        "Running migration for gateway common pallet"
    );

    migrate_trustee_sig_record::<T>()
        .saturating_add(migrate_trustee_transition_status::<T>())
        .saturating_add(migrate_little_black_house::<T>())
        .saturating_add(migrate_trustee_intention_properties::<T>())
}

/// Migrate from the old trustee session info.
pub fn migrate_trustee_sig_record<T: Config>() -> Weight {
    for (trustee, record) in
        storage_key_iter::<T::AccountId, u64, Twox64Concat>(b"XGatewayCommon", b"TrusteeSigRecord")
            .drain()
    {
        TrusteeSigRecord::<T>::insert(Chain::Bitcoin, trustee, record);
    }
    let count = TrusteeSigRecord::<T>::iter_values().count();
    info!(
        target: "runtime::gateway::common",
        "migrated trustee_intention_properties.",
    );
    <T as frame_system::Config>::DbWeight::get().reads_writes(count as Weight, count as Weight)
}

/// Migrate from the old trustee transition status.
pub fn migrate_trustee_transition_status<T: Config>() -> Weight {
    if let Some(status) =
        take_storage_value::<bool>(b"XGatewayCommon", b"TrusteeTransitionStatus", b"")
    {
        TrusteeTransitionStatus::<T>::insert(Chain::Bitcoin, status);
    }

    info!(
        target: "runtime::gateway::common",
        "migrated trustee_transition_status.",
    );
    <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1)
}

/// Migrate from the old little black house.
pub fn migrate_little_black_house<T: Config>() -> Weight {
    if let Some(trustees) =
        take_storage_value::<Vec<T::AccountId>>(b"XGatewayCommon", b"LittleBlackHouse", b"")
    {
        LittleBlackHouse::<T>::insert(Chain::Bitcoin, trustees);
    }
    info!(
        target: "runtime::gateway::common",
        "migrated little_black_house.",
    );
    <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1)
}

/// Migrate from the old trustee intention properties.
pub fn migrate_trustee_intention_properties<T: Config>() -> Weight {
    TrusteeIntentionPropertiesOf::<T>::translate::<OldGenericTrusteeIntentionProps, _>(
        |_, _, props| {
            let hot_key = &props.0.hot_entity;
            let cold_key = &props.0.cold_entity;
            let hot_pubkey =
                PublicKey::parse_slice(hot_key).expect("must be success, or panic; qed");
            let cold_pubkey =
                PublicKey::parse_slice(cold_key).expect("must be success, or panic; qed");
            // Unified use of the full public key
            Some(GenericTrusteeIntentionProps(TrusteeIntentionProps {
                proxy_account: None,
                about: props.0.about,
                hot_entity: hot_pubkey.serialize().to_vec(),
                cold_entity: cold_pubkey.serialize().to_vec(),
            }))
        },
    );
    let count = TrusteeIntentionPropertiesOf::<T>::iter_values().count();
    info!(
        target: "runtime::gateway::common",
        "migrated {} trustee_intention_properties.",
        count,
    );
    <T as frame_system::Config>::DbWeight::get()
        .reads_writes(count as Weight + 1, count as Weight + 1)
}
