
use super::{
    DispatchError, BaseArithmetic, MaybeSerializeDeserialize, Debug, Parameter, Member
};


pub trait MultiAsset<AccountId> {
    type AssetId: Copy + Parameter + Member + MaybeSerializeDeserialize + Debug + Ord;
    type AssetBalance: Default + BaseArithmetic + Copy + Parameter + Member + MaybeSerializeDeserialize + Debug + Ord;

    fn balance_of(
        asset_id: &Self::AssetId,
        who: &AccountId
    ) -> Self::AssetBalance;

    fn total_supply(
        asset_id: &Self::AssetId
    ) -> Self::AssetBalance;

    fn transfer(
        asset_id: &Self::AssetId,
        origin: &AccountId,
        target: &AccountId,
        amount: &Self::AssetBalance
    ) -> Result<Self::AssetBalance, DispatchError> {
        let withdrawn = Self::withdraw(asset_id, origin, amount)?;
        Self::deposit(asset_id, target, amount)?;

        Ok(withdrawn)
    }

    fn deposit(
        asset_id: &Self::AssetId,
        target: &AccountId,
        amount: &Self::AssetBalance
    ) -> Result<Self::AssetBalance, DispatchError>;

    fn withdraw(
        asset_id: &Self::AssetId,
        origin: &AccountId,
        amount: &Self::AssetBalance
    ) -> Result<Self::AssetBalance, DispatchError>;
}
