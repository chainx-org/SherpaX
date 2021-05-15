use super::*;
use crate::{mock::*, rpc::TokenInfo};
use frame_support::{assert_noop, assert_ok};
use xpallet_assets_registrar::{AssetInfo, Chain};

#[test]
fn create_new_pair_should_work() {
    ExtBuilder::default().build_and_execute(|| {
        match Swap::swap_metadata((PCX, X_BTC)) {
            None => assert!(true),
            _ => assert!(false),
        }
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        match Swap::swap_metadata((PCX, X_BTC)) {
            Some(_) => assert!(true),
            None => assert!(false),
        }
        let create_pair_event =
            mock::Event::pallet_swap(crate::Event::PairCreated(ALICE, PCX, X_BTC));

        assert!(System::events().iter().any(|record| { record.event == create_pair_event }));

        assert_noop!(
            Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC),
            crate::Error::<Test>::PairAlreadyExists
        );
    });
}

#[test]
fn add_liquidity_should_fail() {
    ExtBuilder::default().build_and_execute(|| {
        assert_noop!(
            Swap::add_liquidity(
                Origin::signed(ALICE),
                PCX,
                X_BTC,
                50_000_000,
                40_000_000,
                0,
                0,
                100
            ),
            crate::Error::<Test>::PairNotExists
        );
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_noop!(
            Swap::add_liquidity(
                Origin::signed(ALICE),
                PCX,
                X_BTC,
                5_000_000_000,
                4_000_000_000,
                0,
                0,
                100
            ),
            crate::Error::<Test>::InsufficientAssetBalance
        );
        assert_noop!(
            Swap::add_liquidity(Origin::signed(ALICE), PCX, X_BTC, 0, 40_000_000, 0, 0, 100),
            crate::Error::<Test>::Overflow
        );
    })
}

#[test]
fn add_liquidity_should_work() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));
        let add_liquidity_event =
            mock::Event::pallet_swap(crate::Event::LiquidityAdded(ALICE, PCX, X_BTC));
        assert!(System::events().iter().any(|record| record.event == add_liquidity_event));
        assert_eq!(
            Swap::swap_metadata((PCX, X_BTC))
                .map(|(_, total_liquidity_)| total_liquidity_)
                .unwrap(),
            44721359
        );
        assert_eq!(Swap::swap_ledger(((PCX, X_BTC), ALICE)), 44721359);
        assert_eq!(<Test as pallet::Config>::MultiAsset::balance_of(PCX, &ALICE), 9_950_000_000);
        assert_eq!(<Test as pallet::Config>::MultiAsset::balance_of(X_BTC, &ALICE), 960_000_000);

        let pair_account =
            Swap::swap_metadata((PCX, X_BTC)).map(|(pair_account_, _)| pair_account_).unwrap();
        let reserve_0 = <Test as pallet::Config>::MultiAsset::balance_of(PCX, &pair_account);
        let reserve_1 = <Test as pallet::Config>::MultiAsset::balance_of(X_BTC, &pair_account);
        assert_eq!(reserve_1, 40_000_000);
        assert_eq!(reserve_0, 50_000_000);

        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));

        assert_eq!(
            Swap::swap_metadata((PCX, X_BTC))
                .map(|(_, total_liquidity_)| total_liquidity_)
                .unwrap(),
            89442718
        );
    })
}

#[test]
fn remove_liquidity_should_fail() {
    ExtBuilder::default().build_and_execute(|| {
        assert_noop!(
            Swap::remove_liquidity(
                Origin::signed(ALICE),
                PCX,
                X_BTC,
                50_000_000,
                40_000_000,
                0,
                0,
                100
            ),
            crate::Error::<Test>::PairNotExists
        );
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(BOB),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));
        assert_noop!(
            Swap::remove_liquidity(
                Origin::signed(ALICE),
                PCX,
                X_BTC,
                100_000_000,
                0,
                0,
                ALICE,
                100
            ),
            crate::Error::<Test>::InsufficientLiquidity
        );
        assert_noop!(
            Swap::remove_liquidity(Origin::signed(ALICE), PCX, X_BTC, 60_000_000, 0, 0, ALICE, 100),
            crate::Error::<Test>::InsufficientLiquidity
        );
    })
}

