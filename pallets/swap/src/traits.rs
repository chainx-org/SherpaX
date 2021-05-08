
use super::{
    DispatchError, BaseArithmetic, MaybeSerializeDeserialize, FullCodec, Debug
};


pub trait MultiAsset<AccountId> {
    type AssetId: Copy + FullCodec + Eq + PartialEq + MaybeSerializeDeserialize + Debug;
    type AssetBalance: Default + BaseArithmetic + Copy + FullCodec + Eq + PartialEq + MaybeSerializeDeserialize + Debug;

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
