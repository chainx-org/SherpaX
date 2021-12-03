pub use crate::{AccountId, Balance, BlockNumber};
use codec::{Decode, Encode};
use sp_runtime::traits::{AtLeast32Bit, UniqueSaturatedInto, Zero};
use sp_std::{vec, vec::Vec};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::io::Write;

#[derive(Clone, Eq, PartialEq, Encode, Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct SherpaXBalances<AccountId, Balance: AtLeast32Bit> {
    pub balances: Vec<(AccountId, Balance)>,
}

macro_rules! balance_decode {
    ($encode_file:literal, $total_accounts:expr, $total_balance:expr,) => {{
        let raw = include_bytes!($encode_file).to_vec();
        let config: SherpaXBalances<_, Balance> = codec::Decode::decode(&mut raw.as_ref())?;

        let total = config
            .balances
            .iter()
            .fold(Zero::zero(), |acc: Balance, &(_, n)| acc + n);

        assert_eq!($total_accounts, config.balances.len());
        assert_eq!($total_balance, total);

        config
    }};

    ($encode_file:literal, $total_accounts:expr, $total_balance:expr, $T:tt) => {{
        let raw = include_bytes!($encode_file).to_vec();
        let config: SherpaXBalances<_, $T::Balance> = codec::Decode::decode(&mut raw.as_ref())?;

        let total = config
            .balances
            .iter()
            .fold(Zero::zero(), |acc: $T::Balance, &(_, n)| acc + n);

        assert_eq!($total_accounts, config.balances.len());
        assert_eq!($total_balance, total.unique_saturated_into());

        config
    }};
}

pub fn balances_decode_all<T>(
) -> Result<Vec<SherpaXBalances<T::AccountId, T::Balance>>, codec::Error>
where
    T: pallet_balances::Config,
{
    Ok(vec![
        balance_decode!(
            "../../../migration/balances/non_dust_100ksx_airdrop_3686_10396654507485690000000000.encode",
            3686,
            10396654507485690000000000u128,
            T
        ),
    ])
}

#[cfg(feature = "std")]
pub fn balances_decode_all_for_std(
) -> Result<Vec<SherpaXBalances<AccountId, Balance>>, codec::Error> {
    Ok(vec![
        balance_decode!(
            "../../../migration/balances/non_dust_100ksx_airdrop_3686_10396654507485690000000000.encode",
            3686,
            10396654507485690000000000u128,
        )
    ])
}

#[cfg(feature = "std")]
macro_rules! balances {
    ($file:expr, $total_accounts:expr, $total_balance:expr) => {{
        let file = std::fs::File::open($file)
            .map_err(|e| format!("Error opening balances json file: {}", e))?;

        let config: SherpaXBalances<AccountId, Balance> = serde_json::from_reader(file)
            .map_err(|e| format!("Error parsing balances json file: {}", e))?;

        let total = config.balances.iter().map(|(_, b)| b).sum::<u128>();

        assert_eq!($total_accounts, config.balances.len());
        assert_eq!($total_balance, total);

        config
    }};

    ($file:expr, $encode_file:expr, $total_accounts:expr, $total_balance:expr) => {{
        let file = std::fs::File::open($file)
            .map_err(|e| format!("Error opening balances json file: {}", e))?;

        let config: SherpaXBalances<AccountId, Balance> = serde_json::from_reader(file)
            .map_err(|e| format!("Error parsing balances json file: {}", e))?;

        let total = config.balances.iter().map(|(_, b)| b).sum::<u128>();

        assert_eq!($total_accounts, config.balances.len());
        assert_eq!($total_balance, total);

        let encoded = codec::Encode::encode(&config);
        let mut file = std::fs::File::create($encode_file)
            .map_err(|e| format!("Error open encoded file: {}", e))?;

        file.write_all(&encoded)
            .map_err(|e| format!("Error write encoded data: {}", e))
    }};
}

#[cfg(feature = "std")]
pub fn balances() -> Result<Vec<SherpaXBalances<AccountId, Balance>>, String> {
    Ok(vec![
        balances!(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../migration/balances/non_dust_100ksx_airdrop_3686_10396654507485690000000000.json"
            ),
            3686,
            10396654507485690000000000
        )
    ])
}

#[cfg(feature = "std")]
pub fn generate_encode_files() -> Result<(), String> {
    balances!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migration/balances/non_dust_100ksx_airdrop_3686_10396654507485690000000000.json"
        ),
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migration/balances/non_dust_100ksx_airdrop_3686_10396654507485690000000000.encode"
        ),
        3686,
        10396654507485690000000000
    )?;

    Ok(())
}

#[test]
fn total_balances() {
    let total = balances()
        .unwrap()
        .into_iter()
        .flat_map(|s| s.balances)
        .map(|(_, b)| b)
        .sum::<u128>();

    assert_eq!(total, 10396654507485690000000000);
}

#[test]
fn decode_all_balances() {
    let _ = balances_decode_all_for_std().unwrap();
}
