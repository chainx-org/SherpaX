#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, ExistenceRequirement};
use core::marker::PhantomData;
use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use pallet_evm::{Precompile, AddressMapping};
use sp_core::{H160, U256, hexdisplay::HexDisplay};
use sp_runtime::{traits::UniqueSaturatedInto, AccountId32};
use codec::{Encode, Decode};
use frame_support::log;
use pallet_coming_id::ComingNFT;
use sp_std::vec;

pub struct NFT<T: pallet_evm::Config + pallet_coming_nft::Config> {
    _marker: PhantomData<T>,
}


impl<T: pallet_evm::Config + pallet_coming_nft::Config> NFT<T>
{
    fn process(
        input: &[u8]
    ) -> Result<bool, ExitError>{
        match input.len() {
            // withdraw balance
            // input = from(evm address, 20 bytes) + to(substrate pubkey, 32 bytes) + value(32 bytes)
            84 => {
                log::debug!(target: "coming-nft", "withdraw balance: call");

                Self::process_withdraw_balance(input)
                    .map_err(|err| {
                        log::warn!(target: "coming-nft", "withdraw balance: err = {:?}", err);
                        err
                    })?;

                log::debug!(target: "coming-nft", "withdraw balance: success");

                Ok(true)
            },
            // withdraw cid
            // input = from(evm address, 20 bytes) + to(substrate pubkey, 32 bytes) + cid(8 bytes)
            60 => {
                log::debug!(target: "coming-nft", "withdraw cid: call");

                Self::process_withdraw_cid(input)
                    .map_err(|err| {
                        log::warn!(target: "coming-nft", "withdraw cid: err = {:?}", err);
                        err
                    })?;

                log::debug!(target: "coming-nft", "withdraw cid: success");

                Ok(true)
            },
            // match owner
            // input = from(evm address, 20 bytes) + cid(8 bytes)
            28 => {
                log::debug!(target: "coming-nft", "match owner: call");

                let is_match = Self::process_match_owner(input)
                    .map_err(|err| {
                        log::warn!(target: "coming-nft", "match owner: err = {:?}", err);
                        err
                })?;

                log::debug!(target: "coming-nft", "match owner: {:?}", is_match);

                Ok(is_match)
            },
            // match operator of cid
            // input = from(evm address, 20 bytes) + cid(8 bytes) + padding(1 bytes)
            29 => {
                log::debug!(target: "coming-nft", "match operator: call");

                let is_operator = Self::process_match_operator(input)
                    .map_err(|err| {
                        log::warn!(target: "coming-nft", "match operator: err = {:?}", err);
                        err
                    })?;

                log::debug!(target: "coming-nft", "match operator: {:?}", is_operator);

                Ok(is_operator)
            },
            // get approved for all
            // input = owner(evm address, 20 bytes) + operator(evm address, 20 bytes)
            40 => {
                log::debug!(target: "coming-nft", "get approved: call");

                let approved = Self::process_get_approved(input)
                    .map_err(|err| {
                        log::warn!(target: "coming-nft", "get approved: err = {:?}", err);
                        err
                    })?;

                log::debug!(target: "coming-nft", "get approved: {:?}", approved);

                Ok(approved)
            },
            // transferFrom cid
            // input = operator(evm address, 20 bytes) + from(evm address, 20 bytes) + to(evm address, 20 bytes) + cid(8 bytes)
            68 => {
                log::debug!(target: "coming-nft", "transfer from: call");

                Self::process_transfer_from(input)
                    .map_err(|err| {
                        log::warn!(target: "coming-nft", "transfer from: err = {:?}", err);
                        err
                    })?;

                log::debug!(target: "coming-nft", "transfer from: success");

                Ok(true)
            },
            // approve
            // input = owner(evm address, 20 bytes) + operator(evm address, 20 bytes) + cid(8 bytes)
            48 => {
                log::debug!(target: "coming-nft", "approve: call");

                Self::process_approve(input)
                    .map_err(|err| {
                        log::warn!(target: "coming-nft", "approve: err = {:?}", err);
                        err
                    })?;

                log::debug!(target: "coming-nft", "approve: success");

                Ok(true)
            },
            // set approval for all
            // input = owner(evm address, 20 bytes) + operator(evm address, 20 bytes) + approved(1 bytes)
            41 => {
                log::debug!(target: "coming-nft", "set approval all: call");

                Self::process_set_approval_all(input)
                    .map_err(|err| {
                        log::warn!(target: "coming-nft", "approve: err = {:?}", err);
                        err
                    })?;

                log::debug!(target: "coming-nft", "set approval all: success");

                Ok(true)
            },
            _ => {
                log::warn!(target: "coming-nft", "invalid input: {:?}", input);

                Err(ExitError::Other("invalid input".into()))
            }
        }
    }

    fn account_from_address(
        address: &[u8]
    ) -> Result<T::AccountId, ExitError> {
        frame_support::ensure!(address.len() == 20, ExitError::Other("invalid address".into()));

        let from = H160::from_slice(&address[0..20]);

        Ok(T::AddressMapping::into_account_id(from))
    }

    fn account_from_pubkey(
        pubkey: &[u8]
    ) -> Result<T::AccountId, ExitError> {
        frame_support::ensure!(pubkey.len() == 32, ExitError::Other("invalid pubkey".into()));

        let mut target = [0u8; 32];
        target[0..32].copy_from_slice(&pubkey[0..32]);

        T::AccountId::decode(&mut &AccountId32::new(target).encode()[..])
            .map_err(|_| ExitError::Other("decode AccountId32 failed".into()))
    }

