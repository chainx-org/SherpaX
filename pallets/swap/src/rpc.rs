use super::*;
use sp_std::collections::btree_set::BTreeSet;
use xpallet_assets_registrar::AssetInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TokenInfo {
    assert_id: AssetId,
    assert_info: AssetInfo,
}

impl<T: Config> Pallet<T> {
    pub fn get_amount_out_price(amount_in: BalanceOf<T>, path: Vec<AssetId>) -> BalanceOf<T> {
        Self::get_amount_out_by_path(amount_in, &path)
            .map_or(Zero::zero(), |amounts| *amounts.last().unwrap_or(&Zero::zero()))
    }

    pub fn get_amount_in_price(amount_out: BalanceOf<T>, path: Vec<AssetId>) -> BalanceOf<T> {
        Self::get_amount_in_by_path(amount_out, &path)
            .map_or(Zero::zero(), |amounts| *amounts.first().unwrap_or(&Zero::zero()))
    }

    pub fn get_token_list() -> Vec<TokenInfo> {
        let mut all_assert: BTreeSet<AssetId> = BTreeSet::new();
        SwapMetadata::<T>::iter()
            .filter(|(_, (_, total_liquidity))| *total_liquidity > Zero::zero())
            .for_each(|((asset_0, asset_1), _)| {
                all_assert.insert(asset_0);
                all_assert.insert(asset_1);
            });

        all_assert
            .iter()
            .map(|&assert_id| TokenInfo {
                assert_id,
                assert_info: <xpallet_assets_registrar::Module<T>>::get_asset_info(&assert_id)
                    .unwrap(),
            })
            .collect::<Vec<_>>()
    }

    pub fn get_balance(asset_id: AssetId, account: T::AccountId) -> BalanceOf<T>{
        T::MultiAsset::balance_of(asset_id, &account)
    }
}
