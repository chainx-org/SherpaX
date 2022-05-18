// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::dispatch::{DispatchError, DispatchResult};
use sp_std::{convert::TryFrom, prelude::Vec};

use crate::types::{ScriptInfo, TrusteeInfoConfig, TrusteeIntentionProps, TrusteeSessionInfo};
use light_bitcoin::chain::Transaction;
use sherpax_primitives::ReferralId;
use xp_assets_registrar::Chain;

pub trait BytesLike: Into<Vec<u8>> + TryFrom<Vec<u8>> {}
impl<T: Into<Vec<u8>> + TryFrom<Vec<u8>>> BytesLike for T {}

pub trait ChainProvider {
    fn chain() -> Chain;
}

pub trait TotalSupply<Balance> {
    fn total_supply() -> Balance;
}

impl<Balance: Default> TotalSupply<Balance> for () {
    fn total_supply() -> Balance {
        Balance::default()
    }
}

pub trait ProposalProvider {
    type WithdrawalProposal;

    fn get_withdrawal_proposal() -> Option<Self::WithdrawalProposal>;
}

impl ProposalProvider for () {
    type WithdrawalProposal = ();

    fn get_withdrawal_proposal() -> Option<Self::WithdrawalProposal> {
        None
    }
}

pub trait TrusteeForChain<
    AccountId,
    BlockNumber,
    TrusteeEntity: BytesLike,
    TrusteeAddress: BytesLike,
>
{
    fn check_trustee_entity(raw_addr: &[u8]) -> Result<TrusteeEntity, DispatchError>;

    fn generate_trustee_session_info(
        props: Vec<(AccountId, TrusteeIntentionProps<AccountId, TrusteeEntity>)>,
        config: TrusteeInfoConfig,
    ) -> Result<
        (
            TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>,
            ScriptInfo<AccountId>,
        ),
        DispatchError,
    >;
}

impl<AccountId, BlockNumber, TrusteeEntity: BytesLike, TrusteeAddress: BytesLike>
    TrusteeForChain<AccountId, BlockNumber, TrusteeEntity, TrusteeAddress> for ()
{
    fn check_trustee_entity(_: &[u8]) -> Result<TrusteeEntity, DispatchError> {
        Err("NoTrustee".into())
    }

    fn generate_trustee_session_info(
        _: Vec<(AccountId, TrusteeIntentionProps<AccountId, TrusteeEntity>)>,
        _: TrusteeInfoConfig,
    ) -> Result<
        (
            TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>,
            ScriptInfo<AccountId>,
        ),
        DispatchError,
    > {
        Err("NoTrustee".into())
    }
}

pub trait TrusteeSession<AccountId, BlockNumber, TrusteeAddress: BytesLike> {
    fn trustee_session(
        number: u32,
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError>;

    fn current_trustee_session(
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError>;

    fn current_proxy_account() -> Result<Vec<AccountId>, DispatchError>;

    fn last_trustee_session(
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError>;

    fn trustee_transition_state(chain: Chain) -> bool;

    #[cfg(feature = "std")]
    fn genesis_trustee(chain: Chain, init: &[AccountId]);
}

impl<AccountId, BlockNumber, TrusteeAddress: BytesLike>
    TrusteeSession<AccountId, BlockNumber, TrusteeAddress> for ()
{
    fn trustee_session(
        _: u32,
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError> {
        Err("NoTrustee".into())
    }

    fn current_trustee_session(
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError> {
        Err("NoTrustee".into())
    }

    fn current_proxy_account() -> Result<Vec<AccountId>, DispatchError> {
        Err("NoTrustee".into())
    }

    fn last_trustee_session(
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError> {
        Err("NoTrustee".into())
    }

    fn trustee_transition_state(_: Chain) -> bool {
        false
    }

    #[cfg(feature = "std")]
    fn genesis_trustee(_: Chain, _: &[AccountId]) {}
}

pub trait TrusteeInfoUpdate {
    /// Update the trustee trasition status when the renewal of the trustee is completed
    fn update_transition_status(chain: Chain, status: bool, trans_amount: Option<u64>);
    /// Each withdrawal is completed to record the weight of the signer
    fn update_trustee_sig_record(
        chain: Chain,
        tx: Transaction,
        withdraw_amout: u64,
    ) -> DispatchResult;
}

impl TrusteeInfoUpdate for () {
    fn update_transition_status(_: Chain, _: bool, _: Option<u64>) {}

    fn update_trustee_sig_record(_: Chain, _: Transaction, _: u64) -> DispatchResult {
        Ok(())
    }
}

pub trait ReferralBinding<AccountId, AssetId> {
    fn update_binding(asset_id: &AssetId, who: &AccountId, referral_name: Option<ReferralId>);
    fn referral(asset_id: &AssetId, who: &AccountId) -> Option<AccountId>;
}

impl<AccountId, AssetId> ReferralBinding<AccountId, AssetId> for () {
    fn update_binding(_: &AssetId, _: &AccountId, _: Option<ReferralId>) {}
    fn referral(_: &AssetId, _: &AccountId) -> Option<AccountId> {
        None
    }
}

pub trait AddressBinding<AccountId, Address: Into<Vec<u8>>> {
    fn update_binding(chain: Chain, address: Address, who: AccountId);
    fn address(chain: Chain, address: Address) -> Option<AccountId>;
}

impl<AccountId, Address: Into<Vec<u8>>> AddressBinding<AccountId, Address> for () {
    fn update_binding(_: Chain, _: Address, _: AccountId) {}
    fn address(_: Chain, _: Address) -> Option<AccountId> {
        None
    }
}
