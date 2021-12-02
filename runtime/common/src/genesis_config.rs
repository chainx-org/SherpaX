pub use crate::{AccountId, Balance, BlockNumber};
use codec::{Decode, Encode};
use sp_runtime::traits::{AtLeast32Bit, UniqueSaturatedInto, Zero};
use sp_std::{vec, vec::Vec};
use frame_support::traits::Currency;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::io::Write;

type BalanceOf<T> = <<T as pallet_vesting::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Eq, PartialEq, Encode, Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct SherpaXBalances<AccountId, Balance: AtLeast32Bit> {
    pub balances: Vec<(AccountId, Balance)>,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct SherpaXVesting<AccountId, BlockNumber, Balance: AtLeast32Bit> {
    // * who - Account which we are generating vesting configuration for
    // * begin - Block when the account will start to vest
    // * length - Number of blocks from `begin` until fully vested
    // * liquid - Number of units which can be spent before vesting begins
    pub vesting: Vec<(AccountId, BlockNumber, BlockNumber, Balance)>,
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

macro_rules! vesting_decode {
    ($encode_file:literal, $total_accounts:expr, $total_liquid:expr,) => {{
        let raw = include_bytes!($encode_file).to_vec();
        let config: SherpaXVesting<_, _, Balance> = codec::Decode::decode(&mut raw.as_ref())?;

        let vesting_liquid = config
            .vesting
            .iter()
            .fold(Zero::zero(), |acc: Balance, &(_, _, _, n)| acc + n);

        assert_eq!($total_accounts, config.vesting.len());
        assert_eq!($total_liquid, vesting_liquid);

        config
    }};
    ($encode_file:literal, $total_accounts:expr, $total_liquid:expr, $T:tt) => {{
        let raw = include_bytes!($encode_file).to_vec();
        let config: SherpaXVesting<_, _, BalanceOf<$T>> = codec::Decode::decode(&mut raw.as_ref())?;

        let vesting_liquid = config
            .vesting
            .iter()
            .fold(Zero::zero(), |acc: BalanceOf<$T>, &(_, _, _, n)| acc + n);

        assert_eq!($total_accounts, config.vesting.len());
        assert_eq!($total_liquid, vesting_liquid.unique_saturated_into());

        config
    }};
}

pub fn balances_decode_all<T>() -> Result<Vec<SherpaXBalances<T::AccountId, T::Balance>>, codec::Error>
where
    T: pallet_balances::Config
{
    Ok(vec![
        balance_decode!(
            "../../../migration/balances/dust_airdrop_10747_826282235120000000000.encode",
            10747,
            826282235120000000000u128,
            T
        ),
        balance_decode!(
            "../../../migration/balances/non_dust_airdrop_7418_10499173717764880000000000.encode",
            7418,
            10499173717764880000000000u128,
            T
        ),
    ])
}

pub fn vesting_decode_all<T>() -> Result<Vec<SherpaXVesting<T::AccountId, T::BlockNumber, BalanceOf<T>>>, codec::Error>
where
    T: pallet_vesting::Config
{
    Ok(vec![
        vesting_decode!(
            "../../../migration/vesting/vesting_airdrop_7417_943235795035215000000000.encode",
            7417,
            943235795035215000000000u128,
            T
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
macro_rules! vesting {
    ($file:expr, $total_accounts:expr, $total_liquid:expr) => {{
        let file = std::fs::File::open($file)
            .map_err(|e| format!("Error opening vesting json file: {}", e))?;

        let config: SherpaXVesting<AccountId, BlockNumber, Balance> = serde_json::from_reader(file)
            .map_err(|e| format!("Error parsing vesting json file: {}", e))?;

        let vesting_liquid = config
            .vesting
            .iter()
            .map(|(_, _, _, liquid)| liquid)
            .sum::<u128>();

        assert_eq!($total_accounts, config.vesting.len());
        assert_eq!($total_liquid, vesting_liquid);

        config
    }};
    ($file:expr, $encode_file:expr, $total_accounts:expr, $total_liquid:expr) => {{
        let file = std::fs::File::open($file)
            .map_err(|e| format!("Error opening vesting json file: {}", e))?;

        let config: SherpaXVesting<AccountId, BlockNumber, Balance> = serde_json::from_reader(file)
            .map_err(|e| format!("Error parsing vesting json file: {}", e))?;

        let vesting_liquid = config
            .vesting
            .iter()
            .map(|(_, _, _, liquid)| liquid)
            .sum::<u128>();

        assert_eq!($total_accounts, config.vesting.len());
        assert_eq!($total_liquid, vesting_liquid);

        let encoded = codec::Encode::encode(&config);
        let mut file =
            std::fs::File::create($encode_file).map_err(|e| format!("Error open file: {}", e))?;

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
                "/../../migration/balances/dust_airdrop_10747_826282235120000000000.json"
            ),
            10747,
            826282235120000000000
        ),
        balances!(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../migration/balances/non_dust_airdrop_7418_10499173717764880000000000.json"
            ),
            7418,
            10499173717764880000000000
        ),
    ])
}

#[cfg(feature = "std")]
pub fn vesting() -> Result<Vec<SherpaXVesting<AccountId, BlockNumber, Balance>>, String> {
    Ok(vec![vesting!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migration/vesting/vesting_airdrop_7417_943235795035215000000000.json"
        ),
        7417,
        943235795035215000000000
    )])
}

#[cfg(feature = "std")]
pub fn balances_decode_all_for_std() -> Result<Vec<SherpaXBalances<AccountId, Balance>>, codec::Error> {
    Ok(vec![
        balance_decode!(
            "../../../migration/balances/dust_airdrop_10747_826282235120000000000.encode",
            10747,
            826282235120000000000u128,
        ),
        balance_decode!(
            "../../../migration/balances/non_dust_airdrop_7418_10499173717764880000000000.encode",
            7418,
            10499173717764880000000000u128,
        ),
    ])
}

#[cfg(feature = "std")]
pub fn vesting_decode_all_for_std() -> Result<Vec<SherpaXVesting<AccountId, BlockNumber, Balance>>, codec::Error>
{
    Ok(vec![
        vesting_decode!(
            "../../../migration/vesting/vesting_airdrop_7417_943235795035215000000000.encode",
            7417,
            943235795035215000000000u128,
        )
    ])
}

#[cfg(feature = "std")]
pub fn generate_encode_files() -> Result<(), String> {
    balances!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migration/balances/dust_airdrop_10747_826282235120000000000.json"
        ),
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migration/balances/dust_airdrop_10747_826282235120000000000.encode"
        ),
        10747,
        826282235120000000000
    )?;

    balances!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migration/balances/non_dust_airdrop_7418_10499173717764880000000000.json"
        ),
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migration/balances/non_dust_airdrop_7418_10499173717764880000000000.encode"
        ),
        7418,
        10499173717764880000000000
    )?;

    vesting!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migration/vesting/vesting_airdrop_7417_943235795035215000000000.json"
        ),
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migration/vesting/vesting_airdrop_7417_943235795035215000000000.encode"
        ),
        7417,
        943235795035215000000000
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

    assert_eq!(total, 10500000000000000000000000);
}

#[test]
fn decode_all_balances() {
    let _ = balances_decode_all_for_std().unwrap();
}
