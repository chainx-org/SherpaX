use sp_std::marker::PhantomData;

use frame_support::traits::{Currency, Get, ReservableCurrency};

use crate::{AssetId, BalanceOf, Config, DispatchError, Pallet};

pub trait MultiAsset<AccountId, Balance: Copy> {
    fn balance_of(asset_id: AssetId, who: &AccountId) -> Balance;

    fn total_supply(asset_id: AssetId) -> Balance;

    fn transfer(
        asset_id: AssetId,
        origin: &AccountId,
        target: &AccountId,
        amount: Balance,
    ) -> Result<Balance, DispatchError> {
        let withdrawn = Self::withdraw(asset_id, origin, amount)?;
        Self::deposit(asset_id, target, amount)?;

        Ok(withdrawn)
    }

    fn deposit(
        asset_id: AssetId,
        target: &AccountId,
        amount: Balance,
    ) -> Result<Balance, DispatchError>;

    fn withdraw(
        asset_id: AssetId,
        origin: &AccountId,
        amount: Balance,
    ) -> Result<Balance, DispatchError>;
}

pub struct SimpleMultiAsset<T>(PhantomData<T>);

impl<T: Config> MultiAsset<T::AccountId, BalanceOf<T>> for SimpleMultiAsset<T> {
    fn balance_of(asset_id: AssetId, who: &T::AccountId) -> BalanceOf<T> {
        if <T as Config>::NativeAssetId::get() == asset_id {
            <T as xpallet_assets::Config>::Currency::free_balance(who)
        } else {
            todo!("XAssets::free_balance(AssetId::convert(asset_id), who).into()")
        }
    }

    fn total_supply(asset_id: AssetId) -> BalanceOf<T> {
        if <T as Config>::NativeAssetId::get() == asset_id {
            todo!("Balances::total_issuance(who)")
        } else {
            todo!("XAssets::total_issuance(AssetId::convert(asset_id), who)")
        }
    }

    fn transfer(
        asset_id: AssetId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if <T as Config>::NativeAssetId::get() == asset_id {
            todo!("Balances::transfer(from, to, amount, KeepAlive)")
        } else {
            todo!("XAssets::transfer(asset_id, from, to, amount)")
        }
    }

    fn deposit(
        asset_id: AssetId,
        who: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if <T as Config>::NativeAssetId::get() == asset_id {
            todo!("Balances::deposit_creating(who)")
        } else {
            todo!("XAssets::issue(asset_id, who)")
        }
    }

    fn withdraw(
        asset_id: AssetId,
        who: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if <T as Config>::NativeAssetId::get() == asset_id {
            todo!("Balances::withdraw(origin, amount, WithdrawReasons::TRANSFER, AllowDeath)")
        } else {
            todo!("XAssets::withdraw(asset_id, who)")
        }
    }
}
