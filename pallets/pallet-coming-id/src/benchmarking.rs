//! Benchmarking setup for pallet-coming-id

use super::*;

#[allow(unused)]
use crate::Pallet as ComingId;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_std::vec;

const SEED: u32 = 0;

// Alice
fn admin_account<AccountId: Decode + Default>() -> AccountId {
    let alice =
        hex_literal::hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"];
    AccountId::decode(&mut &alice[..]).unwrap_or_default()
}

benchmarks! {
    register {
        let admin: T::AccountId = admin_account();
        let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(admin.clone());
        let claim_cid: Cid = 1000000;
    }: register(RawOrigin::Signed(admin), claim_cid, recipient_lookup)
    verify {
        assert!(Distributed::<T>::get(claim_cid).is_some());
    }

    bond {
        let common_user: T::AccountId = account("common_user", 0, SEED);
        let claim_cid: Cid = 1000000;

        let _ = Distributed::<T>::try_mutate_exists::<_,_,Error<T>,_>(claim_cid, |details|{
            *details = Some(CidDetails{
                owner: common_user.clone(),
                bonds: Vec::new(),
                card: Bytes::from(Vec::new())
            });

            Ok(())
        })?;

        let b in 0 .. *T::BlockLength::get().max.get(DispatchClass::Normal) as u32;
        let bond_data = BondData{
            bond_type: 1u16,
            data: vec![1; b as usize].into(),
        };

    }: bond(RawOrigin::Signed(common_user.clone()), claim_cid, bond_data.clone())
    verify {
        let option = Distributed::<T>::get(claim_cid);
        assert!(option.is_some());

        let cid_details = option.unwrap();
        assert_eq!(cid_details.owner, common_user);
    }

    unbond {
        let common_user: T::AccountId = account("common_user", 0, SEED);
        let claim_cid: Cid = 1000000;
        let bond_data = BondData{
            bond_type: 1u16,
            data: Bytes::from(b"benchmark".to_vec()),
        };

        let mut bonds: Vec<BondData> = Vec::new();
        bonds.push(bond_data);

        let _ = Distributed::<T>::try_mutate_exists::<_,_,Error<T>,_>(claim_cid, |details|{
            *details = Some(CidDetails{
                owner: common_user.clone(),
                bonds: bonds,
                card: Bytes::from(Vec::new())
            });

            Ok(())
        })?;

    }: unbond(RawOrigin::Signed(common_user.clone()), claim_cid, 1u16)
    verify {
        let option = Distributed::<T>::get(claim_cid);
        assert!(option.is_some());

        let cid_details = option.unwrap();
        assert_eq!(cid_details.owner, common_user);
    }
}

impl_benchmark_test_suite!(
    ComingId,
    crate::mock::new_test_ext(super::admin_account()),
    crate::mock::Test,
);
