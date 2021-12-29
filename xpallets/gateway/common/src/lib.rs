// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

//! this module is for bridge common parts
//! define trait and type for
//! `trustees`, `crosschain binding` and something others

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default, clippy::type_complexity)]

#[cfg(any(feature = "runtime-benchmarks", test))]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod binding;
pub mod traits;
pub mod trustees;
pub mod types;
pub mod utils;
pub mod weights;

use frame_support::traits::ExistenceRequirement;
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    log::{error, info},
    traits::{ChangeMembers, Currency, Get},
};
use frame_system::{ensure_root, ensure_signed};
use sp_runtime::traits::{CheckedDiv, Saturating, Zero};
use sp_runtime::SaturatedConversion;
use sp_std::{collections::btree_map::BTreeMap, convert::TryFrom, prelude::*};

use self::traits::{TrusteeForChain, TrusteeInfoUpdate, TrusteeSession};
use self::types::{
    GenericTrusteeIntentionProps, GenericTrusteeSessionInfo, TrusteeInfoConfig,
    TrusteeIntentionProps,
};
pub use self::weights::WeightInfo;
use crate::trustees::bitcoin::BtcTrusteeAddrInfo;
use crate::types::{RewardInfo, ScriptInfo, TrusteeSessionInfo};
pub use pallet::*;
use sherpax_primitives::{AddrStr, ChainAddress, Text};
use xp_assets_registrar::Chain;
use xp_runtime::Memo;
use xpallet_gateway_records::{
    ChainT, Withdrawal, WithdrawalLimit, WithdrawalRecordId, WithdrawalState,
};
use xpallet_support::traits::{MultisigAddressFor, Validator};

