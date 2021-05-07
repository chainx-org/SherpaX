use chainx_primitives::AssetId;
use xp_assets_registrar::Chain;

pub trait CollateralT<Balance, AccountId> {
    const CHAIN: Chain;
    const TARGET_ASSET_ID: AssetId;
    const SHADOW_ASSET_ID: AssetId; /*Currently useless*/
    fn total() -> Balance;
    fn get(who: &AccountId) -> Balance;
    fn add(who: &AccountId, amount: Balance);
    fn sub(who: &AccountId, amount: Balance);
}
