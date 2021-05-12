use super::*;
use sp_std::collections::btree_set::BTreeSet;
use xpallet_assets_registrar::AssetInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Token {
    assert_id: AssetId,
    assert_info: AssetInfo,
}

impl<T: Config> Pallet<T> {
    pub fn supply_out_amount(supply: BalanceOf<T>, path: Vec<AssetId>) -> BalanceOf<T> {
        Self::get_amount_out_by_path(supply, &path)
            .map_or(Zero::zero(), |amounts| *amounts.last().unwrap_or(&Zero::zero()))
    }

    pub fn desired_in_amount(desired_amount: BalanceOf<T>, path: Vec<AssetId>) -> BalanceOf<T> {
        Self::get_amount_in_by_path(desired_amount, &path)
            .map_or(Zero::zero(), |amounts| *amounts.first().unwrap_or(&Zero::zero()))
    }

    pub fn get_token_list() -> Vec<Token> {
        let mut all_assert: BTreeSet<AssetId> = BTreeSet::new();
        SwapMetadata::<T>::iter()
            .filter(|(_, (_, total_liquidity))| *total_liquidity > Zero::zero())
            .for_each(|((asset_0, asset_1), _)| {
                all_assert.insert(asset_0);
                all_assert.insert(asset_1);
            });

        all_assert
            .iter()
            .map(|&assert_id| Token {
                assert_id,
                assert_info: <xpallet_assets_registrar::Module<T>>::get_asset_info(&assert_id)
                    .unwrap(),
            })
            .collect::<Vec<_>>()
    }
}