type Balanceof<T> =
    <<T as xpallet_gateway_records::Config>::Currency as frame_support::traits::Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::fungibles::Inspect;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + xpallet_gateway_records::Config
        + pallet_elections_phragmen::Config
        + pallet_assets::Config
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Validator: Validator<Self::AccountId>;

        type DetermineMultisigAddress: MultisigAddressFor<Self::AccountId>;

        type CouncilOrigin: EnsureOrigin<Self::Origin>;

        // for bitcoin
        type Bitcoin: ChainT<Self::AssetId, Self::Balance>;
        type BitcoinTrustee: TrusteeForChain<
            Self::AccountId,
            Self::BlockNumber,
            trustees::bitcoin::BtcTrusteeType,
            trustees::bitcoin::BtcTrusteeAddrInfo,
        >;
        type BitcoinTrusteeSessionProvider: TrusteeSession<
            Self::AccountId,
            Self::BlockNumber,
            trustees::bitcoin::BtcTrusteeAddrInfo,
        >;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a withdrawal.
        /// Withdraws some balances of `asset_id` to address `addr` of target chain.
        ///
        /// WithdrawalRecord State: `Applying`
        ///
        /// NOTE: `ext` is for the compatibility purpose, e.g., EOS requires a memo when doing the transfer.
        #[pallet::weight(< T as Config >::WeightInfo::withdraw())]
        pub fn withdraw(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] value: T::Balance,
            addr: AddrStr,
            ext: Memo,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                pallet_assets::Pallet::<T>::can_withdraw(asset_id, &who, value)
                    .into_result()
                    .is_ok(),
                Error::<T>::InvalidWithdrawal,
            );
            Self::verify_withdrawal(asset_id, value, &addr, &ext)?;

            xpallet_gateway_records::Pallet::<T>::withdraw(&who, asset_id, value, addr, ext)?;
            Ok(())
        }

        /// Cancel the withdrawal by the applicant.
        ///
        /// WithdrawalRecord State: `Applying` ==> `NormalCancel`
        #[pallet::weight(< T as Config >::WeightInfo::cancel_withdrawal())]
        pub fn cancel_withdrawal(origin: OriginFor<T>, id: WithdrawalRecordId) -> DispatchResult {
            let from = ensure_signed(origin)?;
            xpallet_gateway_records::Pallet::<T>::cancel_withdrawal(id, &from)
        }

        /// Setup the trustee info.
        ///
        /// The hot and cold public keys of the current trustee cannot be replaced at will. If they
        /// are randomly replaced, the hot and cold public keys of the current trustee before the
        /// replacement will be lost, resulting in the inability to reconstruct the `Mast` tree and
        /// generate the corresponding control block.
        ///
        /// There are two solutions:
        /// - the first is to record the hot and cold public keys for each
        /// trustee renewal, and the trustee can update the hot and cold public keys at will.
        /// - The second is to move these trusts into the `lttle_black_house` when it is necessary
        /// to update the hot and cold public keys of trusts, and renew the trustee.
        /// After the renewal of the trustee is completed, the hot and cold public keys can be
        /// updated.
        ///
        /// The second option is currently selected. `The time when the second option
        /// allows the hot and cold public keys to be updated is that the member is not in the
        /// current trustee and is not in a state of renewal of the trustee`.
        /// The advantage of the second scheme is that there is no need to change the storage
        /// structure and record the hot and cold public keys of previous trusts.
        /// The disadvantage is that the update of the hot and cold public keys requires the
        /// participation of the admin account and the user cannot update the hot and cold public
        /// keys at will.
        #[pallet::weight(< T as Config >::WeightInfo::setup_trustee())]
        pub fn setup_trustee(
            origin: OriginFor<T>,
            proxy_account: Option<T::AccountId>,
            chain: Chain,
            about: Text,
            hot_entity: Vec<u8>,
            cold_entity: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // make sure this person is a pre-selected trustee
            // or the trustee is in little black house
            ensure!(
                Self::generate_trustee_pool().contains(&who)
                    || Self::little_black_house().contains(&who),
                Error::<T>::NotTrusteePreselectedMember
            );

            ensure!(
                Self::ensure_not_current_trustee(&who) && !Self::trustee_transition_status(),
                Error::<T>::ExistCurrentTrustee
            );

            Self::setup_trustee_impl(who, proxy_account, chain, about, hot_entity, cold_entity)
        }

        /// Transition the trustee session.
        #[pallet::weight(< T as Config >::WeightInfo::transition_trustee_session())]
        pub fn transition_trustee_session(
            origin: OriginFor<T>,
            chain: Chain,
            new_trustees: Vec<T::AccountId>,
        ) -> DispatchResult {
            match ensure_signed(origin.clone()) {
                Ok(who) => {
                    if who != Self::trustee_multisig_addr(chain) {
                        return Err(Error::<T>::InvalidMultisig.into());
                    }
                }
                Err(_) => {
                    ensure_root(origin)?;
                }
            };

            info!(
                target: "runtime::gateway::common",
                "[transition_trustee_session] Try to transition trustees, chain:{:?}, new_trustees:{:?}",
                chain,
                new_trustees
            );
            Self::transition_trustee_session_impl(chain, new_trustees)
        }

        /// Move a current trustee into a small black room.
        ///
        /// This is to allow for timely replacement in the event of a problem with a particular trustee.
        /// The trustee will be moved into the small black room.
        ///
        /// This is called by the trustee admin and root.
        /// # <weight>
        /// Since this is a root call and will go into trustee election, we assume full block for now.
        /// # </weight>
        #[pallet::weight(100_000_000u64)]
        pub fn move_trust_to_black_room(
            origin: OriginFor<T>,
            trustees: Option<Vec<T::AccountId>>,
        ) -> DispatchResult {
            match ensure_signed(origin.clone()) {
                Ok(who) => {
                    if who != Self::trustee_admin() {
                        return Err(Error::<T>::NotTrusteeAdmin.into());
                    }
                }
                Err(_) => {
                    ensure_root(origin)?;
                }
            };

            info!(
                target: "runtime::gateway::common",
                "[move_trust_to_black_room] Try to move a trustee to black room, trustee:{:?}",
                trustees
            );

            if let Some(trustees) = trustees {
                LittleBlackHouse::<T>::mutate(|l| {
                    for trustee in trustees.iter() {
                        l.push(trustee.clone());
                    }
                });
            }

            Self::do_trustee_election()?;
            Ok(())
        }

        /// Automatic trustee transfer from relayer.
        ///
        /// Since the time of the function exectution only have 0.5 s during
        /// the initialization of parachain, the action of the trustee election
        /// is not supported, so the automatic trustee election is triggered by
        /// the Relayer.
        ///
        /// This is called by the relayer and root.
        #[pallet::weight(100_000_000u64)]
        pub fn auto_trustee_election(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            match ensure_signed(origin.clone()) {
                Ok(who) => {
                    if who != Self::relayer() {
                        return Err(Error::<T>::InvalidRelayer.into());
                    }
                }
                Err(_) => {
                    ensure_root(origin)?;
                }
            };

            Self::do_trustee_election()?;
            Ok(Pays::No.into())
        }

        /// Force trustee election
        ///
        /// Mandatory trustee renewal if the current trustee is not doing anything
        ///
        /// This is called by the root.
        #[pallet::weight(100_000_000u64)]
        pub fn force_trustee_election(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            Self::update_transition_status(false);

            Ok(())
        }

        /// Regenerate the trustee's aggregated public key information.
        ///
        /// There is some problem with generating the number of aggregated
        /// public keys, regenerate the aggregated public key information
        /// after the repair is completed, and then remove the call.
        ///
        /// This is called by the root.
        #[pallet::weight(100_000_000u64)]
        pub fn regenerate_aggpubkey(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;

            info!(
                target: "runtime::gateway::common",
                "[regenerate_aggpubkey] Try to regenerate the aggregated public key information"
            );
            let trustee_session = T::BitcoinTrusteeSessionProvider::current_trustee_session()?;
            let trustees = trustee_session
                .trustee_list
                .into_iter()
                .unzip::<_, _, _, Vec<u64>>()
                .0;
            AggPubkeyInfo::<T>::remove_all(None);
            let info = Self::try_generate_session_info(Chain::Bitcoin, trustees)?;
            let session_number = Self::trustee_session_info_len(Chain::Bitcoin);

            for index in 0..info.1.agg_pubkeys.len() {
                AggPubkeyInfo::<T>::insert(
                    &info.1.agg_pubkeys[index],
                    info.1.personal_accounts[index].clone(),
                );
            }
            // There is no multi-signature address inserted in info so
            // the event will not display the multi-signature address.
            Self::deposit_event(Event::<T>::TrusteeSetChanged(
                Chain::Bitcoin,
                session_number,
                info.0,
                info.1.agg_pubkeys.len() as u32,
            ));
            Ok(())
        }

        /// Set the state of withdraw record by the trustees.
        #[pallet::weight(< T as Config >::WeightInfo::set_withdrawal_state())]
        pub fn set_withdrawal_state(
            origin: OriginFor<T>,
            #[pallet::compact] id: WithdrawalRecordId,
            state: WithdrawalState,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            let map = Self::trustee_multisigs();
            let chain = map
                .into_iter()
                .find_map(|(chain, multisig)| if from == multisig { Some(chain) } else { None })
                .ok_or(Error::<T>::InvalidMultisig)?;

            xpallet_gateway_records::Pallet::<T>::set_withdrawal_state_by_trustees(id, chain, state)
        }

        /// Set the config of trustee information.
        ///
        /// This is a root-only operation.
        #[pallet::weight(< T as Config >::WeightInfo::set_trustee_info_config())]
        pub fn set_trustee_info_config(
            origin: OriginFor<T>,
            chain: Chain,
            config: TrusteeInfoConfig,
        ) -> DispatchResult {
            match T::CouncilOrigin::ensure_origin(origin.clone()) {
                Err(_) => {
                    ensure_root(origin)?;
                }
                _ => (),
            };
            TrusteeInfoConfigOf::<T>::insert(chain, config);
            Ok(())
        }

        /// Dangerous! Be careful to set TrusteeTransitionDuration
        #[pallet::weight(< T as Config >::WeightInfo::change_trustee_transition_duration())]
        pub fn change_trustee_transition_duration(
            origin: OriginFor<T>,
            duration: T::BlockNumber,
        ) -> DispatchResult {
            match T::CouncilOrigin::ensure_origin(origin.clone()) {
                Err(_) => {
                    ensure_root(origin)?;
                }
                _ => (),
            };

            TrusteeTransitionDuration::<T>::put(duration);
            Ok(())
        }

        /// Set relayer.
        ///
        /// This is a root-only operation.
        #[pallet::weight(< T as Config >::WeightInfo::set_trustee_admin())]
        pub fn set_relayer(origin: OriginFor<T>, relayer: T::AccountId) -> DispatchResult {
            match T::CouncilOrigin::ensure_origin(origin.clone()) {
                Err(_) => {
                    ensure_root(origin)?;
                }
                _ => (),
            };
            Relayer::<T>::put(relayer);
            Ok(())
        }

        /// Set the trustee admin.
        ///
        /// This is a root-only operation.
        /// The trustee admin is the account who can change the trustee list.
        #[pallet::weight(< T as Config >::WeightInfo::set_trustee_admin())]
        pub fn set_trustee_admin(
            origin: OriginFor<T>,
            admin: T::AccountId,
            chain: Chain,
        ) -> DispatchResult {
            match T::CouncilOrigin::ensure_origin(origin.clone()) {
                Err(_) => {
                    ensure_root(origin)?;
                }
                _ => (),
            };
            Self::trustee_intention_props_of(&admin, chain).ok_or_else::<DispatchError, _>(
                || {
                    error!(
                        target: "runtime::gateway::common",
                        "[set_trustee_admin] admin {:?} has not in TrusteeIntentionPropertiesOf",
                        admin
                    );
                    Error::<T>::NotRegistered.into()
                },
            )?;
            TrusteeAdmin::<T>::put(admin);
            Ok(())
        }

        /// A certain trustee member declares the reward
        #[pallet::weight(< T as Config >::WeightInfo::tranfer_trustee_reward())]
        pub fn tranfer_trustee_reward(
            origin: OriginFor<T>,
            session_num: i32,
            amount: Balanceof<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let session_num: u32 = if session_num < 0 {
                match session_num {
                    -1i32 => Self::trustee_session_info_len(Chain::Bitcoin),
                    -2i32 => Self::trustee_session_info_len(Chain::Bitcoin)
                        .checked_sub(1)
                        .ok_or(Error::<T>::InvalidSessionNum)?,
                    _ => return Err(Error::<T>::InvalidSessionNum.into()),
                }
            } else {
                session_num as u32
            };
            ensure!(
                Self::trustee_session_info_len(Chain::Bitcoin) > session_num,
                Error::<T>::InvalidSessionNum
            );
            let trustee_info = T::BitcoinTrusteeSessionProvider::trustee_session(session_num)?;
            Self::apply_tranfer_trustee_reward(&who, session_num, &trustee_info, amount)?;
            Self::apply_claim_trustee_reward(&who, session_num, &trustee_info)
        }

        /// A certain trustee member declares the reward
        #[pallet::weight(< T as Config >::WeightInfo::claim_trustee_reward())]
        pub fn claim_trustee_reward(origin: OriginFor<T>, session_num: i32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let session_num: u32 = if session_num < 0 {
                match session_num {
                    -1i32 => Self::trustee_session_info_len(Chain::Bitcoin),
                    -2i32 => Self::trustee_session_info_len(Chain::Bitcoin)
                        .checked_sub(1)
                        .ok_or(Error::<T>::InvalidSessionNum)?,
                    _ => return Err(Error::<T>::InvalidSessionNum.into()),
                }
            } else {
                session_num as u32
            };
            let trustee_info = T::BitcoinTrusteeSessionProvider::trustee_session(session_num)?;

            Self::apply_claim_trustee_reward(&who, session_num, &trustee_info)
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A (potential) trustee set the required properties. [who, chain, trustee_props]
        SetTrusteeProps(
            T::AccountId,
            Chain,
            GenericTrusteeIntentionProps<T::AccountId>,
        ),
        /// An account set its referral_account of some chain. [who, chain, referral_account]
        ReferralBinded(T::AccountId, Chain, T::AccountId),
        /// The trustee set of a chain was changed. [chain, session_number, session_info, script_info]
        TrusteeSetChanged(
            Chain,
            u32,
            GenericTrusteeSessionInfo<T::AccountId, T::BlockNumber>,
            u32,
        ),
        /// Treasury transfer to trustee. [source, target, chain, session_number, reward_total]
        TransferTrusteeReward(T::AccountId, T::AccountId, Chain, u32, Balanceof<T>),
        /// The reward of trustee is assigned. [who, chain, session_number, reward_info]
        TrusteeRewardComplete(
            T::AccountId,
            Chain,
            u32,
            RewardInfo<T::AccountId, Balanceof<T>>,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// the value of withdrawal less than than the minimum value
        InvalidWithdrawal,
        /// convert generic data into trustee session info error
        InvalidGenericData,
        /// trustee session info not found
        InvalidTrusteeSession,
        /// exceed the maximum length of the about field of trustess session info
        InvalidAboutLen,
        /// invalid multisig
        InvalidMultisig,
        /// unsupported chain
        NotSupportedChain,
        /// existing duplicate account
        DuplicatedAccountId,
        /// not registered as trustee
        NotRegistered,
        /// just allow validator to register trustee
        NotValidator,
        /// just allow trustee admin to remove trustee
        NotTrusteeAdmin,
        /// just allow trustee preselected members to set their trustee information
        NotTrusteePreselectedMember,
        /// invalid public key
        InvalidPublicKey,
        /// invalid relayer
        InvalidRelayer,
        /// invalid session number
        InvalidSessionNum,
        /// invalid trustee history member
        InvalidTrusteeHisMember,
        /// invalid multi account
        InvalidMultiAccount,
        /// The reward of multi account is zero
        MultiAccountRewardZero,
        /// invalid trustee weight
        InvalidTrusteeWeight,
        /// invalid trustee start height
        InvalidTrusteeStartHeight,
        /// invalid trustee end height
        InvalidTrusteeEndHeight,
        /// not multi signature count
        NotMultiSigCount,
        /// The last trustee transition was not completed.
        LastTransitionNotCompleted,
        /// The trustee members was not enough.
        TrusteeMembersNotEnough,
        /// Exist in current trustee
        ExistCurrentTrustee,
    }

    #[pallet::storage]
    #[pallet::getter(fn trustee_multisig_addr)]
    pub type TrusteeMultiSigAddr<T: Config> =
        StorageMap<_, Twox64Concat, Chain, T::AccountId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn trustee_admin)]
    pub type TrusteeAdmin<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn relayer)]
    pub type Relayer<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn agg_pubkey_info)]
    pub type AggPubkeyInfo<T: Config> =
        StorageMap<_, Twox64Concat, Vec<u8>, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn trustee_sig_record)]
    pub type TrusteeSigRecord<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, u64, ValueQuery>;

    /// Trustee info config of the corresponding chain.
    #[pallet::storage]
    #[pallet::getter(fn trustee_info_config_of)]
    pub type TrusteeInfoConfigOf<T: Config> =
        StorageMap<_, Twox64Concat, Chain, TrusteeInfoConfig, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultForTrusteeSessionInfoLen() -> u32 {
        0
    }

    /// Current Trustee session info number of the chain.
    ///
    /// Auto generate a new session number (0) when generate new trustee of a chain.
    /// If the trustee of a chain is changed, the corresponding number will increase by 1.
    ///
    /// NOTE: The number can't be modified by users.
    #[pallet::storage]
    #[pallet::getter(fn trustee_session_info_len)]
    pub type TrusteeSessionInfoLen<T: Config> =
        StorageMap<_, Twox64Concat, Chain, u32, ValueQuery, DefaultForTrusteeSessionInfoLen>;

    /// Trustee session info of the corresponding chain and number.
    #[pallet::storage]
    #[pallet::getter(fn trustee_session_info_of)]
    pub type TrusteeSessionInfoOf<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        Chain,
        Twox64Concat,
        u32,
        GenericTrusteeSessionInfo<T::AccountId, T::BlockNumber>,
    >;

    /// Trustee intention properties of the corresponding account and chain.
    #[pallet::storage]
    #[pallet::getter(fn trustee_intention_props_of)]
    pub type TrusteeIntentionPropertiesOf<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        Chain,
        GenericTrusteeIntentionProps<T::AccountId>,
    >;

    /// The account of the corresponding chain and chain address.
    #[pallet::storage]
    pub type AddressBindingOf<T: Config> =
        StorageDoubleMap<_, Twox64Concat, Chain, Blake2_128Concat, ChainAddress, T::AccountId>;

    /// The bound address of the corresponding account and chain.
    #[pallet::storage]
    pub type BoundAddressOf<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        Chain,
        Vec<ChainAddress>,
        ValueQuery,
    >;

    /// The referral account of the corresponding account and chain.
    #[pallet::storage]
    #[pallet::getter(fn referral_binding_of)]
    pub type ReferralBindingOf<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, Chain, T::AccountId>;

    /// How long each trustee is kept. This defines the next block number at which an
    /// trustee transition will happen. If set to zero, no trustee transition are ever triggered.
    #[pallet::storage]
    #[pallet::getter(fn trustee_transition_duration)]
    pub type TrusteeTransitionDuration<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// The status of the of the trustee transition
    #[pallet::storage]
    #[pallet::getter(fn trustee_transition_status)]
    pub type TrusteeTransitionStatus<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Members not participating in trustee elections.
    ///
    /// The current trustee members did not conduct multiple signings and put the members in the
    /// little black room. Filter out the member in the next trustee election
    #[pallet::storage]
    #[pallet::getter(fn little_black_house)]
    pub type LittleBlackHouse<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub trustees: Vec<(
            Chain,
            TrusteeInfoConfig,
            Vec<(T::AccountId, Text, Vec<u8>, Vec<u8>)>,
        )>,
        pub genesis_trustee_transition_duration: T::BlockNumber,
        pub genesis_trustee_transition_status: bool,
        pub relayer: T::AccountId,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                trustees: Default::default(),
                genesis_trustee_transition_duration: Default::default(),
                genesis_trustee_transition_status: Default::default(),
                relayer: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let extra_genesis_builder: fn(&Self) = |config| {
                for (chain, info_config, trustee_infos) in config.trustees.iter() {
                    let mut trustees = Vec::with_capacity(trustee_infos.len());
                    for (who, about, hot, cold) in trustee_infos.iter() {
                        Pallet::<T>::setup_trustee_impl(
                            who.clone(),
                            None,
                            *chain,
                            about.clone(),
                            hot.clone(),
                            cold.clone(),
                        )
                        .expect("setup trustee can not fail; qed");
                        trustees.push(who.clone());
                    }
                    TrusteeInfoConfigOf::<T>::insert(chain, info_config.clone());
                }
                TrusteeTransitionDuration::<T>::put(config.genesis_trustee_transition_duration);
                TrusteeTransitionStatus::<T>::put(&config.genesis_trustee_transition_status);
                Relayer::<T>::put(&config.relayer)
            };
            extra_genesis_builder(self);
        }
    }
}

