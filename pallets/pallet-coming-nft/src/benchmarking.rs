//! Benchmarking setup for pallet-coming-nft

use super::*;

#[allow(unused)]
use crate::Pallet as ComingNFT;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use pallet_coming_id as ComingId;
use sp_std::vec;

// Alice
fn admin_account<AccountId: Decode + Default>() -> AccountId {
    let alice =
        hex_literal::hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"];
    AccountId::decode(&mut &alice[..]).unwrap_or_default()
}

benchmarks! {
    mint {
        let admin: T::AccountId = admin_account();
        let claim_cid: Cid = 1000000;
        let recipient: T::AccountId = account("recipient", 0, 0);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
        let b in 1 .. T::MaxCardSize::get();
        let card = vec![1; b as usize];

        assert!(
            ComingId::Pallet::<T>::register(
                RawOrigin::Signed(admin.clone()).into(),
                claim_cid,
                recipient_lookup,
            )
            .is_ok()
        );

    }: mint(RawOrigin::Signed(admin), claim_cid, card.clone())
    verify {
        assert_eq!(ComingNFT::<T>::card_of_cid(claim_cid), Some(Bytes::from(card)));
        assert_eq!(ComingNFT::<T>::owner_of_cid(claim_cid), Some(recipient.clone()));
        assert_eq!(ComingNFT::<T>::cids_of_owner(recipient), vec![claim_cid]);
    }

    burn {
        let admin: T::AccountId = admin_account();
        let claim_cid: Cid = 99999;
        let recipient: T::AccountId = account("recipient", 0, 0);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());

        assert!(
            ComingId::Pallet::<T>::register(
                RawOrigin::Signed(admin.clone()).into(),
                claim_cid,
                recipient_lookup,
            )
            .is_ok()
        );

        assert_eq!(ComingNFT::<T>::card_of_cid(claim_cid), None);
        assert_eq!(ComingNFT::<T>::owner_of_cid(claim_cid), Some(recipient.clone()));
        assert_eq!(ComingNFT::<T>::cids_of_owner(recipient.clone()), vec![claim_cid]);
    }: burn(RawOrigin::Signed(admin), claim_cid)
    verify {
        assert_eq!(ComingNFT::<T>::card_of_cid(claim_cid), None);
        assert_eq!(ComingNFT::<T>::owner_of_cid(claim_cid), None);
        assert!(ComingNFT::<T>::cids_of_owner(recipient).is_empty());
    }

    transfer {
        let admin: T::AccountId = admin_account();
        let claim_cid: Cid = 1000000;
        let owner: T::AccountId = account("recipient", 0, 0);
        let owner_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(owner.clone());
        let recipient: T::AccountId = account("recipient", 0, 1);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
        let card = br#"{"name":"testcard"}"#.to_vec();

        assert!(
            ComingId::Pallet::<T>::register(
                RawOrigin::Signed(admin.clone()).into(),
                claim_cid,
                owner_lookup,
            )
            .is_ok()
        );

        assert!(
            ComingNFT::<T>::mint(
                RawOrigin::Signed(admin).into(),
                claim_cid,
                card.clone()
            )
            .is_ok()
        );

        assert_eq!(ComingNFT::<T>::cids_of_owner(owner.clone()), vec![claim_cid]);

    }: transfer(RawOrigin::Signed(owner.clone()), claim_cid, recipient_lookup)
    verify {
        assert_eq!(ComingNFT::<T>::card_of_cid(claim_cid), Some(Bytes::from(card)));
        assert_eq!(ComingNFT::<T>::owner_of_cid(claim_cid), Some(recipient.clone()));
        assert_eq!(ComingNFT::<T>::cids_of_owner(recipient), vec![claim_cid]);

        assert!(ComingNFT::<T>::cids_of_owner(owner).is_empty());
    }

    transfer_from {
        let admin: T::AccountId = admin_account();
        let claim_cid: Cid = 1000000;
        let owner: T::AccountId = account("recipient", 0, 0);
        let owner_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(owner.clone());
        let recipient: T::AccountId = account("recipient", 0, 1);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
        let operator: T::AccountId = account("recipient", 0, 2);
        let operator_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(operator.clone());
        let card = br#"{"name":"testcard"}"#.to_vec();

        assert!(
            ComingId::Pallet::<T>::register(
                RawOrigin::Signed(admin.clone()).into(),
                claim_cid,
                owner_lookup.clone(),
            )
            .is_ok()
        );

        assert!(
            ComingNFT::<T>::mint(
                RawOrigin::Signed(admin).into(),
                claim_cid,
                card.clone()
            )
            .is_ok()
        );

        assert!(
            ComingNFT::<T>::approve(
                RawOrigin::Signed(owner.clone()).into(),
                operator_lookup,
                claim_cid,
            )
            .is_ok()
        );

        assert_eq!(ComingNFT::<T>::cids_of_owner(owner.clone()), vec![claim_cid]);
        assert_eq!(ComingNFT::<T>::get_approved(claim_cid), Some(operator.clone()));
    }: transfer_from(RawOrigin::Signed(operator.clone()), owner_lookup, recipient_lookup, claim_cid)
    verify {
        assert_eq!(ComingNFT::<T>::card_of_cid(claim_cid), Some(Bytes::from(card)));
        assert_eq!(ComingNFT::<T>::owner_of_cid(claim_cid), Some(recipient.clone()));
        assert_eq!(ComingNFT::<T>::cids_of_owner(recipient), vec![claim_cid]);

        assert!(ComingNFT::<T>::cids_of_owner(owner).is_empty());
        assert_eq!(ComingNFT::<T>::get_approved(claim_cid), None);
    }

    approve {
        let admin: T::AccountId = admin_account();
        let claim_cid: Cid = 1000000;
        let owner: T::AccountId = account("recipient", 0, 0);
        let owner_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(owner.clone());
        let operator: T::AccountId = account("recipient", 0, 1);
        let operator_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(operator.clone());

        assert!(
            ComingId::Pallet::<T>::register(
                RawOrigin::Signed(admin.clone()).into(),
                claim_cid,
                owner_lookup.clone(),
            )
            .is_ok()
        );

        assert_eq!(ComingNFT::<T>::cids_of_owner(owner.clone()), vec![claim_cid]);
    }: approve(RawOrigin::Signed(owner.clone()), operator_lookup, claim_cid)
    verify {
        assert_eq!(ComingNFT::<T>::owner_of_cid(claim_cid), Some(owner));
        assert_eq!(ComingNFT::<T>::get_approved(claim_cid), Some(operator));
    }

    set_approval_for_all {
        let owner: T::AccountId = account("recipient", 0, 0);
        let owner_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(owner.clone());
        let operator: T::AccountId = account("recipient", 0, 1);
        let operator_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(operator.clone());


        assert!(!ComingNFT::<T>::is_approved_for_all(&owner, &operator));
    }: set_approval_for_all(RawOrigin::Signed(owner.clone()), operator_lookup, true)
    verify {
        assert!(ComingNFT::<T>::is_approved_for_all(&owner, &operator));
    }
}

impl_benchmark_test_suite!(
    ComingNFT,
    crate::mock::new_test_ext(super::admin_account()),
    crate::mock::Test,
);
