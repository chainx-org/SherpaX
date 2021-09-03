#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, ExistenceRequirement};
use core::marker::PhantomData;
use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use pallet_evm::{Precompile, AddressMapping};
use sp_core::{H160, U256, hexdisplay::HexDisplay};
use sp_runtime::{traits::UniqueSaturatedInto, AccountId32};
use codec::{Encode, Decode};
use frame_support::log;
pub struct Withdraw<T: pallet_evm::Config> {
    _marker: PhantomData<T>,
}

impl<T> Precompile for Withdraw<T>
    where
        T: pallet_evm::Config,
        T::AccountId: Decode,
{
    fn execute(
        input: &[u8],
        _target_gas: Option<u64>,
        context: &Context,
    ) -> core::result::Result<PrecompileOutput, ExitError> {
        log::debug!(target: "evm", "withdraw: input: {:?}", input);
        log::debug!(target: "evm", "withdraw: caller: {:?}", context.caller);

        const BASE_GAS_COST: u64 = 45_000;

        // input = from(evm address, 20 bytes) + to(substrate pubkey, 32 bytes) + value(32 bytes)
        if input.len() != 20 + 32 + 32 {
            return Err(ExitError::Other("invalid input".into()))
        }

        let from = H160::from_slice(&input[0..20]);
        log::debug!(target: "evm", "withdraw: from: {:?}", from);

        let address_account_id = T::AddressMapping::into_account_id(from);
        log::debug!(target: "evm", "withdraw: source: {:?}", HexDisplay::from(&address_account_id.encode()));

        let mut target = [0u8; 32];
        target[0..32].copy_from_slice(&input[20..52]);
        let dest = T::AccountId::decode(&mut &AccountId32::new(target).encode()[..])
            .map_err(|_| ExitError::Other("decode failed".into()))?;
        log::debug!(target: "evm", "withdraw: target: {:?}", HexDisplay::from(&target));

        let value = U256::from_big_endian(&input[52..84])
            .low_u128().unique_saturated_into();
        log::debug!(target: "evm", "withdraw: value: {:?}", value);

        T::Currency::transfer(
            &address_account_id,
            &dest,
            value,
            ExistenceRequirement::AllowDeath,
        ).map_err(|err| {
            log::debug!(target: "evm", "withdraw: err = {:?}", err);

            ExitError::OutOfFund
        })?;

        log::debug!(target: "evm", "withdraw: success");

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Stopped,
            cost: BASE_GAS_COST,
            output: Default::default(),
            logs: Default::default(),
        })
    }
}
