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
    #[pallet::metadata(T::AccountId = "AccountId", AssetIdOf<T> = "AssetId", AssetBalanceOf<T> = "AssetBalance")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Create a trading pair. \[creator, asset_id, asset_id\]
        PairCreated(T::AccountId, AssetIdOf<T>, AssetIdOf<T>),
        /// Add liquidity. \[owner, asset_id, asset_id\]
        LiquidityAdded(T::AccountId, AssetIdOf<T>, AssetIdOf<T>),
        /// Remove liquidity. \[owner, recipient, asset_id, asset_id, amount\]
        LiquidityRemoved(T::AccountId, T::AccountId, AssetIdOf<T>, AssetIdOf<T>, AssetBalanceOf<T>),
        /// Transact in trading \[owner, recipient, swap_path\]
        TokenSwap(T::AccountId, T::AccountId, Vec<AssetIdOf<T>>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Trading pair can't be created.
        DeniedCreatePair,
        /// Trading pair already exists.
        PairAlreadyExists,
        /// Trading pair does not exist.
        PairNotExists,
        /// Liquidity is not enough.
        InsufficientLiquidity,
        /// Trading pair does have enough asset.
        InsufficientPairReserve,
        /// Get target amount is less than exception.
        InsufficientTargetAmount,
        /// Sold amount is more than exception.
        ExcessiveSoldAmount,
        /// Can't find pair though trading path.
        InvalidPath,
        /// Incorrect asset amount range.
        IncorrectAssetAmountRange,
        /// Overflow.
        Overflow,
        /// Transaction block number is larger than the end block number.
        Deadline,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T:Config> Pallet<T> {

        /// Create pair by two assets.
        ///
        /// The order of asset dot effect result.
        ///
        /// # Arguments
        ///
        /// - `asset_0`: Asset which make up Pair
        /// - `asset_1`: Asset which make up Pair
        #[pallet::weight(1000_000)]
        pub fn create_pair(
            origin: OriginFor<T>,
            asset_0: AssetIdOf<T>,
            asset_1: AssetIdOf<T>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(asset_0 != asset_1, Error::<T>::DeniedCreatePair);

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

        /// Provide liquidity to a pair.
        ///
        /// The order of asset dot effect result.
        ///
        /// # Arguments
        ///
        /// - `asset_0`: Asset which make up pair
        /// - `asset_1`: Asset which make up pair
        /// - `amount_0_desired`: Maximum amount of asset_0 added to the pair
        /// - `amount_1_desired`: Maximum amount of asset_1 added to the pair
        /// - `amount_0_min`: Minimum amount of asset_0 added to the pair
        /// - `amount_1_min`: Minimum amount of asset_1 added to the pair
        /// - `deadline`: Height of the cutoff block of this transaction
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
            let who = ensure_signed(origin)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_add_liquidity(
                &who, &asset_0, &asset_1, amount_0_desired, amount_1_desired, amount_0_min, amount_1_min
            )?;

            Self::deposit_event(Event::LiquidityAdded(who, asset_0, asset_1));

            Ok(())
        }

        /// Extract liquidity.
        ///
        /// The order of asset dot effect result.
        ///
        /// # Arguments
        ///
        /// - `asset_0`: Asset which make up pair
        /// - `asset_1`: Asset which make up pair
        /// - `amount_asset_0_min`: Minimum amount of asset_0 to exact
        /// - `amount_asset_1_min`: Minimum amount of asset_1 to exact
        /// - `recipient`: Account that accepts withdrawal of assets
        /// - `deadline`: Height of the cutoff block of this transaction
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
            let who = ensure_signed(origin)?;
            let recipient = T::Lookup::lookup(recipient)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_remove_liquidity(
                &who, &asset_0, &asset_1, liquidity, amount_asset_0_min, amount_asset_1_min, &recipient
            )?;

            Self::deposit_event(Event::LiquidityRemoved(who, recipient, asset_0, asset_1, liquidity));

            Ok(())
        }

        /// Sell amount of asset by path.
        ///
        /// # Arguments
        ///
        /// - `amount_in`: Amount of the asset will be sold
        /// - `amount_out_min`: Minimum amount of target asset
        /// - `path`: path can convert to pairs.
        /// - `recipient`: Account that receive the target asset
        /// - `deadline`: Height of the cutoff block of this transaction
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
            let who = ensure_signed(origin)?;
            let recipient = T::Lookup::lookup(recipient)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_swap_exact_tokens_for_tokens(
                &who, amount_in, amount_out_min, &path, &recipient
            )?;

            Self::deposit_event(Event::TokenSwap(who, recipient, path));

            Ok(())
        }

        /// Buy amount of asset by path.
        ///
        /// # Arguments
        ///
        /// - `amount_out`: Amount of the asset will be bought
        /// - `amount_in_max`: Maximum amount of sold asset
        /// - `path`: path can convert to pairs.
        /// - `recipient`: Account that receive the target asset
        /// - `deadline`: Height of the cutoff block of this transaction
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
            let who = ensure_signed(origin)?;
            let recipient = T::Lookup::lookup(recipient)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_swap_tokens_for_exact_tokens(
                &who, amount_out, amount_in_max, &path, &recipient
            )?;

            Self::deposit_event(Event::TokenSwap(who, recipient, path));

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// The account ID of a pair account
    fn pair_account_id(asset_0: AssetIdOf<T>, asset_1: AssetIdOf<T>) -> T::AccountId {
        let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);

        T::PalletId::get().into_sub_account((asset_0, asset_1))
    }

    /// Sorted the asset id of assets pair
    fn sort_asset_id(asset_0: AssetIdOf<T>, asset_1: AssetIdOf<T>) -> (AssetIdOf<T>, AssetIdOf<T>) {
        if asset_0 < asset_1 {
            (asset_0, asset_1)
        } else {
            (asset_1, asset_0)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_add_liquidity(
        who: &T::AccountId,
        asset_0: &AssetIdOf<T>,
        asset_1: &AssetIdOf<T>,
        amount_0_desired: AssetBalanceOf<T>,
        amount_1_desired: AssetBalanceOf<T>,
        amount_0_min: AssetBalanceOf<T>,
        amount_1_min: AssetBalanceOf<T>,
    ) -> DispatchResult {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_remove_liquidity(
        who: &T::AccountId,
        asset_0: &AssetIdOf<T>,
        asset_1: &AssetIdOf<T>,
        remove_liquidity: AssetBalanceOf<T>,
        amount_token_0_min: AssetBalanceOf<T>,
        amount_token_1_min: AssetBalanceOf<T>,
        recipient: &T::AccountId,
    ) -> DispatchResult {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_swap_exact_tokens_for_tokens(
        who: &T::AccountId,
        amount_in: AssetBalanceOf<T>,
        amount_out_min: AssetBalanceOf<T>,
        path: &[AssetIdOf<T>],
        recipient: &T::AccountId,
    ) -> DispatchResult {
        Ok(())
    }

    pub fn inner_swap_tokens_for_exact_tokens(
        who: &T::AccountId,
        amount_out: AssetBalanceOf<T>,
        amount_in_max: AssetBalanceOf<T>,
        path: &[AssetIdOf<T>],
        recipient: &T::AccountId,
    ) -> DispatchResult {
        Ok(())
    }
}
