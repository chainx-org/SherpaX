use sp_std::marker::PhantomData;

use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive, Get};

use crate::{AssetId, BalanceOf, Config, DispatchError};

pub trait MultiAsset<AccountId, Balance: Copy> {
    fn balance_of(asset_id: AssetId, who: &AccountId) -> Balance;

    fn total_supply(asset_id: AssetId) -> Balance;

    fn transfer(
        asset_id: AssetId,
        origin: &AccountId,
        target: &AccountId,
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
            <xpallet_assets::Module<T>>::move_usable_balance(&asset_id, from, to, amount)
                .map_err(|_| DispatchError::Other("Unexpected error from XAssets Module"))?;
        }
        Ok(amount)
    }
}
