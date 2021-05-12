// Copyright 2019-2021 ChainX Project Authors. Licensed under GPL-3.0.

//! # SWAP Module
//!
//! ## Overview
//!
//! Built-in decentralized exchange modules in SherpaX network, the swap
//! mechanism refers to the design of Uniswap V2.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::{
    inherent::Vec,
    pallet_prelude::*,
    traits::{Currency, Get},
    PalletId,
};

use sp_runtime::traits::{
    AccountIdConversion, CheckedAdd, CheckedDiv, CheckedSub, IntegerSquareRoot, One, Saturating,
    StaticLookup, Zero,
};

mod multiasset;
pub mod rpc;

pub use self::multiasset::{MultiAsset, SimpleMultiAsset};
pub use pallet::*;

pub type AssetId = u32;

pub type BalanceOf<T> = <<T as xpallet_assets::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::dispatch::DispatchResult;
    use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + xpallet_assets::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        ///
        type NativeAssetId: Get<AssetId>;
        ///
        type MultiAsset: MultiAsset<Self::AccountId, BalanceOf<Self>>;
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
    pub type SwapMetadata<T: Config> =
        StorageMap<_, Twox64Concat, (AssetId, AssetId), (T::AccountId, BalanceOf<T>)>;

    #[pallet::storage]
    #[pallet::getter(fn swap_ledger)]
    /// ((AssetId, AssetId), AccountId) -> BalanceOf<T>
    pub type SwapLedger<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ((AssetId, AssetId), T::AccountId),
        BalanceOf<T>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Create a trading pair. \[creator, asset_id, asset_id\]
        PairCreated(T::AccountId, AssetId, AssetId),
        /// Add liquidity. \[owner, asset_id, asset_id\]
        LiquidityAdded(T::AccountId, AssetId, AssetId),
        /// Remove liquidity. \[owner, recipient, asset_id, asset_id, amount\]
        LiquidityRemoved(T::AccountId, T::AccountId, AssetId, AssetId, BalanceOf<T>),
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
    impl<T: Config> Pallet<T> {
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
            asset_1: AssetId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(asset_0 != asset_1, Error::<T>::DeniedCreatePair);

            let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);

            let pair_account = Self::pair_account_id(asset_0, asset_1);

            SwapMetadata::<T>::try_mutate((asset_0, asset_1), |meta| {
                ensure!(meta.is_none(), Error::<T>::PairAlreadyExists);
                *meta = Some((pair_account, Default::default()));

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
            #[pallet::compact] amount_0_desired: BalanceOf<T>,
            #[pallet::compact] amount_1_desired: BalanceOf<T>,
            #[pallet::compact] amount_0_min: BalanceOf<T>,
            #[pallet::compact] amount_1_min: BalanceOf<T>,
            #[pallet::compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let now = frame_system::Pallet::<T>::block_number();
            let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_add_liquidity(
                &who,
                asset_0,
                asset_1,
                amount_0_desired,
                amount_1_desired,
                amount_0_min,
                amount_1_min,
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
            #[pallet::compact] liquidity: BalanceOf<T>,
            #[pallet::compact] amount_asset_0_min: BalanceOf<T>,
            #[pallet::compact] amount_asset_1_min: BalanceOf<T>,
            recipient: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let recipient = T::Lookup::lookup(recipient)?;
            let now = frame_system::Pallet::<T>::block_number();
            let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_remove_liquidity(
                &who,
                asset_0,
                asset_1,
                liquidity,
                amount_asset_0_min,
                amount_asset_1_min,
                &recipient,
            )?;

            Self::deposit_event(Event::LiquidityRemoved(
                who, recipient, asset_0, asset_1, liquidity,
            ));

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
            #[pallet::compact] amount_in: BalanceOf<T>,
            #[pallet::compact] amount_out_min: BalanceOf<T>,
            path: Vec<AssetId>,
            recipient: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let recipient = T::Lookup::lookup(recipient)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_swap_exact_tokens_for_tokens(
                &who,
                amount_in,
                amount_out_min,
                &path,
                &recipient,
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
            #[pallet::compact] amount_out: BalanceOf<T>,
            #[pallet::compact] amount_in_max: BalanceOf<T>,
            path: Vec<AssetId>,
            recipient: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let recipient = T::Lookup::lookup(recipient)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_swap_tokens_for_exact_tokens(
                &who,
                amount_out,
                amount_in_max,
                &path,
                &recipient,
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

    /// The account ID of a pair account from storage
    fn get_pair_account_id(asset_0: AssetId, asset_1: AssetId) -> Option<T::AccountId> {
        let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);

        Self::swap_metadata((asset_0, asset_1)).map(|(pair_account, _)| pair_account)
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
        amount_0_desired: BalanceOf<T>,
        amount_1_desired: BalanceOf<T>,
        amount_0_min: BalanceOf<T>,
        amount_1_min: BalanceOf<T>,
    ) -> DispatchResult {
        SwapMetadata::<T>::try_mutate((asset_0, asset_1), |meta| {
            ensure!(meta.is_some(), Error::<T>::PairNotExists);

            if let Some((pair_account, total_liquidity)) = meta {
                let reserve_0 = T::MultiAsset::balance_of(asset_0, pair_account);
                let reserve_1 = T::MultiAsset::balance_of(asset_1, pair_account);

                let (amount_0, amount_1) = Self::calculate_added_amount(
                    amount_0_desired,
                    amount_1_desired,
                    amount_0_min,
                    amount_1_min,
                    reserve_0,
                    reserve_1,
                )?;

                let balance_asset_0 = T::MultiAsset::balance_of(asset_0, who);
                let balance_asset_1 = T::MultiAsset::balance_of(asset_1, who);
                ensure!(
                    balance_asset_0 >= amount_0 && balance_asset_1 >= amount_1,
                    Error::<T>::InsufficientAssetBalance
                );

                let mint_liquidity = Self::calculate_liquidity(
                    amount_0,
                    amount_1,
                    reserve_0,
                    reserve_1,
                    *total_liquidity,
                );
                ensure!(mint_liquidity > Zero::zero(), Error::<T>::Overflow);

                *total_liquidity =
                    total_liquidity.checked_add(&mint_liquidity).ok_or(Error::<T>::Overflow)?;
                Self::mutate_liquidity(asset_0, asset_1, who, mint_liquidity, true)?;

                T::MultiAsset::transfer(asset_0, &who, &pair_account, amount_0)?;
                T::MultiAsset::transfer(asset_1, &who, &pair_account, amount_1)?;
            }

            Ok(())
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_remove_liquidity(
        who: &T::AccountId,
        asset_0: AssetId,
        asset_1: AssetId,
        remove_liquidity: BalanceOf<T>,
        amount_token_0_min: BalanceOf<T>,
        amount_token_1_min: BalanceOf<T>,
        recipient: &T::AccountId,
    ) -> DispatchResult {
        ensure!(
            Self::swap_ledger(((asset_0, asset_1), who)) >= remove_liquidity,
            Error::<T>::InsufficientLiquidity
        );

        SwapMetadata::<T>::try_mutate((asset_0, asset_1), |meta| {
            ensure!(meta.is_some(), Error::<T>::PairNotExists);

            if let Some((pair_account, total_liquidity)) = meta {
                let reserve_0 = T::MultiAsset::balance_of(asset_0, &pair_account);
                let reserve_1 = T::MultiAsset::balance_of(asset_1, &pair_account);

                let amount_0 =
                    Self::calculate_share_amount(remove_liquidity, *total_liquidity, reserve_0);
                let amount_1 =
                    Self::calculate_share_amount(remove_liquidity, *total_liquidity, reserve_1);

                ensure!(
                    amount_0 >= amount_token_0_min && amount_1 >= amount_token_1_min,
                    Error::<T>::InsufficientTargetAmount
                );

                *total_liquidity = total_liquidity
                    .checked_sub(&remove_liquidity)
                    .ok_or(Error::<T>::InsufficientLiquidity)?;
                Self::mutate_liquidity(asset_0, asset_1, who, remove_liquidity, false)?;

                T::MultiAsset::transfer(asset_0, &pair_account, recipient, amount_0)?;
                T::MultiAsset::transfer(asset_1, &pair_account, recipient, amount_1)?;
            }

            Ok(())
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_swap_exact_tokens_for_tokens(
        who: &T::AccountId,
        amount_in: BalanceOf<T>,
        amount_out_min: BalanceOf<T>,
        path: &[AssetId],
        recipient: &T::AccountId,
    ) -> DispatchResult {
        let amounts = Self::get_amount_out_by_path(amount_in, &path)?;
        ensure!(amounts[amounts.len() - 1] >= amount_out_min, Error::<T>::InsufficientTargetAmount);

        let pair_account =
            Self::get_pair_account_id(path[0], path[1]).ok_or(Error::<T>::PairNotExists)?;

        T::MultiAsset::transfer(path[0], who, &pair_account, amount_in)?;
        Self::swap(&amounts, &path, &recipient)?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_swap_tokens_for_exact_tokens(
        who: &T::AccountId,
        amount_out: BalanceOf<T>,
        amount_in_max: BalanceOf<T>,
        path: &[AssetId],
        recipient: &T::AccountId,
    ) -> DispatchResult {
        let amounts = Self::get_amount_in_by_path(amount_out, &path)?;

        ensure!(amounts[0] <= amount_in_max, Error::<T>::ExcessiveSoldAmount);

        let pair_account =
            Self::get_pair_account_id(path[0], path[1]).ok_or(Error::<T>::PairNotExists)?;

        T::MultiAsset::transfer(path[0], who, &pair_account, amounts[0])?;
        Self::swap(&amounts, &path, recipient)?;

        Ok(())
    }

    fn calculate_share_amount(
        amount_0: BalanceOf<T>,
        reserve_0: BalanceOf<T>,
        reserve_1: BalanceOf<T>,
    ) -> BalanceOf<T> {
        amount_0.saturating_mul(reserve_1).checked_div(&reserve_0).unwrap_or_default()
    }

    fn calculate_liquidity(
        amount_0: BalanceOf<T>,
        amount_1: BalanceOf<T>,
        reserve_0: BalanceOf<T>,
        reserve_1: BalanceOf<T>,
        total_liquidity: BalanceOf<T>,
    ) -> BalanceOf<T> {
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
        amount_0_desired: BalanceOf<T>,
        amount_1_desired: BalanceOf<T>,
        amount_0_min: BalanceOf<T>,
        amount_1_min: BalanceOf<T>,
        reserve_0: BalanceOf<T>,
        reserve_1: BalanceOf<T>,
    ) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
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
        amount: BalanceOf<T>,
        is_mint: bool,
    ) -> DispatchResult {
        SwapLedger::<T>::try_mutate(((asset_0, asset_1), who), |liquidity| {
            if is_mint {
                *liquidity = liquidity.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
                // liquidity.saturating_add(amount).ok_or(Error::<T>::Overflow)?;
            } else {
                *liquidity =
                    liquidity.checked_sub(&amount).ok_or(Error::<T>::InsufficientLiquidity)?;
            }
            Ok(())
        })
    }

    fn get_amount_in(
        output_amount: BalanceOf<T>,
        input_reserve: BalanceOf<T>,
        output_reserve: BalanceOf<T>,
    ) -> BalanceOf<T> {
        // See primitives/arithmetic/fuzzer/src/multiply_by_rational.rs

        let numerator = input_reserve.saturating_mul(output_amount).saturating_mul(1000u32.into());

        let denominator =
            output_reserve.saturating_sub(output_amount).saturating_mul(997u32.into());

        numerator.checked_div(&denominator).unwrap_or_default()
    }

    fn get_amount_out(
        input_amount: BalanceOf<T>,
        input_reserve: BalanceOf<T>,
        output_reserve: BalanceOf<T>,
    ) -> BalanceOf<T> {
        let input_amount_with_fee = input_amount.saturating_mul(997u32.into());
        let numerator = input_amount_with_fee.saturating_mul(output_reserve);

        let denominator =
            input_reserve.saturating_mul(1000u32.into()).saturating_add(input_amount_with_fee);

        numerator.checked_div(&denominator).unwrap_or_default()
    }

    fn get_amount_in_by_path(
        amount_out: BalanceOf<T>,
        path: &[AssetId],
    ) -> Result<Vec<BalanceOf<T>>, DispatchError> {
        let len = path.len();
        ensure!(len > 1, Error::<T>::InvalidPath);

        let mut i = len - 1;
        let mut out_vec = Vec::new();
        out_vec.push(amount_out);

        while i > 0 {
            let pair_account = Self::pair_account_id(path[i], path[i - 1]);
            let reserve_0 = T::MultiAsset::balance_of(path[i], &pair_account);
            let reserve_1 = T::MultiAsset::balance_of(path[i - 1], &pair_account);

            ensure!(reserve_1 > Zero::zero() && reserve_0 > Zero::zero(), Error::<T>::InvalidPath);

            let amount = Self::get_amount_in(out_vec[len - 1 - i], reserve_1, reserve_0);
            ensure!(amount > One::one(), Error::<T>::InvalidPath);

            out_vec.push(amount);
            i -= 1;
        }

        out_vec.reverse();
        Ok(out_vec)
    }

    fn get_amount_out_by_path(
        amount_in: BalanceOf<T>,
        path: &[AssetId],
    ) -> Result<Vec<BalanceOf<T>>, DispatchError> {
        ensure!(path.len() > 1, Error::<T>::InvalidPath);

        let len = path.len() - 1;
        let mut out_vec = Vec::new();
        out_vec.push(amount_in);

        for i in 0..len {
            let pair_account = Self::pair_account_id(path[i], path[i + 1]);
            let reserve_0 = T::MultiAsset::balance_of(path[i], &pair_account);
            let reserve_1 = T::MultiAsset::balance_of(path[i + 1], &pair_account);

            ensure!(reserve_1 > Zero::zero() && reserve_0 > Zero::zero(), Error::<T>::InvalidPath);

            let amount = Self::get_amount_out(out_vec[i], reserve_0, reserve_1);
            ensure!(amount > Zero::zero(), Error::<T>::InvalidPath);
            out_vec.push(amount);
        }

        Ok(out_vec)
    }

    fn swap(
        amounts: &[BalanceOf<T>],
        path: &[AssetId],
        recipient: &T::AccountId,
    ) -> DispatchResult {
        for i in 0..(amounts.len() - 1) {
            let input = path[i];
            let output = path[i + 1];
            let mut amount0_out: BalanceOf<T> = Default::default();
            let mut amount1_out = amounts[i + 1];

            let (asset_0, asset_1) = Self::sort_asset_id(input, output);
            if input != asset_0 {
                amount0_out = amounts[i + 1];
                amount1_out = Default::default();
            }
            let pair_account =
                Self::get_pair_account_id(asset_0, asset_1).ok_or(Error::<T>::PairNotExists)?;

            if i < (amounts.len() - 2) {
                let mid_account = Self::get_pair_account_id(output, path[i + 2])
                    .ok_or(Error::<T>::PairNotExists)?;
                Self::pair_swap(
                    asset_0,
                    asset_1,
                    &pair_account,
                    amount0_out,
                    amount1_out,
                    &mid_account,
                )?;
            } else {
                Self::pair_swap(
                    asset_0,
                    asset_1,
                    &pair_account,
                    amount0_out,
                    amount1_out,
                    &recipient,
                )?;
            };
        }
        Ok(())
    }

    fn pair_swap(
        asset_0: AssetId,
        asset_1: AssetId,
        pair_account: &T::AccountId,
        amount_0: BalanceOf<T>,
        amount_1: BalanceOf<T>,
        recipient: &T::AccountId,
    ) -> DispatchResult {
        let reserve_0 = T::MultiAsset::balance_of(asset_0, &pair_account);
        let reserve_1 = T::MultiAsset::balance_of(asset_1, &pair_account);

        ensure!(
            amount_0 <= reserve_0 && amount_1 <= reserve_1,
            Error::<T>::InsufficientPairReserve
        );

        if amount_0 > Zero::zero() {
            T::MultiAsset::transfer(asset_0, &pair_account, recipient, amount_0)?;
        }

        if amount_1 > Zero::zero() {
            T::MultiAsset::transfer(asset_1, &pair_account, recipient, amount_1)?;
        }

        Ok(())
    }
}
