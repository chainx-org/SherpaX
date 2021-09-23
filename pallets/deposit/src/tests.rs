use crate::mock::*;

use frame_support::assert_ok;
use sp_runtime::traits::BlakeTwo256;
use sp_core::{H160, H256};
use std::str::FromStr;
use pallet_coming_id::ComingNFT;

const COMMON_CID: u64 = 1_000_000;
const ETH_ADDRESS: &str = "f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
const SUB_ACCOUNT: &str = "5Fghzk1AJt88PeFEzuRfXzbPchiBbsVGTTXcdx599VdZzkTA";

#[test]
fn address_mapping_is_ok() {
    use pallet_evm::AddressMapping;

    /// Hashed address mapping.
    pub struct HashedAddressMapping<H>(sp_std::marker::PhantomData<H>);

    impl<H: sp_core::Hasher<Out = H256>> pallet_evm::AddressMapping<AccountId32> for HashedAddressMapping<H> {
        fn into_account_id(address: H160) -> AccountId32 {
            let mut data = [0u8; 24];
            data[0..4].copy_from_slice(b"evm:");
            data[4..24].copy_from_slice(&address[..]);
            let hash = H::hash(&data);

            AccountId32::from(Into::<[u8; 32]>::into(hash))
        }
    }

    let eth_address: H160 = H160::from_slice(&hex::decode(ETH_ADDRESS).unwrap());
    let account_id: AccountId32 = HashedAddressMapping::<BlakeTwo256>::into_account_id(eth_address);

    // use codec::Encode;
    // use sp_core::hexdisplay::HexDisplay;
    // println!("{}", HexDisplay::from(&account_id.encode()));
    // a02a00e549cb104f710d3fe6f2f83e91524d2a40c4ed831658a120883077f9a9
    // println!("{}", account_id);
    // 5Fghzk1AJt88PeFEzuRfXzbPchiBbsVGTTXcdx599VdZzkTA

    let sub_account: AccountId32 = AccountId32::from_str(SUB_ACCOUNT).unwrap();

    assert_eq!(account_id, sub_account);
}

#[test]
fn deposit_balance_should_work() {
    new_test_ext(ALICE).execute_with(|| {
        let eth_address: H160 = H160::from_slice(&hex::decode(ETH_ADDRESS).unwrap());
        let value: u128 = 1_000_000;
        let sub_account: AccountId32 = AccountId32::from_str(SUB_ACCOUNT).unwrap();
        let balance_alice = <Test as Config>::Currency::free_balance(&ALICE);

        assert_eq!(<Test as Config>::Currency::free_balance(&sub_account), 0);
        assert_ok!(Deposit::deposit_balance(Origin::signed(ALICE), eth_address, value));
        assert_eq!(<Test as Config>::Currency::free_balance(&sub_account), value);
        assert_eq!(<Test as Config>::Currency::free_balance(&ALICE), balance_alice - value);
    });
}

#[test]
fn deposit_cid_should_work() {
    new_test_ext(ALICE).execute_with(|| {
        let eth_address: H160 = H160::from_slice(&hex::decode(ETH_ADDRESS).unwrap());
        let sub_account: AccountId32 = AccountId32::from_str(SUB_ACCOUNT).unwrap();

        assert_ok!(ComingId::register(Origin::signed(ALICE), COMMON_CID, ALICE));
        assert_eq!(<Test as Config>::ComingNFT::owner_of_cid(COMMON_CID), Some(ALICE));

        assert_ok!(Deposit::deposit_cid(Origin::signed(ALICE), eth_address, COMMON_CID));
        assert_eq!(<Test as Config>::ComingNFT::owner_of_cid(COMMON_CID), Some(sub_account));
    });
}
