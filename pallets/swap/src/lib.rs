// Copyright 2019-2021 ChainX Project Authors. Licensed under GPL-3.0.

//! # SWAP Module
//!
//! ## Overview
//!
//! Built-in decentralized exchange modules in SherpaX network, the swap
//! mechanism refers to the design of Uniswap V2.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// mod types;
mod traits;
pub use traits::MultiAsset;

use frame_support::{
    inherent::Vec,
    RuntimeDebug,
};
use codec::{Encode, Decode, FullCodec};
use sp_runtime::{
    traits::{
        StaticLookup, AccountIdConversion, MaybeSerializeDeserialize
    },
};
use sp_arithmetic::traits::BaseArithmetic;
use sp_std::fmt::Debug;
use frame_support::{
    pallet_prelude::*,
    PalletId,
};

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

pub type AssetIdOf<T> =
<<T as Config>::MultiAssets as MultiAsset<<T as frame_system::Config>::AccountId>>::AssetId;

pub type AssetBalanceOf<T> =
<<T as Config>::MultiAssets as MultiAsset<<T as frame_system::Config>::AccountId>>::AssetBalance;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::dispatch::DispatchResult;
    use frame_system::pallet_prelude::*;
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The assets interface beyond native currency and xpallet_assets.
        type MultiAssets: MultiAsset<Self::AccountId>;
        /// This pallet Id.
        type PalletId: Get<PalletId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn swap_metadata)]
    /// TWOX-NOTE: `AssetId` is trusted, so this is safe.
    /// (AssetId, AssetId) -> (PairAccountId, TotalSupply)
    pub type SwapMetadata<T: Config> = StorageMap<_, Twox64Concat, (AssetIdOf<T>, AssetIdOf<T>), (T::AccountId, AssetBalanceOf<T>)>;

    #[pallet::storage]
    #[pallet::getter(fn swap_ledger)]
    /// ((AssetId, AssetId), AccountId) -> AssetBalance
    pub type SwapLedger<T: Config> = StorageMap<_, Blake2_128Concat, ((AssetIdOf<T>, AssetIdOf<T>), T::AccountId), AssetBalanceOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId", AssetIdOf<T> = "AssetId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PairCreated(T::AccountId, AssetIdOf<T>, AssetIdOf<T>)
    }

    #[pallet::error]
    pub enum Error<T> {
        PairAlreadyExists
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T:Config> Pallet<T> {

        #[pallet::weight(1000_000)]
        pub fn create_pair(
            origin: OriginFor<T>,
            asset_0: AssetIdOf<T>,
            asset_1: AssetIdOf<T>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);

            let pair_account = Self::pair_account_id(asset_0, asset_1);

            SwapMetadata::<T>::try_mutate((asset_0, asset_1), |meta|{
                ensure!(meta.is_none(), Error::<T>::PairAlreadyExists);

                *meta = Some(
                    (pair_account, Default::default())
                );

                Self::deposit_event(Event::PairCreated(who, asset_0, asset_1));

                Ok(())
            })
        }

        #[pallet::weight(1000_000)]
        #[frame_support::transactional]
        #[allow(clippy::too_many_arguments)]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            asset_0: AssetIdOf<T>,
            asset_1: AssetIdOf<T>,
            #[pallet::compact] amount_0_desired : AssetBalanceOf<T>,
            #[pallet::compact] amount_1_desired : AssetBalanceOf<T>,
            #[pallet::compact] amount_0_min : AssetBalanceOf<T>,
            #[pallet::compact] amount_1_min : AssetBalanceOf<T>,
            #[pallet::compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            Ok(())
        }

        #[pallet::weight(1000_000)]
        #[frame_support::transactional]
        #[allow(clippy::too_many_arguments)]
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            asset_0: AssetIdOf<T>,
            asset_1: AssetIdOf<T>,
            #[pallet::compact] liquidity: AssetBalanceOf<T>,
            #[pallet::compact] amount_asset_0_min : AssetBalanceOf<T>,
            #[pallet::compact] amount_asset_1_min : AssetBalanceOf<T>,
            recipient: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            Ok(())
        }

        #[pallet::weight(1000_000)]
        #[frame_support::transactional]
        pub fn swap_exact_tokens_for_tokens(
            origin: OriginFor<T>,
            #[pallet::compact] amount_in: AssetBalanceOf<T>,
            #[pallet::compact] amount_out_min: AssetBalanceOf<T>,
            path: Vec<AssetIdOf<T>>,
            recipient: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            Ok(())
        }

        #[pallet::weight(1000_000)]
        #[frame_support::transactional]
        pub fn swap_tokens_for_exact_tokens(
            origin: OriginFor<T>,
            #[pallet::compact] amount_out: AssetBalanceOf<T>,
            #[pallet::compact] amount_in_max: AssetBalanceOf<T>,
            path: Vec<AssetIdOf<T>>,
            recipient: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// The account ID of a pair account
    pub fn pair_account_id(asset_0: AssetIdOf<T>, asset_1: AssetIdOf<T>) -> T::AccountId {
        let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);

        T::PalletId::get().into_sub_account((asset_0, asset_1))
    }

    /// Sorted the asset id of assets pair
    pub fn sort_asset_id(asset_0: AssetIdOf<T>, asset_1: AssetIdOf<T>) -> (AssetIdOf<T>, AssetIdOf<T>) {
        if asset_0 < asset_1 {
            (asset_0, asset_1)
        } else {
            (asset_1, asset_0)
        }
    }

}