#[test]
fn remove_liquidity_should_work() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(BOB),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));
        assert_ok!(Swap::remove_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            30_000_000,
            0,
            0,
            ALICE,
            100
        ));
        assert_eq!(
            Swap::swap_metadata((PCX, X_BTC))
                .map(|(_, total_liquidity_)| total_liquidity_)
                .unwrap(),
            59442718
        );
        assert_eq!(Swap::swap_ledger(((PCX, X_BTC), ALICE)), 14721359);

        let remove_liquidity_event = mock::Event::pallet_swap(crate::Event::LiquidityRemoved(
            ALICE, ALICE, PCX, X_BTC, 30_000_000,
        ));
        assert!(System::events().iter().any(|record| record.event == remove_liquidity_event));
    })
}

#[test]
fn swap_exact_tokens_for_tokens_should_fail() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));
        assert_noop!(
            Swap::swap_exact_tokens_for_tokens(
                Origin::signed(ALICE),
                3_000_000,
                5_000_000,
                vec![PCX, X_BTC],
                ALICE,
                100
            ),
            crate::Error::<Test>::InsufficientTargetAmount
        );

        assert_noop!(
            Swap::swap_exact_tokens_for_tokens(
                Origin::signed(ALICE),
                100_000_000,
                5_000_000,
                vec![PCX, X_ETH],
                ALICE,
                100
            ),
            crate::Error::<Test>::PairNotExists
        );
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_ETH));

        assert_noop!(
            Swap::swap_exact_tokens_for_tokens(
                Origin::signed(ALICE),
                100_000_000,
                5_000_000,
                vec![PCX, X_ETH],
                ALICE,
                100
            ),
            crate::Error::<Test>::InvalidPath
        );

        assert_noop!(
            Swap::swap_exact_tokens_for_tokens(
                Origin::signed(ALICE),
                0,
                5_000_000,
                vec![PCX, X_BTC],
                ALICE,
                100
            ),
            crate::Error::<Test>::InvalidPath
        );
    })
}

#[test]
fn swap_exact_tokens_for_tokens_should_work() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));

        assert_ok!(Swap::swap_exact_tokens_for_tokens(
            Origin::signed(ALICE),
            100_000_000,
            5_000_000,
            vec![PCX, X_BTC],
            ALICE,
            100
        ));
        let swap_event =
            mock::Event::pallet_swap(crate::Event::TokenSwap(ALICE, ALICE, vec![PCX, X_BTC]));
        assert!(System::events().iter().any(|record| record.event == swap_event));

        let pair_account =
            Swap::swap_metadata((PCX, X_BTC)).map(|(pair_account_, _)| pair_account_).unwrap();
        let reserve_0 = <Test as pallet::Config>::MultiAsset::balance_of(PCX, &pair_account);
        let reserve_1 = <Test as pallet::Config>::MultiAsset::balance_of(X_BTC, &pair_account);
        assert_eq!(reserve_0, 150_000_000);
        assert_eq!(reserve_1, 13360054);
    })
}

#[test]
fn swap_tokens_for_exact_tokens_should_fail() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));
        assert_noop!(
            Swap::swap_tokens_for_exact_tokens(
                Origin::signed(ALICE),
                100_000_000,
                5_000_000,
                vec![PCX, X_BTC],
                ALICE,
                100
            ),
            crate::Error::<Test>::InvalidPath
        );

        assert_noop!(
            Swap::swap_tokens_for_exact_tokens(
                Origin::signed(ALICE),
                20_000_000,
                5_000_000,
                vec![PCX, X_BTC],
                ALICE,
                100
            ),
            crate::Error::<Test>::ExcessiveSoldAmount
        );

        assert_noop!(
            Swap::swap_tokens_for_exact_tokens(
                Origin::signed(ALICE),
                30_000_000,
                50_000_000,
                vec![PCX, X_ETH],
                ALICE,
                100
            ),
            crate::Error::<Test>::PairNotExists
        );
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_ETH));

        assert_noop!(
            Swap::swap_tokens_for_exact_tokens(
                Origin::signed(ALICE),
                20_000_000,
                100_000_000,
                vec![PCX, X_ETH],
                ALICE,
                100
            ),
            crate::Error::<Test>::InvalidPath
        );
    })
}

