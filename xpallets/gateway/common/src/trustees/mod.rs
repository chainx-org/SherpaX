// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

pub mod bitcoin;
pub mod dogecoin;

use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    log::{error, warn},
    pallet_prelude::Get,
    traits::{fungibles::Mutate, SortedMembers},
};
use light_bitcoin::{
    chain::Transaction,
    keys::{Public, Signature},
    script::{Script, SignatureVersion, TransactionInputSigner},
};
use sp_runtime::traits::Zero;
use sp_std::{convert::TryFrom, marker::PhantomData, prelude::*};
use xp_assets_registrar::Chain;
use xpallet_support::traits::MultiSig;

use crate::{
    traits::{BytesLike, ChainProvider, TrusteeInfoUpdate, TrusteeSession},
    types::TrusteeSessionInfo,
    CheckedDiv, Config, Error, Event, Pallet, PreTotalSupply, SaturatedConversion, Saturating,
    TrusteeSessionInfoOf, TrusteeSigRecord, TrusteeTransitionStatus,
};

pub struct TrusteeSessionManager<T: Config, TrusteeAddress>(
    PhantomData<T>,
    PhantomData<TrusteeAddress>,
);

impl<T: Config, TrusteeAddress: BytesLike + ChainProvider>
    TrusteeSession<T::AccountId, T::BlockNumber, TrusteeAddress>
    for TrusteeSessionManager<T, TrusteeAddress>
{
    fn trustee_session(
        number: u32,
    ) -> Result<TrusteeSessionInfo<T::AccountId, T::BlockNumber, TrusteeAddress>, DispatchError>
    {
        let chain = TrusteeAddress::chain();
        let generic_info =
            Pallet::<T>::trustee_session_info_of(chain, number).ok_or_else(|| {
                error!(
                    target: "runtime::gateway::common",
                    "[trustee_session] Can not find session info, chain:{:?}, number:{}",
                    chain,
                    number
                );
                Error::<T>::InvalidTrusteeSession
            })?;
        let info = TrusteeSessionInfo::<T::AccountId, T::BlockNumber, TrusteeAddress>::try_from(
            generic_info,
        )
        .map_err(|_| Error::<T>::InvalidGenericData)?;
        Ok(info)
    }

    fn current_trustee_session(
    ) -> Result<TrusteeSessionInfo<T::AccountId, T::BlockNumber, TrusteeAddress>, DispatchError>
    {
        let chain = TrusteeAddress::chain();
        let number = Pallet::<T>::trustee_session_info_len(chain);
        Self::trustee_session(number)
    }

    fn current_proxy_account() -> Result<Vec<T::AccountId>, DispatchError> {
        Ok(Self::current_trustee_session()?
            .trustee_list
            .iter()
            .filter_map(|info| {
                match Pallet::<T>::trustee_intention_props_of(&info.0, TrusteeAddress::chain()) {
                    None => None,
                    Some(n) => n.0.proxy_account,
                }
            })
            .collect::<Vec<T::AccountId>>())
    }

    fn last_trustee_session(
    ) -> Result<TrusteeSessionInfo<T::AccountId, T::BlockNumber, TrusteeAddress>, DispatchError>
    {
        let chain = TrusteeAddress::chain();
        let number = match Pallet::<T>::trustee_session_info_len(chain).checked_sub(1) {
            Some(r) => r,
            None => u32::max_value(),
        };
        Self::trustee_session(number).map_err(|err| {
            warn!(
                target: "runtime::gateway::common",
                "[last_trustee_session] Last trustee session not exist yet for chain:{:?}",
                chain
            );
            err
        })
    }

    fn trustee_transition_state(chain: Chain) -> bool {
        Pallet::<T>::trustee_transition_status(chain)
    }

    #[cfg(feature = "std")]
    fn genesis_trustee(chain: Chain, trustees: &[T::AccountId]) {
        Pallet::<T>::transition_trustee_session_impl(chain, trustees.to_vec())
            .expect("trustee session transition can not fail; qed");
    }
}

pub struct TrusteeMultisigProvider<T: Config, C: ChainProvider>(PhantomData<T>, PhantomData<C>);
impl<T: Config, C: ChainProvider> TrusteeMultisigProvider<T, C> {
    pub fn new() -> Self {
        TrusteeMultisigProvider::<_, _>(Default::default(), Default::default())
    }
}

impl<T: Config, C: ChainProvider> MultiSig<T::AccountId> for TrusteeMultisigProvider<T, C> {
    fn multisig() -> T::AccountId {
        Pallet::<T>::trustee_multisig_addr(C::chain())
    }
}

impl<T: Config, C: ChainProvider> SortedMembers<T::AccountId> for TrusteeMultisigProvider<T, C> {
    fn sorted_members() -> Vec<T::AccountId> {
        vec![Self::multisig()]
    }
}

