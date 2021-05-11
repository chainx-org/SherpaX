
use super::*;
impl<T: Config> Pallet<T> {
    pub fn supply_out_amount(supply: u128, path: Vec<AssetId>) -> u128 {
        todo!("Self::get_amount_out_by_path(supply, &path).map_or(0u128, |amounts| *amounts.last().unwrap_or(0u128))")
    }

    pub fn desired_in_amount(desired_amount: u128, path: Vec<AssetId>) -> u128 {
        todo!("Self::get_amount_in_by_path(desired_amount, &path).map_or(0u128, |amounts| *amounts.first().unwrap_or(0u128))")
    }
}
