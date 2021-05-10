use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};


#[test]
fn test_normal_case() {
	let asset_id1:u32 = 1;
	let asset_id2:u32 = 2;
	ExtBuilder::default()
		.build_default()
		.execute_with(|| {
			let amount_pcx_desired = 500_000;
			let amount_btc_desired = 50_000;
			let amount_pcx_min = 0;
			let amount_x_btc_min = 0;
			let deadline: BlockNumber = 10;

			assert_eq!(Swap::swap_metadata((PCX, X_BTC)).is_some(), false);
			assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
			assert_eq!(Swap::swap_metadata((PCX, X_BTC)).is_some(), true);

			assert_ok!(Swap::add_liquidity(
				Origin::signed(ALICE), PCX, X_BTC,
				amount_pcx_desired, amount_btc_desired,
				amount_pcx_min, amount_x_btc_min,
				deadline.into()));

			let liquidity = 1000;
			assert_ok!(Swap::remove_liquidity(
				Origin::signed(ALICE), PCX, X_BTC,
				liquidity,
				amount_pcx_min,
				amount_x_btc_min,
				ALICE.into(),
				deadline.into()));

			let amount_in = 100;
			let amount_out_min = 0;
			let path = vec![PCX, X_BTC];
			assert_ok!(Swap::swap_exact_tokens_for_tokens(
				Origin::signed(ALICE),
				amount_in,
				amount_out_min,
				path,
				ALICE.into(),
				deadline.into()));

			let amount_out = 100;
			let amount_in_max = 10_000;
			let path = vec![PCX, X_BTC];
			assert_ok!(Swap::swap_tokens_for_exact_tokens(
				Origin::signed(ALICE),
				amount_out,
				amount_in_max,
				path,
				ALICE.into(),
				deadline.into()));
		});
}

