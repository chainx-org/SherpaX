use crate::mock::*;
use crate::{to_ascii_hex, EcdsaSignature};
use frame_support::{assert_noop, assert_ok};
use sp_core::{H160, U256};

use ethabi::{Function, Param, ParamType, Token};
use hex_literal::hex;
use std::str::FromStr;

/*
{
  "address": "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
  "msg": "evm:d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
  "sig": "0x7def4e5806b7cf5dbfa44bc9d14422462dc9fe803c74e5d544db71bdcefc8ba04fc54cd079f2f8a2947f4d3b1c0d9e9f12fa279f6a40828ecc08766b4bab4bb21c",
  "version": "2"
}
*/
const SIGNATURE: [u8; 65] = hex!["7def4e5806b7cf5dbfa44bc9d14422462dc9fe803c74e5d544db71bdcefc8ba04fc54cd079f2f8a2947f4d3b1c0d9e9f12fa279f6a40828ecc08766b4bab4bb21c"];
const EVM_ADDR: [u8; 20] = hex!["f24ff3a9cf04c71dbc94d0b566f7a27b94566cac"];
const SUB_ACCOUNT: &str = "5USGSZK3raH3LD4uxvNTa23HN5VULnYrkXonRktyizTJUYg9";
const PUBKEY: &str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const ERC20_1: [u8; 20] = [1u8; 20];
const ERC20_2: [u8; 20] = [2u8; 20];

pub fn mint_into_abi() -> Function {
    #[allow(deprecated)]
    Function {
        name: "mint_into".to_owned(),
        inputs: vec![
            Param {
                name: "account".to_owned(),
                kind: ParamType::Address,
                internal_type: None,
            },
            Param {
                name: "amount".to_owned(),
                kind: ParamType::Uint(256),
                internal_type: None,
            },
        ],
        outputs: vec![],
        constant: false,
        state_mutability: Default::default(),
    }
}

pub fn burn_from_abi() -> Function {
    #[allow(deprecated)]
    Function {
        name: "burn_from".to_owned(),
        inputs: vec![
            Param {
                name: "account".to_owned(),
                kind: ParamType::Address,
                internal_type: None,
            },
            Param {
                name: "amount".to_owned(),
                kind: ParamType::Uint(256),
                internal_type: None,
            },
        ],
        outputs: vec![],
        constant: false,
        state_mutability: Default::default(),
    }
}

#[test]
fn test_to_ascii_hex() {
    let sub_account: AccountId32 = AccountId32::from_str(SUB_ACCOUNT).unwrap();
    let pubkey = String::from_utf8(to_ascii_hex(sub_account.as_ref())).unwrap();

    assert_eq!(&pubkey, PUBKEY);
}

#[test]
fn recover_eth_address() {
    new_test_ext().execute_with(|| {
        let s = EcdsaSignature::from_slice(&SIGNATURE);
        let p = PUBKEY.as_bytes();
        let address = crate::eth_recover(&s, p, &[][..]).unwrap();

        assert_eq!(address, H160::from_slice(&EVM_ADDR))
    })
}

#[test]
fn mint_into_abi_encode() {
    #[allow(deprecated)]
    let mint_into = mint_into_abi();

    let account = H160::from_slice(&EVM_ADDR);
    let amount = U256::from(100_000_000);
    let mut uint = [0u8; 32];
    amount.to_big_endian(&mut uint[..]);

    let encoded = mint_into
        .encode_input(&[Token::Address(account), Token::Uint(uint.into())])
        .unwrap();

    let expected = hex!("efe51695000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac0000000000000000000000000000000000000000000000000000000005f5e100").to_vec();
    assert_eq!(encoded, expected);

    let expected_sig = hex!("efe51695").to_vec();
    assert_eq!(mint_into.short_signature().to_vec(), expected_sig);

    let encoded2 = crate::mint_into_encode(account, 100_000_000u128);
    assert_eq!(encoded2, expected);
}

#[test]
fn burn_from_abi_encode() {
    #[allow(deprecated)]
    let burn_from = burn_from_abi();

    let account = H160::from_slice(&EVM_ADDR);
    let amount = U256::from(100_000_000);
    let mut uint = [0u8; 32];
    amount.to_big_endian(&mut uint[..]);

    let encoded = burn_from
        .encode_input(&[Token::Address(account), Token::Uint(uint.into())])
        .unwrap();

    let expected = hex!("0f536f84000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac0000000000000000000000000000000000000000000000000000000005f5e100").to_vec();
    assert_eq!(encoded, expected);

    let expected_sig = hex!("0f536f84").to_vec();
    assert_eq!(burn_from.short_signature().to_vec(), expected_sig);

    let encoded2 = crate::burn_from_encode(account, 100_000_000u128);
    assert_eq!(encoded2, expected);
}