// withdraw
impl<T: Config> Pallet<T> {
    pub fn withdrawal_limit(
        asset_id: &T::AssetId,
    ) -> Result<WithdrawalLimit<T::Balance>, DispatchError> {
        let chain = xpallet_gateway_records::Pallet::<T>::chain_of(asset_id)?;
        match chain {
            Chain::Bitcoin => T::Bitcoin::withdrawal_limit(asset_id),
            _ => Err(Error::<T>::NotSupportedChain.into()),
        }
    }

    pub fn withdrawal_list_with_fee_info(
        asset_id: &T::AssetId,
    ) -> Result<
        BTreeMap<
            WithdrawalRecordId,
            (
                Withdrawal<T::AccountId, T::AssetId, T::Balance, T::BlockNumber>,
                WithdrawalLimit<T::Balance>,
            ),
        >,
        DispatchError,
    > {
        let limit = Self::withdrawal_limit(asset_id)?;
        let result: BTreeMap<
            WithdrawalRecordId,
            (
                Withdrawal<T::AccountId, T::AssetId, T::Balance, T::BlockNumber>,
                WithdrawalLimit<T::Balance>,
            ),
        > = xpallet_gateway_records::PendingWithdrawals::<T>::iter()
            .map(|(id, record)| {
                (
                    id,
                    (
                        Withdrawal::new(
                            record,
                            xpallet_gateway_records::Pallet::<T>::state_of(id).unwrap_or_default(),
                        ),
                        limit.clone(),
                    ),
                )
            })
            .collect();
        Ok(result)
    }