#[test]
fn swap_tokens_for_exact_tokens_should_work() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));

        assert_ok!(Swap::swap_tokens_for_exact_tokens(
            Origin::signed(ALICE),
            20_000_000,
            100_000_000,
            vec![PCX, X_BTC],
            ALICE,
            100
        ));
        let swap_event =
            mock::Event::pallet_swap(crate::Event::TokenSwap(ALICE, ALICE, vec![PCX, X_BTC]));
        assert!(System::events().iter().any(|record| record.event == swap_event));

        let pair_account =
            Swap::swap_metadata((PCX, X_BTC)).map(|(pair_account_, _)| pair_account_).unwrap();
        let reserve_0 = <Test as pallet::Config>::MultiAsset::balance_of(PCX, &pair_account);
        let reserve_1 = <Test as pallet::Config>::MultiAsset::balance_of(X_BTC, &pair_account);
        assert_eq!(reserve_0, 100_150_451);
        assert_eq!(reserve_1, 20_000_000);
    })
}

#[test]
fn get_amount_out_price_work() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));
        assert_eq!(Swap::get_amount_out_price(10_000_000, vec![PCX, X_BTC]), 6649991)
    })
}

#[test]
fn get_amount_in_price_work() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));
        assert_eq!(Swap::get_amount_in_price(6649991, vec![PCX, X_BTC]), 9_999_998)
    })
}

#[test]
fn get_token_list_work() {
    ExtBuilder::default().build_and_execute(|| {
        assert_eq!(Swap::get_token_list(), vec![]);
        assert_ok!(Swap::create_pair(Origin::signed(ALICE), PCX, X_BTC));
        assert_eq!(Swap::get_token_list(), vec![]);

        assert_ok!(Swap::add_liquidity(
            Origin::signed(ALICE),
            PCX,
            X_BTC,
            50_000_000,
            40_000_000,
            0,
            0,
            100
        ));
        assert_eq!(
            Swap::get_token_list(),
            vec![
                TokenInfo {
                    assert_id: 0,
                    assert_info: AssetInfo::new::<Test>(
                        "PCX".into(),
                        "PCX".into(),
                        Chain::ChainX,
                        8u8,
                        "ChainX\'s PCX".into()
                    )
                    .unwrap()
                },
                TokenInfo {
                    assert_id: 1,
                    assert_info: AssetInfo::new::<Test>(
                        "X-BTC".into(),
                        "X-BTC".into(),
                        Chain::Bitcoin,
                        8u8,
                        "ChainX\'s cross-chain Bitcoin".into()
                    )
                    .unwrap()
                }
            ],
        );
    })
}

#[test]
fn get_balance_work() {
    ExtBuilder::default().build_and_execute(|| {
        assert_eq!(Swap::get_balance(PCX, ALICE), 10_000_000_000);
        assert_eq!(Swap::get_balance(PCX, BOB), 20_000_000_000);
        assert_eq!(Swap::get_balance(PCX, CHARLIE), 30_000_000_000);
        assert_eq!(Swap::get_balance(PCX, DAVE), 40_000_000_000);
        assert_eq!(Swap::get_balance(X_BTC, ALICE), 1_000_000_000);
        assert_eq!(Swap::get_balance(X_BTC, BOB), 2_000_000_000);
        assert_eq!(Swap::get_balance(X_BTC, CHARLIE), 3_000_000_000);
        assert_eq!(Swap::get_balance(X_BTC, DAVE), 4_000_000_000);
    })
}
