use sp_std::marker::PhantomData;

use frame_support::traits::{
    ExistenceRequirement::{AllowDeath, KeepAlive},
    WithdrawReasons,
    Currency, Get, ReservableCurrency
};

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
            <xpallet_assets::Module<T>>::usable_balance(who, &asset_id)
        }
    }

    fn total_supply(asset_id: AssetId) -> BalanceOf<T> {
     if <T as Config>::NativeAssetId::get() == asset_id {
            <T as xpallet_assets::Config>::Currency::total_issuance()
        } else {
            <xpallet_assets::Module<T>>::total_issuance(&asset_id)
        }
    }

    fn transfer(
        asset_id: AssetId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if <T as Config>::NativeAssetId::get() == asset_id {
            <T as xpallet_assets::Config>::Currency::transfer(from, to, amount, KeepAlive)?;
        } else {
            <xpallet_assets::Module<T>>::can_transfer(&asset_id)?;
            <xpallet_assets::Module<T>>::move_usable_balance(&asset_id, from, to,amount).
                map_err(|_| DispatchError::Other("Unexpected error from XAssets Module"))?;
        }
        Ok(amount)
    }

    fn deposit(
        asset_id: AssetId,
        who: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if <T as Config>::NativeAssetId::get() == asset_id {
            <T as xpallet_assets::Config>::Currency::deposit_creating(who, amount);
        } else {
            <xpallet_assets::Module<T>>::issue(&asset_id, who, amount)?;
        }
        Ok(amount)
    }

    fn withdraw(
        asset_id: AssetId,
        who: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if <T as Config>::NativeAssetId::get() == asset_id {
            <T as xpallet_assets::Config>::Currency::withdraw(who, amount, WithdrawReasons::TRANSFER, AllowDeath)?;
        } else {
            <xpallet_assets::Module<T>>::destroy_reserved_withdrawal(&asset_id, who, amount)?;
        }
        Ok(amount)
    }
}