    // Make sure the hot and cold pubkey are set and do not check the validity of the address
    pub fn ensure_set_address(who: &T::AccountId, chain: Chain) -> bool {
        Self::trustee_intention_props_of(who, chain).is_some()
    }

    pub fn verify_withdrawal(
        asset_id: T::AssetId,
        value: T::Balance,
        addr: &[u8],
        ext: &Memo,
    ) -> DispatchResult {
        ext.check_validity()?;

        let chain = xpallet_gateway_records::Pallet::<T>::chain_of(&asset_id)?;
        match chain {
            Chain::Bitcoin => {
                // bitcoin do not need memo
                T::Bitcoin::check_addr(addr, b"")?;
            }
            _ => return Err(Error::<T>::NotSupportedChain.into()),
        };
        // we could only split withdrawal limit due to a runtime-api would call `withdrawal_limit`
        // to export `WithdrawalLimit` for an asset.
        let limit = Self::withdrawal_limit(&asset_id)?;
        // withdrawal value should larger than minimal_withdrawal, allow equal
        if value < limit.minimal_withdrawal {
            return Err(Error::<T>::InvalidWithdrawal.into());
        }
        Ok(())
    }

    pub fn generate_trustee_pool() -> Vec<T::AccountId> {
        let members = {
            let mut members = pallet_elections_phragmen::Pallet::<T>::members();
            members.sort_unstable_by(|a, b| b.stake.cmp(&a.stake));
            members
                .iter()
                .map(|m| m.who.clone())
                .collect::<Vec<T::AccountId>>()
        };
        let runners_up = {
            let mut runners_up = pallet_elections_phragmen::Pallet::<T>::runners_up();
            runners_up.sort_unstable_by(|a, b| b.stake.cmp(&a.stake));
            runners_up
                .iter()
                .map(|m| m.who.clone())
                .collect::<Vec<T::AccountId>>()
        };
        [members, runners_up].concat()
    }

