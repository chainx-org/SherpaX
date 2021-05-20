use super::*;
use sp_std::vec;
use xpallet_assets_registrar::AssetInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TokenInfo {
    asset_id: AssetId,
    asset_info: AssetInfo,
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
        let mut unique_asset_ids: Vec<AssetId> = SwapMetadata::<T>::iter()
            .filter(|(_, (_, total_liquidity))| !total_liquidity.is_zero())
            .map(|((asset_0, asset_1), _)| vec![asset_0, asset_1])
            .flatten()
            .collect();

        unique_asset_ids.sort();
        unique_asset_ids.dedup();

        unique_asset_ids
            .into_iter()
            .filter_map(|asset_id| {
                <xpallet_assets_registrar::Module<T>>::get_asset_info(&asset_id)
                    .map(|asset_info| TokenInfo { asset_id, asset_info })
                    .ok()
            })
            .collect::<Vec<_>>()
    }

    pub fn get_balance(asset_id: AssetId, account: T::AccountId) -> BalanceOf<T> {
        T::MultiAsset::balance_of(asset_id, &account)
    }
}
