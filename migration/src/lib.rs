#[test]
fn decode_all() {
    use runtime_common::genesis_config::{balances, balances_decode_all_for_std};

    let _ = balances_decode_all_for_std().unwrap();

    let total = balances()
        .unwrap()
        .into_iter()
        .flat_map(|s| s.balances)
        .map(|(_, b)| b)
        .sum::<u128>();

    // more than 100 ksx
    assert_eq!(total, 10396654507485690000000000);
}