    pub fn do_trustee_election() -> DispatchResult {
        if Self::trustee_transition_status() {
            return Err(Error::<T>::LastTransitionNotCompleted.into());
        }

        // Current trustee list
        let old_trustee_candidate: Vec<T::AccountId> =
            if let Ok(info) = T::BitcoinTrusteeSessionProvider::current_trustee_session() {
                info.trustee_list.into_iter().unzip::<_, _, _, Vec<u64>>().0
            } else {
                vec![]
            };

        let filter_members: Vec<T::AccountId> = Self::little_black_house();

        let all_trustee_pool = Self::generate_trustee_pool();

        let new_trustee_pool: Vec<T::AccountId> = all_trustee_pool
            .iter()
            .filter_map(|who| {
                match filter_members.contains(who) || !Self::ensure_set_address(who, Chain::Bitcoin)
                {
                    true => None,
                    false => Some(who.clone()),
                }
            })
            .collect::<Vec<T::AccountId>>();

        let remain_filter_members = filter_members
            .iter()
            .filter_map(|who| match all_trustee_pool.contains(who) {
                true => Some(who.clone()),
                false => None,
            })
            .collect::<Vec<_>>();

        LittleBlackHouse::<T>::put(remain_filter_members);

        let desired_members =
            (<T as pallet_elections_phragmen::Config>::DesiredMembers::get() - 1) as usize;

        if new_trustee_pool.len() < desired_members {
            return Err(Error::<T>::TrusteeMembersNotEnough.into());
        }

        let new_trustee_candidate = new_trustee_pool[..desired_members].to_vec();
        let mut new_trustee_candidate_sorted = new_trustee_candidate.clone();
        new_trustee_candidate_sorted.sort_unstable();

        let mut old_trustee_candidate_sorted = old_trustee_candidate;
        old_trustee_candidate_sorted.sort_unstable();
        let (incoming, outgoing) =
            <T as pallet_elections_phragmen::Config>::ChangeMembers::compute_members_diff_sorted(
                &old_trustee_candidate_sorted,
                &new_trustee_candidate_sorted,
            );
        if incoming.is_empty() && outgoing.is_empty() {
            return Err(Error::<T>::TrusteeMembersNotEnough.into());
        }
        Self::transition_trustee_session_impl(Chain::Bitcoin, new_trustee_candidate)?;
        if Self::trustee_session_info_len(Chain::Bitcoin) != 1 {
            TrusteeTransitionStatus::<T>::put(true);
        }
        Ok(())
    }
}