impl<T: Config> TrusteeInfoUpdate for Pallet<T> {
    fn update_transition_status(chain: Chain, status: bool, trans_amount: Option<u64>) {
        // The renewal of the trustee is completed, the current trustee information is replaced
        // and the number of multiple signings is archived.
        if Self::trustee_transition_status(chain) && !status {
            let last_session_num = Self::trustee_session_info_len(chain).saturating_sub(1);
            TrusteeSessionInfoOf::<T>::mutate(chain, last_session_num, |info| match info {
                None => {
                    warn!(
                        target: "runtime::gateway::common",
                        "[last_trustee_session] Last trustee session not exist for chain:{:?}, session_num:{}",
                        chain, last_session_num
                    );
                }
                Some(trustee) => {
                    for i in 0..trustee.0.trustee_list.len() {
                        trustee.0.trustee_list[i].1 =
                            Self::trustee_sig_record(chain, &trustee.0.trustee_list[i].0)
                                .unwrap_or(0u64);
                    }
                    let asset_id = match chain {
                        Chain::Bitcoin => T::BtcAssetId::get(),
                        Chain::Dogecoin => T::DogeAssetId::get(),
                        _ => T::BtcAssetId::get(),
                    };
                    let total_apply: T::Balance = Self::pre_total_supply(asset_id);
                    let reward_amount: T::Balance = trans_amount
                        .unwrap_or(0u64)
                        .saturated_into::<T::Balance>()
                        .saturating_sub(total_apply)
                        .max(0u64.saturated_into())
                        .saturating_mul(6u64.saturated_into())
                        .checked_div(&10u64.saturated_into::<T::Balance>())
                        .unwrap_or_else(|| 0u64.saturated_into());

                    if let Some(multi_account) = trustee.0.multi_account.clone() {
                        if !reward_amount.is_zero() {
                            match pallet_assets::Pallet::<T>::mint_into(
                                asset_id,
                                &multi_account,
                                reward_amount,
                            ) {
                                Ok(()) => {
                                    PreTotalSupply::<T>::remove(asset_id);
                                    Pallet::<T>::deposit_event(Event::<T>::TransferAssetReward(
                                        multi_account,
                                        asset_id,
                                        reward_amount,
                                    ));
                                }
                                Err(err) => {
                                    error!(
                                        target: "runtime::gateway::common",
                                        "[deposit_token] Deposit error:{:?}, must use root to fix it",
                                        err
                                    );
                                }
                            };
                        }
                    }
                    let end_height = frame_system::Pallet::<T>::block_number();
                    trustee.0.end_height = Some(end_height);
                }
            });
            TrusteeSigRecord::<T>::remove_all(None);
        }

        TrusteeTransitionStatus::<T>::insert(chain, status);
    }

    fn update_trustee_sig_record(
        chain: Chain,
        tx: Transaction,
        withdraw_amount: u64,
    ) -> DispatchResult {
        let signed_trustees = match chain {
            Chain::Bitcoin => {
                let script = tx.inputs()[0].script_witness[1].as_slice();
                Self::agg_pubkey_info(script)
            }
            Chain::Dogecoin => {
                let mut signed_trustees = vec![];
                let script: Script = tx.inputs[0].script_sig.clone().into();
                let (sigs, redeem_script) = script
                    .extract_multi_scriptsig()
                    .map_err(|_| Error::<T>::InvalidScriptSig)?;
                let (pubkeys, _, _) = redeem_script
                    .parse_redeem_script()
                    .ok_or(Error::<T>::InvalidRedeemScript)?;

                let tx_signer: TransactionInputSigner = tx.clone().into();

                let sighashtype = 1; // Sighsh all

                // when use WitnessV0, the `input_amount` must set value
                let sighash = tx_signer.signature_hash(
                    tx.inputs[0].previous_output.index as usize,
                    0,
                    &redeem_script,
                    SignatureVersion::Base,
                    sighashtype,
                );

                for sig in sigs.iter() {
                    let signature: Signature = sig.as_slice().into();
                    for p in pubkeys.iter() {
                        let pubkey = Public::from_slice(p.as_slice())
                            .map_err(|_| Error::<T>::InvalidPublicKey)?;
                        if pubkey.verify(&sighash, &signature).unwrap_or(false) {
                            let trustee = Self::hot_pubkey_info(p.as_slice());
                            signed_trustees.push(trustee);
                        }
                    }
                }

                signed_trustees
            }
            _ => vec![],
        };

        signed_trustees.into_iter().for_each(|trustee| {
            let amount = if trustee == Self::trustee_admin() {
                withdraw_amount
                    .saturating_mul(Self::trustee_admin_multiply())
                    .saturating_div(10)
            } else {
                withdraw_amount
            };
            if TrusteeSigRecord::<T>::contains_key(chain, &trustee) {
                TrusteeSigRecord::<T>::mutate(chain, &trustee, |record| {
                    if let Some(r) = record {
                        *r += amount
                    }
                });
            } else {
                TrusteeSigRecord::<T>::insert(chain, trustee, amount);
            }
        });
        Ok(())
    }
}
