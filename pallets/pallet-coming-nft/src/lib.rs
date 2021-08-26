#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused)]
#![allow(clippy::unnecessary_cast)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

use frame_support::inherent::Vec;
use frame_support::pallet_prelude::*;
use pallet_coming_id::{Cid, ComingNFT};
use sp_core::Bytes;
use sp_runtime::traits::StaticLookup;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::dispatch::DispatchResult;
    use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_coming_id::Config {
        /// The implement of ComingNFT triat, eg. pallet-coming-id
        type ComingNFT: ComingNFT<Self::AccountId>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(<T as pallet::Config>::WeightInfo::mint(card.len() as u32))]
        pub fn mint(origin: OriginFor<T>, cid: Cid, card: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            T::ComingNFT::mint(&who, cid, card)
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::burn())]
        pub fn burn(origin: OriginFor<T>, cid: Cid) -> DispatchResult {
            let who = ensure_signed(origin)?;

            T::ComingNFT::burn(&who, cid)
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer())]
        pub fn transfer(
            origin: OriginFor<T>,
            cid: Cid,
            recipient: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let recipient = T::Lookup::lookup(recipient)?;

            T::ComingNFT::transfer(&who, cid, &recipient)
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer_from())]
        pub fn transfer_from(
            origin: OriginFor<T>,
            from: <T::Lookup as StaticLookup>::Source,
            to: <T::Lookup as StaticLookup>::Source,
            cid: Cid,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let from = T::Lookup::lookup(from)?;
            let to = T::Lookup::lookup(to)?;

            T::ComingNFT::transfer_from(&who, &from, &to, cid)
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::approve())]
        pub fn approve(
            origin: OriginFor<T>,
            approved: <T::Lookup as StaticLookup>::Source,
            cid: Cid,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let approved = T::Lookup::lookup(approved)?;

            T::ComingNFT::approve(&who, &approved, cid)
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_approval_for_all())]
        pub fn set_approval_for_all(
            origin: OriginFor<T>,
            operator: <T::Lookup as StaticLookup>::Source,
            approved: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let operator = T::Lookup::lookup(operator)?;

            T::ComingNFT::set_approval_for_all(&who, &operator, approved)
        }
    }
}

impl<T: Config> Pallet<T> {
    fn cids_of_owner(who: T::AccountId) -> Vec<Cid> {
        T::ComingNFT::cids_of_owner(&who)
    }

    fn owner_of_cid(cid: Cid) -> Option<T::AccountId> {
        T::ComingNFT::owner_of_cid(cid)
    }

    fn card_of_cid(cid: Cid) -> Option<Bytes> {
        T::ComingNFT::card_of_cid(cid)
    }

    fn get_approved(cid: Cid) -> Option<T::AccountId> {
        T::ComingNFT::get_approved(cid)
    }

    fn is_approved_for_all(owner: &T::AccountId, operator: &T::AccountId) -> bool {
        T::ComingNFT::is_approved_for_all(owner, operator)
    }
}
