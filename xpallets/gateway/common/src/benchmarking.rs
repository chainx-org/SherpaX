// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode};
use frame_benchmarking::benchmarks;
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_core::crypto::AccountId32;
#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::traits::CheckedDiv;
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

use xp_assets_registrar::Chain;
use xpallet_gateway_records::{Pallet as XGatewayRecords, WithdrawalRecordId, WithdrawalState};

use crate::{
    traits::TrusteeSession, types::*, Balanceof, Call, Config, LittleBlackHouse, Pallet,
    TrusteeIntentionPropertiesOf, TrusteeMultiSigAddr, TrusteeSessionInfoLen, TrusteeSessionInfoOf,
};

fn create_default_asset<T: Config>(who: T::AccountId) {
    let miner = T::Lookup::unlookup(who);
    let _ = pallet_assets::Pallet::<T>::force_create(
        RawOrigin::Root.into(),
        T::BtcAssetId::get(),
        miner,
        true,
        1u32.into(),
    );
}
#[cfg(feature = "runtime-benchmarks")]
fn update_trustee_info<T: Config>(session_num: u32) {
    TrusteeSessionInfoOf::<T>::mutate(Chain::Bitcoin, session_num, |info| match info {
        None => (),
        Some(trustee) => {
            for i in 0..trustee.0.trustee_list.len() {
                trustee.0.trustee_list[i].1 = i as u64 + 1;
            }
            let end_height = 10u32.into();
            trustee.0.end_height = Some(end_height);
        }
    });
}