    fn balance(value: &[u8]) -> Result<u128, ExitError> {
        frame_support::ensure!(value.len() == 32, ExitError::Other("invalid balance".into()));

        Ok(U256::from_big_endian(&value[0..32]).low_u128())
    }

    fn parse_cid(value: &[u8]) -> Result<u64, ExitError> {
        frame_support::ensure!(value.len() == 8, ExitError::Other("invalid cid".into()));

        let mut cid = [0u8; 8];
        cid[0..8].copy_from_slice(&value[0..8]);

        Ok(u64::from_be_bytes(cid))
    }

    fn parse_approved(value: &u8) -> Result<bool, ExitError> {
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ExitError::Other("invalid approved".into()))
        }
    }

    fn process_withdraw_balance(
        input: &[u8]
    ) -> Result<(), ExitError> {
        let from = Self::account_from_address(&input[0..20])?;
        let to = Self::account_from_pubkey(&input[20..52])?;
        let balance = Self::balance(&input[52..84])?;

        log::debug!(target: "coming-nft", "from(evm): {:?}", H160::from_slice(&input[0..20]));
        log::debug!(target: "coming-nft", "from(sub): {:?}", HexDisplay::from(&from.encode()));
        log::debug!(target: "coming-nft", "to(sub): {:?}", HexDisplay::from(&to.encode()));
        log::debug!(target: "coming-nft", "value(sub): {:?}", balance);

        T::Currency::transfer(
            &from,
            &to,
            balance.unique_saturated_into(),
            ExistenceRequirement::AllowDeath,
        ).map_err(|err| {
            ExitError::Other(sp_std::borrow::Cow::Borrowed(err.into()))
        })
    }

    fn process_withdraw_cid(
        input: &[u8]
    ) -> Result<(), ExitError> {
        let from = Self::account_from_address(&input[0..20])?;
        let to = Self::account_from_pubkey(&input[20..52])?;
        let cid = Self::parse_cid(&input[52..60])?;

        T::ComingNFT::transfer(
            &from,
            cid,
            &to
        ).map_err(|err| {
            ExitError::Other(sp_std::borrow::Cow::Borrowed(err.into()))
        })
    }

    fn process_match_owner(
        input: &[u8]
    ) -> Result<bool, ExitError> {
        let from = Self::account_from_address(&input[0..20])?;
        let cid = Self::parse_cid(&input[20..28])?;

        match T::ComingNFT::owner_of_cid(cid) {
            Some(owner) if owner == from => Ok(true),
            _ => Ok(false)
        }
    }

    fn process_match_operator(
        input: &[u8]
    ) -> Result<bool, ExitError> {
        let from = Self::account_from_address(&input[0..20])?;
        let cid = Self::parse_cid(&input[20..28])?;

        match T::ComingNFT::get_approved(cid) {
            Some(operator) if operator == from => Ok(true),
            _ => Ok(false)
        }
    }

    fn process_get_approved(
        input: &[u8]
    ) -> Result<bool, ExitError> {
        let owner = Self::account_from_address(&input[0..20])?;
        let operator = Self::account_from_address(&input[20..40])?;

        Ok(T::ComingNFT::is_approved_for_all(&owner, &operator))
    }

    fn process_transfer_from(
        input: &[u8]
    ) -> Result<(), ExitError> {
        let operator = Self::account_from_address(&input[0..20])?;
        let from = Self::account_from_address(&input[20..40])?;
        let to = Self::account_from_address(&input[40..60])?;
        let cid = Self::parse_cid(&input[60..68])?;

        T::ComingNFT::transfer_from(
            &operator,
            &from,
            &to,
            cid
        ).map_err(|err| {
            ExitError::Other(sp_std::borrow::Cow::Borrowed(err.into()))
        })
    }

    fn process_approve(
        input: &[u8]
    ) -> Result<(), ExitError> {
        let owner = Self::account_from_address(&input[0..20])?;
        let operator = Self::account_from_address(&input[20..40])?;
        let cid = Self::parse_cid(&input[40..48])?;

        T::ComingNFT::approve(
            &owner,
            &operator,
            cid,
        ).map_err(|err| {
            ExitError::Other(sp_std::borrow::Cow::Borrowed(err.into()))
        })
    }

    fn process_set_approval_all(
        input: &[u8]
    ) -> Result<(), ExitError> {
        let owner = Self::account_from_address(&input[0..20])?;
        let operator = Self::account_from_address(&input[20..40])?;
        let approved = Self::parse_approved(&input[41])?;

        T::ComingNFT::set_approval_for_all(
            &owner,
            &operator,
            approved,
        ).map_err(|err| {
            ExitError::Other(sp_std::borrow::Cow::Borrowed(err.into()))
        })
    }
}

impl<T> Precompile for NFT<T>
    where
        T: pallet_evm::Config + pallet_coming_nft::Config,
        T::AccountId: Decode,
{
    fn execute(
        input: &[u8],
        _target_gas: Option<u64>,
        context: &Context,
    ) -> Result<PrecompileOutput, ExitError> {

        log::debug!(target: "coming-nft", "caller: {:?}", context.caller);

        const BASE_GAS_COST: u64 = 45_000;

        // Refer: https://github.com/rust-ethereum/ethabi/blob/master/ethabi/src/encoder.rs#L144
        let mut out = vec![0u8;32];

        if Self::process(input)? {
            out[31] = 1u8;
        }

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: BASE_GAS_COST,
            output: out.to_vec(),
            logs: Default::default(),
        })
    }
}