pub fn is_valid_about<T: Config>(about: &[u8]) -> DispatchResult {
    // TODO
    if about.len() > 128 {
        return Err(Error::<T>::InvalidAboutLen.into());
    }

    xp_runtime::xss_check(about)
}

// trustees
impl<T: Config> Pallet<T> {
    pub fn ensure_not_current_trustee(who: &T::AccountId) -> bool {
        if let Ok(info) = T::BitcoinTrusteeSessionProvider::current_trustee_session() {
            !info.trustee_list.into_iter().any(|n| &n.0 == who)
        } else {
            true
        }
    }

    pub fn setup_trustee_impl(
        who: T::AccountId,
        proxy_account: Option<T::AccountId>,
        chain: Chain,
        about: Text,
        hot_entity: Vec<u8>,
        cold_entity: Vec<u8>,
    ) -> DispatchResult {
        is_valid_about::<T>(&about)?;

        let (hot, cold) = match chain {
            Chain::Bitcoin => {
                let hot = T::BitcoinTrustee::check_trustee_entity(&hot_entity)?;
                let cold = T::BitcoinTrustee::check_trustee_entity(&cold_entity)?;
                (hot.into(), cold.into())
            }
            _ => return Err(Error::<T>::NotSupportedChain.into()),
        };

        let proxy_account = if let Some(addr) = proxy_account {
            Some(addr)
        } else {
            Some(who.clone())
        };

        let props = GenericTrusteeIntentionProps::<T::AccountId>(TrusteeIntentionProps::<
            T::AccountId,
            Vec<u8>,
        > {
            proxy_account,
            about,
            hot_entity: hot,
            cold_entity: cold,
        });

        if TrusteeIntentionPropertiesOf::<T>::contains_key(&who, chain) {
            if Self::little_black_house().contains(&who) {
                LittleBlackHouse::<T>::mutate(|house| house.retain(|a| *a != who));
            }
            TrusteeIntentionPropertiesOf::<T>::mutate(&who, chain, |t| *t = Some(props.clone()));
        } else {
            TrusteeIntentionPropertiesOf::<T>::insert(&who, chain, props.clone());
        }
        Self::deposit_event(Event::<T>::SetTrusteeProps(who, chain, props));
        Ok(())
    }

