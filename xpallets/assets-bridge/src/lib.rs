//! # Assets Bridge
//!
//! ## Overview
//!
//! Bridge between pallet-assets and Erc20 tokens

#![cfg_attr(not(feature = "std"), no_std)]

pub mod abi;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub use abi::*;
pub mod recover;
pub use recover::*;

use codec::Encode;
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, IsType},
    transactional,
};
use sp_core::{ecdsa, H160, U256};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_runtime::traits::{StaticLookup, UniqueSaturatedInto};
use sp_std::vec::Vec;

use pallet_evm::{AddressMapping, ExitReason, Runner};

pub type EcdsaSignature = ecdsa::Signature;
pub type AddressMappingOf<T> = <T as pallet_evm::Config>::AddressMapping;
pub type BalanceOf<T> = <<T as pallet_evm::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::fungibles::Mutate;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_assets::Config + pallet_evm::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The assets-bridge's inner evm caller.
        #[pallet::constant]
        type EvmCaller: Get<H160>;
    }

    /// The Substrate Account for Evm Addresses
    ///
    /// SubAccounts: map H160 => Option<AccountId>
    #[pallet::storage]
    #[pallet::getter(fn sub_accounts)]
    pub type SubAccounts<T: Config> = StorageMap<_, Twox64Concat, H160, T::AccountId, OptionQuery>;

    /// The Evm Addresses for Substrate Accounts
    ///
    /// EvmAccounts: map AccountId => Option<H160>
    #[pallet::storage]
    #[pallet::getter(fn evm_accounts)]
    pub type EvmAccounts<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, H160, OptionQuery>;

    /// The Erc20 Contract Addresses for Asset Ids
    ///
    /// Erc20s: map AssetId => Option<H160>
    #[pallet::storage]
    #[pallet::getter(fn erc20s)]
    pub type Erc20s<T: Config> = StorageMap<_, Twox64Concat, T::AssetId, H160, OptionQuery>;

    /// The Asset Ids for Erc20 Contract Addresses
    ///
    /// AssetIds: map H160 => Option<AssetId>
    #[pallet::storage]
    #[pallet::getter(fn asset_ids)]
    pub type AssetIds<T: Config> = StorageMap<_, Twox64Concat, H160, T::AssetId, OptionQuery>;

    /// The pallet admin key.
    #[pallet::storage]
    #[pallet::getter(fn admin_key)]
    pub(super) type Admin<T: Config> = StorageValue<_, T::AccountId>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The `AccountId` of the admin key.
        pub admin_key: Option<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                admin_key: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            if let Some(key) = &self.admin_key {
                <Admin<T>>::put(key.clone());
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// (account_id, evm_address)
        ClaimAccount(T::AccountId, H160),
        /// (asset_id, account_id, evm_address, amount, erc20_contract)
        DepositExecuted(T::AssetId, T::AccountId, H160, T::Balance, H160),
        /// (asset_id, account_id, evm_address, amount, erc20_contract)
        WithdrawExecuted(T::AssetId, T::AccountId, H160, T::Balance, H160),
        /// (account_id, evm_address, sub_account, amount, into_evm)
        Teleport(T::AccountId, H160, T::AccountId, BalanceOf<T>, bool),
        /// (account_id)
        SetAdmin(T::AccountId),
        /// (asset_id, erc20_contract)
        Register(T::AssetId, H160),
    }

    /// Error for evm accounts module.
    #[pallet::error]
    pub enum Error<T> {
        /// AccountId has mapped
        AccountIdHasMapped,
        /// Eth address has mapped
        EthAddressHasMapped,
        /// Bad signature
        BadSignature,
        /// Invalid signature
        InvalidSignature,
        /// Eth address has not mapped
        EthAddressHasNotMapped,
        /// AssetId has mapped
        AssetIdHasMapped,
        /// Erc20 contract address has mapped
        ContractAddressHasMapped,
        /// Erc20 contract address has not mapped
        ContractAddressHasNotMapped,
        /// Failed Erc20 contract call
        ExecutedFailed,
        /// Require admin authority
        RequireAdmin,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        DispatchError: From<<<T as pallet_evm::Config>::Runner as pallet_evm::Runner<T>>::Error>,
    {
        /// Claim account mapping between Substrate accounts and EVM accounts.
        /// Ensure eth_address has not been mapped.
        ///
        /// - `eth_address`: The address to bind to the caller's account
        /// - `eth_signature`: A signature generated by the address to prove ownership
        #[pallet::weight(100_000_000u64)]
        #[transactional]
        pub fn claim_account(
            origin: OriginFor<T>,
            eth_address: H160,
            eth_signature: EcdsaSignature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // ensure account_id and eth_address has not been mapped
            ensure!(
                !EvmAccounts::<T>::contains_key(&who),
                Error::<T>::AccountIdHasMapped
            );
            ensure!(
                !SubAccounts::<T>::contains_key(eth_address),
                Error::<T>::EthAddressHasMapped
            );

            // recover evm address from signature
            let address = eth_recover(&eth_signature, &who.using_encoded(to_ascii_hex), &[][..])
                .ok_or(Error::<T>::BadSignature)?;

            ensure!(eth_address == address, Error::<T>::InvalidSignature);

            SubAccounts::<T>::insert(eth_address, &who);
            EvmAccounts::<T>::insert(&who, eth_address);

            Self::deposit_event(Event::ClaimAccount(who, eth_address));

            Ok(())
        }

        #[pallet::weight(100_000_000u64)]
        #[transactional]
        pub fn deposit(
            origin: OriginFor<T>,
            asset_id: T::AssetId,
            amount: T::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. check evm account
            let evm_account = Self::evm_accounts(&who).ok_or(Error::<T>::EthAddressHasNotMapped)?;

            // 2. burn asset
            let _ = pallet_assets::Pallet::<T>::burn_from(asset_id, &who, amount)?;

            // 3. mint erc20
            let erc20 = Self::erc20s(asset_id).ok_or(Error::<T>::ContractAddressHasNotMapped)?;

            let inputs = mint_into_encode(evm_account, amount.unique_saturated_into());

            Self::call_evm(erc20, inputs)?;

            Self::deposit_event(Event::DepositExecuted(
                asset_id,
                who,
                evm_account,
                amount,
                erc20,
            ));

            Ok(())
        }

        #[pallet::weight(100_000_000u64)]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            asset_id: T::AssetId,
            amount: T::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. check evm account
            let evm_account = Self::evm_accounts(&who).ok_or(Error::<T>::EthAddressHasNotMapped)?;

            // 2. burn erc20
            let erc20 = Self::erc20s(asset_id).ok_or(Error::<T>::ContractAddressHasNotMapped)?;

            let inputs = burn_from_encode(evm_account, amount.unique_saturated_into());

            Self::call_evm(erc20, inputs)?;

            // 3. mint asset
            let _ = pallet_assets::Pallet::<T>::mint_into(asset_id, &who, amount)?;

            Self::deposit_event(Event::WithdrawExecuted(
                asset_id,
                who,
                evm_account,
                amount,
                erc20,
            ));

            Ok(())
        }

        #[pallet::weight(10_000_000u64)]
        pub fn teleport(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
            into_evm: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let evm_account =
                Self::evm_accounts(who.clone()).ok_or(Error::<T>::EthAddressHasNotMapped)?;

            let sub_account: T::AccountId = AddressMappingOf::<T>::into_account_id(evm_account);

            let (from, to) = if into_evm {
                (who.clone(), sub_account.clone())
            } else {
                (sub_account.clone(), who.clone())
            };

            <T as pallet_evm::Config>::Currency::transfer(
                &from,
                &to,
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            Self::deposit_event(Event::Teleport(
                who,
                evm_account,
                sub_account,
                amount,
                into_evm,
            ));

            Ok(Pays::No.into())
        }

        #[pallet::weight(10_000_000u64)]
        pub fn register(origin: OriginFor<T>, asset_id: T::AssetId, erc20: H160) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Some(who) == Self::admin_key(), Error::<T>::RequireAdmin);

            // ensure asset_id and erc20 address has not been mapped
            ensure!(
                !Erc20s::<T>::contains_key(&asset_id),
                Error::<T>::AssetIdHasMapped
            );
            ensure!(
                !AssetIds::<T>::contains_key(&erc20),
                Error::<T>::ContractAddressHasMapped
            );

            Erc20s::<T>::insert(asset_id, erc20);
            AssetIds::<T>::insert(erc20, asset_id);

            Self::deposit_event(Event::Register(asset_id, erc20));

            Ok(())
        }

        #[pallet::weight(1_000_000u64)]
        pub fn set_admin(
            origin: OriginFor<T>,
            new_admin: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let new_admin = T::Lookup::lookup(new_admin)?;

            Admin::<T>::mutate(|admin| *admin = Some(new_admin.clone()));

            Self::deposit_event(Event::SetAdmin(new_admin));

            Ok(Pays::No.into())
        }
    }
}

impl<T: Config> Pallet<T>
where
    DispatchError: From<<<T as pallet_evm::Config>::Runner as pallet_evm::Runner<T>>::Error>,
{
    fn call_evm(erc20: H160, inputs: Vec<u8>) -> DispatchResult {
        let info = T::Runner::call(
            T::EvmCaller::get(),
            erc20,
            inputs,
            U256::default(),
            3_000_000,
            None,
            None,
            None,
            Vec::new(),
            T::config()
        )?;

        match info.exit_reason {
            ExitReason::Succeed(_) => Ok(()),
            _ => Err(Error::<T>::ExecutedFailed.into()),
        }
    }
}
