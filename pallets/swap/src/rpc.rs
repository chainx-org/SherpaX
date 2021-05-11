
use super::*;
impl<T: Config> Pallet<T> {
    pub fn supply_out_amount(supply: BalanceOf<T>, path: Vec<AssetId>) -> BalanceOf<T> {
        Self::get_amount_out_by_path(supply, &path).
            map_or(Default::default(), |amounts| *amounts.last().unwrap_or(&Default::default()))
    }

    pub fn desired_in_amount(desired_amount: BalanceOf<T>, path: Vec<AssetId>) -> BalanceOf<T> {
        Self::get_amount_in_by_path(desired_amount, &path)
            .map_or(Default::default(), |amounts| *amounts.first().unwrap_or(&Default::default()))
    }
}