fn account<T: Config>(pubkey: &str) -> T::AccountId {
    let pubkey = hex::decode(pubkey).unwrap();
    let mut public = [0u8; 32];
    public.copy_from_slice(pubkey.as_slice());
    let account = AccountId32::from(public).encode();
    Decode::decode(&mut account.as_slice()).unwrap()
}
fn alice<T: Config>() -> T::AccountId {
    // sr25519 Alice
    account::<T>("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
}
fn bob<T: Config>() -> T::AccountId {
    // sr25519 Bob
    account::<T>("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48")
}
fn charlie<T: Config>() -> T::AccountId {
    // sr25519 Charlie
    account::<T>("90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22")
}
fn dave<T: Config>() -> T::AccountId {
    // sr25519 Dave
    account::<T>("306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20")
}
fn new_trustees<T: Config>() -> Vec<(T::AccountId, Vec<u8>, Vec<u8>, Vec<u8>)> {
    vec![
        (
            alice::<T>(),
            b"".to_vec(),
            hex::decode("0483f579dd2380bd31355d066086e1b4d46b518987c1f8a64d4c0101560280eae2b16f3068e94333e11ee63770936eca9692a25f76012511d38ac30ece20f07dca")
                .expect("hex decode failed"),
            hex::decode("0400849497d4f88ebc3e1bc2583677c5abdbd3b63640b3c5c50cd4628a33a2a2cab6b69094b5a213da80f9ef730fab39de770ca124f2d9a9cb161856be54b9adc5")
                .expect("hex decode failed"),
        ),
        (
            bob::<T>(),
            b"".to_vec(),
            hex::decode("047a0868a14bd18e2e45ff3ad960f892df8d0edd1a5685f0a1dc63c7986d4ad55d47c09531e4f2ca2ae7f9ed80c1f9df2edd8afa19188692724d2bc18c18d98c10")
                .expect("hex decode failed"),
            hex::decode("042122032ae9656f9a133405ffe02101469a8d62002270a33ceccf0e40dda54d08c989b55f1b6b46a8dee284cf6737de0a377e410bcfd361a015528ae80a349529")
                .expect("hex decode failed"),
        ),
        (
            charlie::<T>(),
            b"".to_vec(),
            hex::decode("04c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565faced14acb5172ee19aee5417488fecdda33f4cfea9ff04f250e763e6f7458d5e")
                .expect("hex decode failed"),
            hex::decode("04b3cc747f572d33f12870fa6866aebbfd2b992ba606b8dc89b676b3697590ad63d5ca398bdb6f8ee619f2e16997f21e5e8f0e0b00e2f275c7cb1253f381058d56")
                .expect("hex decode failed"),
        ),
        (
            dave::<T>(),
            b"".to_vec(),
            hex::decode("042122032ae9656f9a133405ffe02101469a8d62002270a33ceccf0e40dda54d08c989b55f1b6b46a8dee284cf6737de0a377e410bcfd361a015528ae80a349529")
                .expect("hex decode failed"),
            hex::decode("047a0868a14bd18e2e45ff3ad960f892df8d0edd1a5685f0a1dc63c7986d4ad55d47c09531e4f2ca2ae7f9ed80c1f9df2edd8afa19188692724d2bc18c18d98c10")
                .expect("hex decode failed"),
        ),
    ]
}

/// removes all the storage items to reverse any genesis state.
fn clean<T: Config>() {
    <LittleBlackHouse<T>>::remove_all(None);
    <TrusteeSessionInfoLen<T>>::remove_all(None);
    <TrusteeSessionInfoOf<T>>::remove_all(None);
}

benchmarks! {
    withdraw {
        let caller: T::AccountId = alice::<T>();
        create_default_asset::<T>(caller.clone());
        let amount: T::Balance = 1_000_000_000u32.into();
        XGatewayRecords::<T>::deposit(&caller, T::BtcAssetId::get(), amount).unwrap();
        let withdrawal = 100_000_000u32.into();
        let addr = b"3PgYgJA6h5xPEc3HbnZrUZWkpRxuCZVyEP".to_vec();
        let memo = b"".to_vec().into();
    }: _(RawOrigin::Signed(caller.clone()), T::BtcAssetId::get(), withdrawal, addr, memo)
    verify {
        assert!(XGatewayRecords::<T>::pending_withdrawals(0).is_some());
        assert_eq!(
            XGatewayRecords::<T>::state_of(0),
            Some(WithdrawalState::Applying)
        );
    }

    cancel_withdrawal {
        let caller: T::AccountId = alice::<T>();
        create_default_asset::<T>(caller.clone());
        let amount: T::Balance = 1_000_000_000_u32.into();
        XGatewayRecords::<T>::deposit(&caller, T::BtcAssetId::get(), amount).unwrap();

        let withdrawal = 100_000_000u32.into();
        let addr = b"3PgYgJA6h5xPEc3HbnZrUZWkpRxuCZVyEP".to_vec();
        let memo = b"".to_vec().into();
        Pallet::<T>::withdraw(
            RawOrigin::Signed(caller.clone()).into(),
            T::BtcAssetId::get(), withdrawal, addr, memo,
        )
        .unwrap();

        let withdrawal_id: WithdrawalRecordId = 0;
        assert!(XGatewayRecords::<T>::pending_withdrawals(withdrawal_id).is_some());
        assert_eq!(
            XGatewayRecords::<T>::state_of(withdrawal_id),
            Some(WithdrawalState::Applying)
        );

    }: _(RawOrigin::Signed(caller.clone()), withdrawal_id)
    verify {
        assert!(XGatewayRecords::<T>::pending_withdrawals(withdrawal_id).is_none());
        assert!(XGatewayRecords::<T>::state_of(withdrawal_id).is_none());
    }

    setup_trustee {
        let caller: T::AccountId = alice::<T>();
        clean::<T>();
        <TrusteeIntentionPropertiesOf<T>>::remove(caller.clone(), Chain::Bitcoin);
        LittleBlackHouse::<T>::append(Chain::Bitcoin, caller.clone());
        let hot = hex::decode("0483f579dd2380bd31355d066086e1b4d46b518987c1f8a64d4c0101560280eae2b16f3068e94333e11ee63770936eca9692a25f76012511d38ac30ece20f07dca")
                .unwrap();
        let cold = hex::decode("0400849497d4f88ebc3e1bc2583677c5abdbd3b63640b3c5c50cd4628a33a2a2cab6b69094b5a213da80f9ef730fab39de770ca124f2d9a9cb161856be54b9adc5")
                .unwrap();

        assert!(Pallet::<T>::trustee_intention_props_of(caller.clone(), Chain::Bitcoin).is_none());
    }: _(RawOrigin::Signed(caller.clone()), None, Chain::Bitcoin, b"about".to_vec(), hot, cold)
    verify {
        assert!(Pallet::<T>::trustee_intention_props_of(caller, Chain::Bitcoin).is_some());
    }

    set_trustee_proxy {
        let caller: T::AccountId = alice::<T>();
        assert!(Pallet::<T>::trustee_intention_props_of(caller.clone(), Chain::Bitcoin).is_some());
    }: _(RawOrigin::Signed(caller.clone()), bob::<T>(), Chain::Bitcoin)
    verify {
        assert_eq!(
            Pallet::<T>::trustee_intention_props_of(caller, Chain::Bitcoin).unwrap().0.proxy_account,
            Some(bob::<T>())
        );
    }

    set_trustee_info_config {
        let config = TrusteeInfoConfig {
            min_trustee_count: 5,
            max_trustee_count: 15,
        };
    }: _(RawOrigin::Root, Chain::Bitcoin, config.clone())
    verify {
        assert_eq!(Pallet::<T>::trustee_info_config_of(Chain::Bitcoin), config);
    }

    set_trustee_admin {
        let who: T::AccountId = alice::<T>();
        for (account, about, hot, cold) in new_trustees::<T>() {
            Pallet::<T>::setup_trustee_impl(account.clone(), None, Chain::Bitcoin, about, hot, cold).unwrap();
        }
    }: _(RawOrigin::Root, who.clone())
    verify {
        assert_eq!(Pallet::<T>::trustee_admin().unwrap(), who);
    }

    set_trustee_admin_multiply {
        let multiply = 12;
    }: _(RawOrigin::Root, multiply)
    verify{
        assert_eq!(Pallet::<T>::trustee_admin_multiply(), multiply);
    }

    claim_trustee_reward {
        let caller: T::AccountId = alice::<T>();
        clean::<T>();
        TrusteeMultiSigAddr::<T>::insert(Chain::Bitcoin, caller.clone());
        assert_eq!(Pallet::<T>::trustee_session_info_len(Chain::Bitcoin), 0);
        assert!(Pallet::<T>::trustee_session_info_of(Chain::Bitcoin, 0).is_none());
        let mut candidators = vec![];
        let trustee_info = new_trustees::<T>();
        let trustee_len = trustee_info.len();
        for (account, about, hot, cold) in (&trustee_info[0..trustee_len-1]).to_vec() {
            Pallet::<T>::setup_trustee_impl(account.clone(), None, Chain::Bitcoin, about, hot, cold).unwrap();
            candidators.push(account);
        }
        assert_eq!(Pallet::<T>::transition_trustee_session_impl(Chain::Bitcoin, candidators), Ok(()));

        let mut candidators = vec![];
        let trustee_info = new_trustees::<T>();
        let trustee_len = trustee_info.len();
        for (account, about, hot, cold) in (&trustee_info[1..trustee_len]).to_vec() {
            Pallet::<T>::setup_trustee_impl(account.clone(), None, Chain::Bitcoin, about, hot, cold).unwrap();
            candidators.push(account);
        }
        assert_eq!(Pallet::<T>::transition_trustee_session_impl(Chain::Bitcoin, candidators), Ok(()));
        assert_eq!(Pallet::<T>::trustee_session_info_len(Chain::Bitcoin), 2);
        assert!(Pallet::<T>::trustee_session_info_of(Chain::Bitcoin, 2).is_some());
        let reward: Balanceof<T> = 100_000_000u32.into();
        let session_num = 1;
        #[cfg(feature = "runtime-benchmarks")]
        update_trustee_info::<T>(session_num);
        #[cfg(feature = "runtime-benchmarks")]
        let reward: Balanceof<T> = <T as xpallet_gateway_records::Config>::Currency::free_balance(&caller).checked_div(&2u32.into()).unwrap();
        let multi_account = <T as crate::Config>::BitcoinTrusteeSessionProvider::trustee_session(session_num).unwrap().multi_account.unwrap();
        <T as xpallet_gateway_records::Config>::Currency::deposit_creating(&multi_account, reward);
    }: _(RawOrigin::Signed(caller.clone()), Chain::Bitcoin, session_num as i32)
    verify {
        #[cfg(not(feature = "runtime-benchmarks"))]
        assert_eq!(<T as xpallet_gateway_records::Config>::Currency::free_balance(&trustee_info[0].0), 33333333u32.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        ExtBuilder::default().build().execute_with(|| {
            assert_ok!(Pallet::<Test>::test_benchmark_withdraw());
            assert_ok!(Pallet::<Test>::test_benchmark_cancel_withdrawal());
            assert_ok!(Pallet::<Test>::test_benchmark_setup_trustee());
            assert_ok!(Pallet::<Test>::test_benchmark_set_trustee_proxy());
            assert_ok!(Pallet::<Test>::test_benchmark_set_trustee_info_config());
            assert_ok!(Pallet::<Test>::test_benchmark_set_trustee_admin());
            assert_ok!(Pallet::<Test>::test_benchmark_set_trustee_admin_multiply());
            assert_ok!(Pallet::<Test>::test_benchmark_claim_trustee_reward());
        });
    }
}
