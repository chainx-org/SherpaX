
use super::{
    DispatchError, AssetId, AssetBalance
};


pub trait MultiAsset<AccountId> {
    fn balance_of(
        asset_id: AssetId,
        who: &AccountId
    ) -> AssetBalance;

    fn total_supply(
        asset_id: AssetId
    ) -> AssetBalance;

    fn transfer(
        asset_id: AssetId,
        origin: &AccountId,
        target: &AccountId,
        amount: AssetBalance
    ) -> Result<AssetBalance, DispatchError> {
        let withdrawn = Self::withdraw(asset_id, origin, amount)?;
        Self::deposit(asset_id, target, amount)?;

        Ok(withdrawn)
    }

    fn deposit(
        asset_id: AssetId,
        target: &AccountId,
        amount: AssetBalance
    ) -> Result<AssetBalance, DispatchError>;

    fn withdraw(
        asset_id: AssetId,
        origin: &AccountId,
        amount: AssetBalance
    ) -> Result<AssetBalance, DispatchError>;
}