#[test]
fn pause_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AssetsBridge::register(
            Origin::signed(ALICE.into()),
            1,
            H160::from_slice(&ERC20_1)
        ));
        expect_event(AssetsBridgeEvent::Register(1, H160::from_slice(&ERC20_1)));

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::EthAddressHasNotMapped
        );

        assert_ok!(AssetsBridge::pause(Origin::signed(ALICE.into()), Some(1)));
        expect_event(AssetsBridgeEvent::Paused(1));

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::InEmergency
        );
    })
}

#[test]
fn pause_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AssetsBridge::pause(Origin::signed(ALICE.into()), Some(1)),
            Error::<Test>::AssetIdHasNotMapped
        );

        assert_ok!(AssetsBridge::register(
            Origin::signed(ALICE.into()),
            1,
            H160::from_slice(&ERC20_1)
        ));
        expect_event(AssetsBridgeEvent::Register(1, H160::from_slice(&ERC20_1)));

        assert_noop!(
            AssetsBridge::pause(Origin::signed(BOB.into()), Some(1)),
            Error::<Test>::RequireAdmin
        );
    })
}

#[test]
fn pause_after_pause_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AssetsBridge::register(
            Origin::signed(ALICE.into()),
            1,
            H160::from_slice(&ERC20_1)
        ));
        expect_event(AssetsBridgeEvent::Register(1, H160::from_slice(&ERC20_1)));

        assert_ok!(AssetsBridge::register(
            Origin::signed(ALICE.into()),
            2,
            H160::from_slice(&ERC20_2)
        ));
        expect_event(AssetsBridgeEvent::Register(2, H160::from_slice(&ERC20_2)));

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::EthAddressHasNotMapped
        );

        // 1. pause(1)
        assert_ok!(AssetsBridge::pause(Origin::signed(ALICE.into()), Some(1)));
        expect_event(AssetsBridgeEvent::Paused(1));
        assert_eq!(AssetsBridge::emergencies(), vec![1]);

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::InEmergency
        );

        // 2. pause(1)
        assert_ok!(AssetsBridge::pause(Origin::signed(ALICE.into()), Some(1)));
        expect_event(AssetsBridgeEvent::Paused(1));
        assert_eq!(AssetsBridge::emergencies(), vec![1]);

        // 3. pause all
        assert_ok!(AssetsBridge::pause(Origin::signed(ALICE.into()), None));
        expect_event(AssetsBridgeEvent::PausedAll);
        assert_eq!(AssetsBridge::emergencies(), vec![1, 2]);

        // 4. pause(2)
        assert_ok!(AssetsBridge::pause(Origin::signed(ALICE.into()), Some(2)));

        // 5. pause(3)
        assert_noop!(
            AssetsBridge::pause(Origin::signed(ALICE.into()), Some(3)),
            Error::<Test>::AssetIdHasNotMapped
        );
    })
}

#[test]
fn unpause_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AssetsBridge::register(
            Origin::signed(ALICE.into()),
            1,
            H160::from_slice(&ERC20_1)
        ));
        expect_event(AssetsBridgeEvent::Register(1, H160::from_slice(&ERC20_1)));

        assert_ok!(AssetsBridge::pause(Origin::signed(ALICE.into()), None));
        expect_event(AssetsBridgeEvent::PausedAll);

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::InEmergency
        );

        assert_ok!(AssetsBridge::unpause(Origin::signed(ALICE.into()), Some(1)));
        expect_event(AssetsBridgeEvent::UnPaused(1));

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::EthAddressHasNotMapped
        );
    })
}

#[test]
fn unpause_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AssetsBridge::unpause(Origin::signed(ALICE.into()), Some(1)),
            Error::<Test>::AssetIdHasNotMapped
        );

        assert_ok!(AssetsBridge::register(
            Origin::signed(ALICE.into()),
            1,
            H160::from_slice(&ERC20_1)
        ));
        expect_event(AssetsBridgeEvent::Register(1, H160::from_slice(&ERC20_1)));

        assert_noop!(
            AssetsBridge::unpause(Origin::signed(BOB.into()), Some(1)),
            Error::<Test>::RequireAdmin
        );
    })
}

