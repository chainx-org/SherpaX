#![allow(unused_imports)]

use runtime_common::genesis_config::{
    generate_encode_files, balances_decode_all,
    vesting_decode_all, balances, AccountId, Balance
};
use runtime_common::genesis_config::{balances_decode_all_for_std, vesting_decode_all_for_std};

fn main() {
    let _ = generate_encode_files().unwrap();
}

#[test]
fn decode_all() {
    let _ = generate_encode_files().unwrap();
    let _ = vesting_decode_all_for_std().unwrap();
    let _ = balances_decode_all_for_std().unwrap();

    let total = balances()
        .unwrap()
        .into_iter()
        .flat_map(|s| s.balances)
        .map(|(_, b)| b)
        .sum::<u128>();

    assert_eq!(total, 10500000000000000000000000);
}
