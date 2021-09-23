#![cfg_attr(not(feature = "std"), no_std)]

type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

use frame_support::traits::{Currency, ExistenceRequirement};
use pallet_coming_id::{ComingNFT, Cid};
use pallet_evm::AddressMapping;
use sp_core::H160;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::dispatch::DispatchResult;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_coming_id::Config {
		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;
		/// The implement of ComingNFT triat, eg. pallet-coming-id
		type ComingNFT: ComingNFT<Self::AccountId>;
		/// Mapping from address to account id.
		type AddressMapping: AddressMapping<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn deposit_balance(
			origin: OriginFor<T>,
			address: H160,
			value: BalanceOf<T>,
		) -> DispatchResult {
			// 1. ethereum address -> substrate account_id
			let who = ensure_signed(origin)?;
			let address_account_id = T::AddressMapping::into_account_id(address);

			// 2. transfer balance
			T::Currency::transfer(
				&who,
				&address_account_id,
				value,
				ExistenceRequirement::AllowDeath,
			)
		}

		#[pallet::weight(0)]
		pub fn deposit_cid(
			origin: OriginFor<T>,
			address: H160,
			cid: Cid,
		) -> DispatchResult {
			// 1. ethereum address -> substrate account_id
			let who = ensure_signed(origin)?;
			let address_account_id = T::AddressMapping::into_account_id(address);

			// 2. transfer cid
			T::ComingNFT::transfer(&who, cid, &address_account_id)
		}
	}
}
