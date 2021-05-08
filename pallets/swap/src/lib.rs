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
        StaticLookup, AccountIdConversion, Saturating, Zero, IntegerSquareRoot
    },
};

use sp_std::{
    fmt::Debug, convert::TryInto
};
use sp_core::U256;
use frame_support::{
    pallet_prelude::*,
    PalletId,
};

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

pub type AssetId = u32;
pub type AssetBalance = u128;

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
    pub type SwapMetadata<T: Config> = StorageMap<_, Twox64Concat, (AssetId, AssetId), (T::AccountId, AssetBalance)>;

    #[pallet::storage]
    #[pallet::getter(fn swap_ledger)]
    /// ((AssetId, AssetId), AccountId) -> AssetBalance
    pub type SwapLedger<T: Config> = StorageMap<_, Blake2_128Concat, ((AssetId, AssetId), T::AccountId), AssetBalance, ValueQuery>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Create a trading pair. \[creator, asset_id, asset_id\]
        PairCreated(T::AccountId, AssetId, AssetId),
        /// Add liquidity. \[owner, asset_id, asset_id\]
        LiquidityAdded(T::AccountId, AssetId, AssetId),
        /// Remove liquidity. \[owner, recipient, asset_id, asset_id, amount\]
        LiquidityRemoved(T::AccountId, T::AccountId, AssetId, AssetId, AssetBalance),
        /// Transact in trading \[owner, recipient, swap_path\]
        TokenSwap(T::AccountId, T::AccountId, Vec<AssetId>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account balance must be greater than or equal to the transfer amount.
        InsufficientAssetBalance,
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
            asset_0: AssetId,
            asset_1: AssetId
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
            asset_0: AssetId,
            asset_1: AssetId,
            #[pallet::compact] amount_0_desired : AssetBalance,
            #[pallet::compact] amount_1_desired : AssetBalance,
            #[pallet::compact] amount_0_min : AssetBalance,
            #[pallet::compact] amount_1_min : AssetBalance,
            #[pallet::compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let now = frame_system::Pallet::<T>::block_number();
            let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_add_liquidity(
                &who, asset_0, asset_1, amount_0_desired, amount_1_desired, amount_0_min, amount_1_min
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
            asset_0: AssetId,
            asset_1: AssetId,
            #[pallet::compact] liquidity: AssetBalance,
            #[pallet::compact] amount_asset_0_min : AssetBalance,
            #[pallet::compact] amount_asset_1_min : AssetBalance,
            recipient: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let recipient = T::Lookup::lookup(recipient)?;
            let now = frame_system::Pallet::<T>::block_number();
            let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_remove_liquidity(
                &who, asset_0, asset_1, liquidity, amount_asset_0_min, amount_asset_1_min, &recipient
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
            #[pallet::compact] amount_in: AssetBalance,
            #[pallet::compact] amount_out_min: AssetBalance,
            path: Vec<AssetId>,
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
            #[pallet::compact] amount_out: AssetBalance,
            #[pallet::compact] amount_in_max: AssetBalance,
            path: Vec<AssetId>,
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
    fn pair_account_id(asset_0: AssetId, asset_1: AssetId) -> T::AccountId {
        let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);

        T::PalletId::get().into_sub_account((asset_0, asset_1))
    }

    /// Sorted the asset id of assets pair
    fn sort_asset_id(asset_0: AssetId, asset_1: AssetId) -> (AssetId, AssetId) {
        if asset_0 < asset_1 {
            (asset_0, asset_1)
        } else {
            (asset_1, asset_0)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_add_liquidity(
        who: &T::AccountId,
        asset_0: AssetId,
        asset_1: AssetId,
        amount_0_desired: AssetBalance,
        amount_1_desired: AssetBalance,
        amount_0_min: AssetBalance,
        amount_1_min: AssetBalance,
    ) -> DispatchResult {
        SwapMetadata::<T>::try_mutate((asset_0, asset_1), |meta|{
            ensure!(meta.is_some(), Error::<T>::PairNotExists);

            if let Some((pair_account,  total_liquidity)) = meta {
                let reserve_0 = T::MultiAssets::balance_of(asset_0, pair_account);
                let reserve_1 = T::MultiAssets::balance_of(asset_1, pair_account);

                let (amount_0, amount_1) = Self::calculate_added_amount(
                    amount_0_desired,
                    amount_1_desired,
                    amount_0_min,
                    amount_1_min,
                    reserve_0,
                    reserve_1,
                )?;

                let balance_asset_0 = T::MultiAssets::balance_of(asset_0, who);
                let balance_asset_1 = T::MultiAssets::balance_of(asset_1, who);
                ensure!(
                    balance_asset_0 >= amount_0 && balance_asset_1 >= amount_1,
                    Error::<T>::InsufficientAssetBalance
                );

                let mint_liquidity = Self::calculate_liquidity(
                    amount_0, amount_1, reserve_0, reserve_1, *total_liquidity
                );
                ensure!(mint_liquidity > Zero::zero(), Error::<T>::Overflow);

                *total_liquidity = total_liquidity.checked_add(mint_liquidity).ok_or(Error::<T>::Overflow)?;
                Self::mutate_liquidity(asset_0, asset_1, who, mint_liquidity, true)?;

                T::MultiAssets::transfer(asset_0, &who, &pair_account, amount_0)?;
                T::MultiAssets::transfer(asset_1, &who, &pair_account, amount_1)?;
            }

            Ok(())
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_remove_liquidity(
        who: &T::AccountId,
        asset_0: AssetId,
        asset_1: AssetId,
        remove_liquidity: AssetBalance,
        amount_token_0_min: AssetBalance,
        amount_token_1_min: AssetBalance,
        recipient: &T::AccountId,
    ) -> DispatchResult {
        ensure!(
            Self::swap_ledger(((asset_0, asset_1), who)) >= remove_liquidity,
            Error::<T>::InsufficientLiquidity
        );

        SwapMetadata::<T>::try_mutate((asset_0, asset_1), |meta|{
            ensure!(meta.is_some(), Error::<T>::PairNotExists);

            if let Some((pair_account,  total_liquidity)) = meta {
                let reserve_0 = T::MultiAssets::balance_of(asset_0, &pair_account);
                let reserve_1 = T::MultiAssets::balance_of(asset_1, &pair_account);

                let amount_0 =
                    Self::calculate_share_amount(remove_liquidity, *total_liquidity, reserve_0);
                let amount_1 =
                    Self::calculate_share_amount(remove_liquidity, *total_liquidity, reserve_1);

                ensure!(
                    amount_0 >= amount_token_0_min && amount_1 >= amount_token_1_min,
                    Error::<T>::InsufficientTargetAmount
                );

                *total_liquidity = total_liquidity.checked_sub(remove_liquidity).ok_or(Error::<T>::InsufficientLiquidity)?;
                Self::mutate_liquidity(asset_0, asset_1, who, remove_liquidity, false)?;

                T::MultiAssets::transfer(asset_0, &pair_account, recipient, amount_0)?;
                T::MultiAssets::transfer(asset_1, &pair_account, recipient, amount_1)?;
            }

            Ok(())
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_swap_exact_tokens_for_tokens(
        who: &T::AccountId,
        amount_in: AssetBalance,
        amount_out_min: AssetBalance,
        path: &[AssetId],
        recipient: &T::AccountId,
    ) -> DispatchResult {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_swap_tokens_for_exact_tokens(
        who: &T::AccountId,
        amount_out: AssetBalance,
        amount_in_max: AssetBalance,
        path: &[AssetId],
        recipient: &T::AccountId,
    ) -> DispatchResult {
        Ok(())
    }

    fn calculate_share_amount(
        amount_0: AssetBalance,
        reserve_0: AssetBalance,
        reserve_1: AssetBalance,
    ) -> AssetBalance {
        U256::from(amount_0)
            .saturating_mul(U256::from(reserve_1))
            .checked_div(U256::from(reserve_0))
            .and_then(|n| TryInto::<AssetBalance>::try_into(n).ok())
            .unwrap_or_else(Zero::zero)
    }

    fn calculate_liquidity(
        amount_0: AssetBalance,
        amount_1: AssetBalance,
        reserve_0: AssetBalance,
        reserve_1: AssetBalance,
        total_liquidity: AssetBalance,
    ) -> AssetBalance {
        if total_liquidity == Zero::zero() {
            amount_0.saturating_mul(amount_1).integer_sqrt()
        } else {
            core::cmp::min(
                Self::calculate_share_amount(amount_0, reserve_0, total_liquidity),
                Self::calculate_share_amount(amount_1, reserve_1, total_liquidity),
            )
        }
    }

    fn calculate_added_amount(
        amount_0_desired: AssetBalance,
        amount_1_desired: AssetBalance,
        amount_0_min: AssetBalance,
        amount_1_min: AssetBalance,
        reserve_0: AssetBalance,
        reserve_1: AssetBalance,
    ) -> Result<(AssetBalance, AssetBalance), DispatchError> {
        if reserve_0 == Zero::zero() || reserve_1 == Zero::zero() {
            return Ok((amount_0_desired, amount_1_desired));
        }
        let amount_1_optimal = Self::calculate_share_amount(amount_0_desired, reserve_0, reserve_1);
        if amount_1_optimal <= amount_1_desired {
            ensure!(amount_1_optimal >= amount_1_min, Error::<T>::IncorrectAssetAmountRange);
            return Ok((amount_0_desired, amount_1_optimal));
        }
        let amount_0_optimal = Self::calculate_share_amount(amount_1_desired, reserve_1, reserve_0);
        ensure!(
            amount_0_optimal >= amount_0_min && amount_0_optimal <= amount_0_desired,
            Error::<T>::IncorrectAssetAmountRange
        );
        Ok((amount_0_optimal, amount_1_desired))
    }

    fn mutate_liquidity(
        asset_0: AssetId,
        asset_1: AssetId,
        who: &T::AccountId,
        amount: AssetBalance,
        is_mint: bool
    ) -> DispatchResult {
        SwapLedger::<T>::try_mutate(((asset_0, asset_1), who), |liquidity|{
            if is_mint {
                *liquidity = liquidity.checked_add(amount).ok_or(Error::<T>::Overflow)?;
            } else {
                *liquidity = liquidity.checked_sub(amount).ok_or(Error::<T>::InsufficientLiquidity)?;;
            }

            Ok(())
        })
    }
}