#[test]
fn unpause_after_unpause_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AssetsBridge::register(
            Origin::signed(ALICE.into()),
            1,
            H160::from_slice(&ERC20_1)
        ));
        expect_event(AssetsBridgeEvent::Register(1, H160::from_slice(&ERC20_1)));

        assert_ok!(AssetsBridge::register(
            Origin::signed(ALICE.into()),
            2,
            H160::from_slice(&ERC20_2)
        ));
        expect_event(AssetsBridgeEvent::Register(2, H160::from_slice(&ERC20_2)));

        assert_ok!(AssetsBridge::unpause(Origin::signed(ALICE.into()), Some(1)));
        assert!(AssetsBridge::emergencies().is_empty());

        assert_ok!(AssetsBridge::unpause(Origin::signed(ALICE.into()), Some(2)));
        assert!(AssetsBridge::emergencies().is_empty());

        assert_noop!(
            AssetsBridge::pause(Origin::signed(ALICE.into()), Some(3)),
            Error::<Test>::AssetIdHasNotMapped
        );
        assert!(AssetsBridge::emergencies().is_empty());

        assert_ok!(AssetsBridge::pause(Origin::signed(ALICE.into()), Some(1)));

        assert_eq!(AssetsBridge::emergencies(), vec![1]);

        assert_ok!(AssetsBridge::pause(Origin::signed(ALICE.into()), Some(2)));

        assert_eq!(AssetsBridge::emergencies(), vec![1, 2]);

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::InEmergency
        );

        assert_noop!(
            AssetsBridge::withdraw(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::InEmergency
        );

        assert_ok!(AssetsBridge::unpause(Origin::signed(ALICE.into()), Some(2)));
        expect_event(AssetsBridgeEvent::UnPaused(2));

        assert_noop!(
            AssetsBridge::withdraw(Origin::signed(BOB.into()), 2, 1),
            Error::<Test>::EthAddressHasNotMapped
        );
        assert_eq!(AssetsBridge::emergencies(), vec![1]);

        assert_ok!(AssetsBridge::unpause(Origin::signed(ALICE.into()), None));
        expect_event(AssetsBridgeEvent::UnPausedAll);

        assert_noop!(
            AssetsBridge::withdraw(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::EthAddressHasNotMapped
        );
        assert!(AssetsBridge::emergencies().is_empty());
    })
}

#[test]
fn more_pause_and_unpause_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AssetsBridge::register(
            Origin::signed(ALICE.into()),
            1,
            H160::from_slice(&ERC20_1)
        ));
        expect_event(AssetsBridgeEvent::Register(1, H160::from_slice(&ERC20_1)));

        assert_ok!(AssetsBridge::register(
            Origin::signed(ALICE.into()),
            2,
            H160::from_slice(&ERC20_2)
        ));
        expect_event(AssetsBridgeEvent::Register(2, H160::from_slice(&ERC20_2)));

        assert!(AssetsBridge::emergencies().is_empty());

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::EthAddressHasNotMapped
        );

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 2, 1),
            Error::<Test>::EthAddressHasNotMapped
        );

        assert_ok!(AssetsBridge::pause(Origin::signed(ALICE.into()), None));
        expect_event(AssetsBridgeEvent::PausedAll);

        assert_eq!(AssetsBridge::emergencies(), vec![1, 2]);

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::InEmergency
        );

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 2, 1),
            Error::<Test>::InEmergency
        );

        assert_ok!(AssetsBridge::unpause(Origin::signed(ALICE.into()), Some(2)));
        expect_event(AssetsBridgeEvent::UnPaused(2));

        assert_eq!(AssetsBridge::emergencies(), vec![1]);

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::InEmergency
        );

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 2, 1),
            Error::<Test>::EthAddressHasNotMapped
        );

        assert_ok!(AssetsBridge::unpause(Origin::signed(ALICE.into()), None));
        expect_event(AssetsBridgeEvent::UnPausedAll);

        assert!(AssetsBridge::emergencies().is_empty());

        assert_noop!(
            AssetsBridge::deposit(Origin::signed(BOB.into()), 1, 1),
            Error::<Test>::EthAddressHasNotMapped
        );
    })
}