    pub fn try_generate_session_info(
        chain: Chain,
        new_trustees: Vec<T::AccountId>,
    ) -> Result<
        (
            GenericTrusteeSessionInfo<T::AccountId, T::BlockNumber>,
            ScriptInfo<T::AccountId>,
        ),
        DispatchError,
    > {
        let config = Self::trustee_info_config_of(chain);
        let has_duplicate =
            (1..new_trustees.len()).any(|i| new_trustees[i..].contains(&new_trustees[i - 1]));
        if has_duplicate {
            error!(
                target: "runtime::gateway::common",
                "[try_generate_session_info] Duplicate account, candidates:{:?}",
                new_trustees
            );
            return Err(Error::<T>::DuplicatedAccountId.into());
        }
        let mut props = Vec::with_capacity(new_trustees.len());
        for accountid in new_trustees.into_iter() {
            let p = Self::trustee_intention_props_of(&accountid, chain).ok_or_else(|| {
                error!(
                    target: "runtime::gateway::common",
                    "[transition_trustee_session] Candidate {:?} has not registered as a trustee",
                    accountid
                );
                Error::<T>::NotRegistered
            })?;
            props.push((accountid, p));
        }
        let info = match chain {
            Chain::Bitcoin => {
                let props = props
                    .into_iter()
                    .map(|(id, prop)| {
                        (
                            id,
                            TrusteeIntentionProps::<T::AccountId, _>::try_from(prop)
                                .expect("must decode succss from storage data"),
                        )
                    })
                    .collect();
                let session_info = T::BitcoinTrustee::generate_trustee_session_info(props, config)?;

                (session_info.0.into(), session_info.1)
            }
            _ => return Err(Error::<T>::NotSupportedChain.into()),
        };
        Ok(info)
    }

    fn transition_trustee_session_impl(
        chain: Chain,
        new_trustees: Vec<T::AccountId>,
    ) -> DispatchResult {
        let mut info = Self::try_generate_session_info(chain, new_trustees)?;
        let multi_addr = Self::generate_multisig_addr(chain, &info.0)?;

        let session_number = Self::trustee_session_info_len(chain)
            .checked_add(1)
            .unwrap_or(0u32);

        TrusteeSessionInfoLen::<T>::insert(chain, session_number);
        info.0 .0.multi_account = Some(multi_addr.clone());
        TrusteeSessionInfoOf::<T>::insert(chain, session_number, info.0.clone());
        TrusteeMultiSigAddr::<T>::insert(chain, multi_addr);
        // Remove the information of the previous aggregate public key，Withdrawal is prohibited at this time.
        AggPubkeyInfo::<T>::remove_all(None);
        for index in 0..info.1.agg_pubkeys.len() {
            AggPubkeyInfo::<T>::insert(
                &info.1.agg_pubkeys[index],
                info.1.personal_accounts[index].clone(),
            );
        }
        TrusteeAdmin::<T>::kill();

        Self::deposit_event(Event::<T>::TrusteeSetChanged(
            chain,
            session_number,
            info.0,
            info.1.agg_pubkeys.len() as u32,
        ));
        Ok(())
    }

    pub fn generate_multisig_addr(
        chain: Chain,
        session_info: &GenericTrusteeSessionInfo<T::AccountId, T::BlockNumber>,
    ) -> Result<T::AccountId, DispatchError> {
        // If there is a proxy account, choose a proxy account
        let mut acc_list: Vec<T::AccountId> = vec![];
        for acc in session_info.0.trustee_list.iter() {
            let acc = Self::trustee_intention_props_of(&acc.0, chain)
                .ok_or_else::<DispatchError, _>(|| {
                    error!(
                        target: "runtime::gateway::common",
                        "[generate_multisig_addr] acc {:?} has not in TrusteeIntentionPropertiesOf",
                        acc.0
                    );
                    Error::<T>::NotRegistered.into()
                })?
                .0
                .proxy_account
                .unwrap_or_else(|| acc.0.clone());
            acc_list.push(acc);
        }

        let multi_addr =
            T::DetermineMultisigAddress::calc_multisig(&acc_list, session_info.0.threshold);

        // Each chain must have a distinct multisig address,
        // duplicated multisig address is not allowed.
        let find_duplicated = Self::trustee_multisigs()
            .into_iter()
            .any(|(c, multisig)| multi_addr == multisig && c == chain);
        if find_duplicated {
            return Err(Error::<T>::InvalidMultisig.into());
        }
        Ok(multi_addr)
    }

    fn set_referral_binding(chain: Chain, who: T::AccountId, referral: T::AccountId) {
        ReferralBindingOf::<T>::insert(&who, &chain, referral.clone());
        Self::deposit_event(Event::<T>::ReferralBinded(who, chain, referral))
    }

    pub fn apply_tranfer_trustee_reward(
        who: &T::AccountId,
        session_num: u32,
        trustee_info: &TrusteeSessionInfo<T::AccountId, T::BlockNumber, BtcTrusteeAddrInfo>,
        amount: Balanceof<T>,
    ) -> DispatchResult {
        let multi_account = trustee_info
            .multi_account
            .clone()
            .ok_or(Error::<T>::InvalidMultiAccount)?;

        let start_height = trustee_info
            .start_height
            .ok_or(Error::<T>::InvalidTrusteeStartHeight)?;

        if frame_system::Pallet::<T>::block_number().saturating_sub(start_height)
            < Self::trustee_transition_duration()
        {
            let _end_height = trustee_info
                .end_height
                .ok_or(Error::<T>::InvalidTrusteeEndHeight)?;
        }
        ensure!(
            !trustee_info
                .trustee_list
                .iter()
                .map(|n| n.1)
                .sum::<u64>()
                .is_zero(),
            Error::<T>::NotMultiSigCount
        );

        <T as xpallet_gateway_records::Config>::Currency::transfer(
            who,
            &multi_account,
            amount,
            ExistenceRequirement::AllowDeath,
        )?;

        Self::deposit_event(Event::<T>::TransferTrusteeReward(
            who.clone(),
            multi_account,
            Chain::Bitcoin,
            session_num,
            amount,
        ));

        Ok(())
    }

    pub fn apply_claim_trustee_reward(
        who: &T::AccountId,
        session_num: u32,
        trustee_info: &TrusteeSessionInfo<T::AccountId, T::BlockNumber, BtcTrusteeAddrInfo>,
    ) -> DispatchResult {
        ensure!(
            trustee_info.trustee_list.iter().any(|n| &n.0 == who),
            Error::<T>::InvalidTrusteeHisMember
        );

        let multi_account = match trustee_info.multi_account.clone() {
            None => return Err(Error::<T>::InvalidMultiAccount.into()),
            Some(n) => n,
        };

        ensure!(
            !<T as xpallet_gateway_records::Config>::Currency::free_balance(&multi_account)
                .is_zero(),
            Error::<T>::MultiAccountRewardZero
        );

        let mut reward_info = RewardInfo { rewards: vec![] };
        let trustee_len = trustee_info.trustee_list.len();
        let sum_balance =
            <T as xpallet_gateway_records::Config>::Currency::free_balance(&multi_account);
        let sum_weight: Balanceof<T> = trustee_info
            .trustee_list
            .iter()
            .map(|n| n.1)
            .sum::<u64>()
            .saturated_into();
        let mut acc_balance = Balanceof::<T>::zero();
        for i in 0..trustee_len - 1 {
            let trustee_weight: Balanceof<T> = trustee_info.trustee_list[i].1.saturated_into();
            let amount = sum_balance
                .saturating_mul(trustee_weight)
                .checked_div(&sum_weight)
                .ok_or(Error::<T>::InvalidTrusteeWeight)?;
            reward_info
                .rewards
                .push((trustee_info.trustee_list[i].0.clone(), amount));
            acc_balance = acc_balance.saturating_add(amount);
        }
        let amount = sum_balance.saturating_sub(acc_balance);
        reward_info
            .rewards
            .push((trustee_info.trustee_list[trustee_len - 1].0.clone(), amount));
        for (acc, amount) in reward_info.rewards.iter() {
            <T as xpallet_gateway_records::Config>::Currency::transfer(
                &multi_account,
                acc,
                *amount,
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|e| {
                error!(
                    target: "runtime::gateway::common",
                    "[apply_claim_trustee_reward] error {:?}, sum_balance:{:?}, reward_info:{:?}.",
                    e, sum_balance, reward_info.clone()
                );
                e
            })?;
        }
        Self::deposit_event(Event::<T>::TrusteeRewardComplete(
            who.clone(),
            Chain::Bitcoin,
            session_num,
            reward_info,
        ));
        Ok(())
    }
}

impl<T: Config> Pallet<T> {
    pub fn trustee_multisigs() -> BTreeMap<Chain, T::AccountId> {
        TrusteeMultiSigAddr::<T>::iter().collect()
    }
}
