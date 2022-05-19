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
pub mod migrations;
pub mod traits;
pub mod trustees;
pub mod types;
pub mod utils;
pub mod weights;

use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    log::{error, info},
    traits::{fungibles, ChangeMembers, Currency, ExistenceRequirement, Get},
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};

use sp_runtime::{
    traits::{CheckedDiv, Saturating, UniqueSaturatedInto, Zero},
    SaturatedConversion,
};
use sp_std::{collections::btree_map::BTreeMap, convert::TryFrom, prelude::*};

use sherpax_primitives::{AddrStr, ChainAddress, Text};
use traits::BytesLike;
use xp_assets_registrar::Chain;
use xp_runtime::Memo;

use xpallet_gateway_records::{ChainT, Withdrawal, WithdrawalLimit, WithdrawalRecordId};
use xpallet_support::traits::{MultisigAddressFor, Validator};

use self::{
    traits::{ProposalProvider, TotalSupply, TrusteeForChain, TrusteeInfoUpdate, TrusteeSession},
    types::{
        GenericTrusteeIntentionProps, GenericTrusteeSessionInfo, RewardInfo, ScriptInfo,
        TrusteeInfoConfig, TrusteeIntentionProps, TrusteeSessionInfo,
    },
};

pub use self::weights::WeightInfo;

pub use pallet::*;

type Balanceof<T> = <<T as xpallet_gateway_records::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::fungibles::Inspect, transactional};

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + xpallet_gateway_records::Config
        + pallet_elections_phragmen::Config
        + pallet_assets::Config
    {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Calculate the multi-signature address.
        type DetermineMultisigAddress: MultisigAddressFor<Self::AccountId>;
        /// Check the validator's account.
        type Validator: Validator<Self::AccountId>;
        /// A majority of the council can excute some transactions.
        type CouncilOrigin: EnsureOrigin<Self::Origin>;

        /// Bitcoin
        /// Get btc chain info.
        type Bitcoin: ChainT<Self::AssetId, Self::Balance>;
        /// Generate btc trustee session info.
        type BitcoinTrustee: TrusteeForChain<
            Self::AccountId,
            Self::BlockNumber,
            trustees::bitcoin::BtcTrusteeType,
            trustees::bitcoin::BtcTrusteeAddrInfo,
        >;
        /// Get trustee session info.
        type BitcoinTrusteeSessionProvider: TrusteeSession<
            Self::AccountId,
            Self::BlockNumber,
            trustees::bitcoin::BtcTrusteeAddrInfo,
        >;
        /// When the trust changes, the total supply of btc: total issue + pending deposit. Help
        /// to the allocation of btc withdrawal fees.
        type BitcoinTotalSupply: TotalSupply<Self::Balance>;
        /// Get btc withdrawal proposal.
        type BitcoinWithdrawalProposal: ProposalProvider;

        /// Dogecoin
        /// Get btc chain info.
        type Dogecoin: ChainT<Self::AssetId, Self::Balance>;
        /// Generate dogecoin trustee session info.
        type DogecoinTrustee: TrusteeForChain<
            Self::AccountId,
            Self::BlockNumber,
            trustees::dogecoin::DogeTrusteeType,
            trustees::dogecoin::DogeTrusteeAddrInfo,
        >;
        /// Get trustee session info.
        type DogecoinTrusteeSessionProvider: TrusteeSession<
            Self::AccountId,
            Self::BlockNumber,
            trustees::dogecoin::DogeTrusteeAddrInfo,
        >;
        /// When the trust changes, the total supply of btc: total issue + pending deposit. Help
        /// to the allocation of btc withdrawal fees.
        type DogecoinTotalSupply: TotalSupply<Self::Balance>;
        /// Get btc withdrawal proposal.
        type DogecoinWithdrawalProposal: ProposalProvider;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
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
        #[transactional]
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
        #[transactional]
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
                    || Self::little_black_house(chain).contains(&who),
                Error::<T>::NotTrusteePreselectedMember
            );

            ensure!(
                Self::ensure_not_current_trustee(chain, &who)
                    && !Self::trustee_transition_status(chain),
                Error::<T>::ExistCurrentTrustee
            );

            Self::setup_trustee_impl(who, proxy_account, chain, about, hot_entity, cold_entity)
        }

        /// Manual execution of the election by admin.
        #[pallet::weight(0u64)]
        pub fn execute_trustee_election(origin: OriginFor<T>, chain: Chain) -> DispatchResult {
            T::CouncilOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(Self::try_ensure_trustee_admin)
                .map(|_| ())
                .or_else(ensure_root)?;

            Self::do_trustee_election(chain)
        }

        /// Force cancel trustee transition
        ///
        /// This is called by the root or council.
        #[pallet::weight(0u64)]
        pub fn cancel_trustee_election(origin: OriginFor<T>, chain: Chain) -> DispatchResult {
            T::CouncilOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;

            Self::cancel_trustee_transition_impl(chain)?;
            TrusteeTransitionStatus::<T>::insert(chain, false);
            Ok(())
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
        #[pallet::weight(0u64)]
        #[transactional]
        pub fn move_trust_into_black_room(
            origin: OriginFor<T>,
            chain: Chain,
            trustees: Option<Vec<T::AccountId>>,
        ) -> DispatchResult {
            T::CouncilOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(Self::try_ensure_trustee_admin)
                .map(|_| ())
                .or_else(ensure_root)?;

            info!(
                target: "runtime::gateway::common",
                "[move_trust_into_black_room] Try to move a trustee into black room, trustee:{:?}",
                trustees
            );

            if let Some(trustees) = trustees {
                LittleBlackHouse::<T>::mutate(chain, |l| {
                    for trustee in trustees.iter() {
                        l.push(trustee.clone());
                    }
                    l.sort_unstable();
                    l.dedup();
                });
                trustees.into_iter().for_each(|trustee| {
                    if TrusteeSigRecord::<T>::contains_key(chain, &trustee) {
                        TrusteeSigRecord::<T>::mutate(chain, &trustee, |record| *record = Some(0));
                    }
                });
            }

            Self::do_trustee_election(chain)?;
            Ok(())
        }

        /// Move member out small black room.
        ///
        /// This is called by the trustee admin and root.
        /// # <weight>
        /// Since this is a root call and will go into trustee election, we assume full block for now.
        /// # </weight>
        #[pallet::weight(0u64)]
        pub fn move_trust_out_black_room(
            origin: OriginFor<T>,
            chain: Chain,
            members: Vec<T::AccountId>,
        ) -> DispatchResult {
            T::CouncilOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(Self::try_ensure_trustee_admin)
                .map(|_| ())
                .or_else(ensure_root)?;

            info!(
                target: "runtime::gateway::common",
                "[move_trust_into_black_room] Try to move a member out black room, member:{:?}",
                members
            );
            members.into_iter().for_each(|member| {
                if Self::little_black_house(chain).contains(&member) {
                    LittleBlackHouse::<T>::mutate(chain, |house| house.retain(|a| *a != member));
                }
            });

            Ok(())
        }

        /// Force trustee election
        ///
        /// Mandatory trustee renewal if the current trustee is not doing anything
        ///
        /// This is called by the root.
        #[pallet::weight(100_000_000u64)]
        pub fn force_trustee_election(origin: OriginFor<T>, chain: Chain) -> DispatchResult {
            ensure_root(origin)?;
            Self::update_transition_status(chain, false, None);

            Ok(())
        }

        /// Force update trustee info
        ///
        /// This is called by the root.
        #[pallet::weight(100_000_000u64)]
        pub fn force_update_trustee(
            origin: OriginFor<T>,
            who: T::AccountId,
            proxy_account: Option<T::AccountId>,
            chain: Chain,
            about: Text,
            hot_entity: Vec<u8>,
            cold_entity: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Self::setup_trustee_impl(who, proxy_account, chain, about, hot_entity, cold_entity)?;
            Ok(())
        }

        /// Set trustee's proxy account
        #[pallet::weight(< T as Config >::WeightInfo::set_trustee_proxy())]
        pub fn set_trustee_proxy(
            origin: OriginFor<T>,
            proxy_account: T::AccountId,
            chain: Chain,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                TrusteeIntentionPropertiesOf::<T>::contains_key(&who, chain),
                Error::<T>::NotRegistered
            );

            Self::set_trustee_proxy_impl(&who, proxy_account, chain);
            Ok(())
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
            T::CouncilOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;

            TrusteeInfoConfigOf::<T>::insert(chain, config);
            Ok(())
        }

        /// Set trustee admin multiply
        #[pallet::weight(< T as Config >::WeightInfo::set_trustee_admin_multiply())]
        pub fn set_trustee_admin_multiply(origin: OriginFor<T>, multiply: u64) -> DispatchResult {
            T::CouncilOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;

            TrusteeAdminMultiply::<T>::put(multiply);
            Ok(())
        }

        /// Set the trustee admin.
        ///
        /// This is a root-only operation.
        /// The trustee admin is the account who can change the trustee list.
        #[pallet::weight(< T as Config >::WeightInfo::set_trustee_admin())]
        pub fn set_trustee_admin(origin: OriginFor<T>, admin: T::AccountId) -> DispatchResult {
            T::CouncilOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;

            TrusteeAdmin::<T>::put(admin);
            Ok(())
        }

        /// Assign trustee reward
        ///
        /// To allocate trust rewards through conuncil. If
        /// trustees have not changed for a long time, the
        /// current trustees' signature record will be
        /// cleared after allocating the current trustees'
        /// reward. The rewards that allocate the previous
        /// session will not clear the trustees' signature
        /// record.
        ///
        /// All trust rewards will be allocated when calling.
        /// I don't think it should be called by a trust. It
        /// is best to call it through council.
        #[pallet::weight(< T as Config >::WeightInfo::claim_trustee_reward())]
        #[transactional]
        pub fn claim_trustee_reward(
            origin: OriginFor<T>,
            chain: Chain,
            session_num: i32,
        ) -> DispatchResult {
            T::CouncilOrigin::ensure_origin(origin)?;

            let session_num: u32 = if session_num < 0 {
                match session_num {
                    -1i32 => Self::trustee_session_info_len(chain),
                    -2i32 => Self::trustee_session_info_len(chain)
                        .checked_sub(1)
                        .ok_or(Error::<T>::InvalidSessionNum)?,
                    _ => return Err(Error::<T>::InvalidSessionNum.into()),
                }
            } else {
                session_num as u32
            };

            if session_num == Self::trustee_session_info_len(chain) {
                // update trustee sig record info (update reward weight)
                TrusteeSessionInfoOf::<T>::mutate(chain, session_num, |info| {
                    if let Some(info) = info {
                        info.0.trustee_list.iter_mut().for_each(|trustee| {
                            trustee.1 = Self::trustee_sig_record(chain, &trustee.0).unwrap_or(0u64);
                        });
                    }
                });
            }

            match chain {
                Chain::Bitcoin => {
                    let session_info =
                        T::BitcoinTrusteeSessionProvider::trustee_session(session_num)?;

                    Self::apply_claim_trustee_reward(chain, session_num, session_info)?;
                }
                Chain::Dogecoin => {
                    let session_info =
                        T::DogecoinTrusteeSessionProvider::trustee_session(session_num)?;

                    Self::apply_claim_trustee_reward(chain, session_num, session_info)?;
                }
                _ => return Err(Error::<T>::NotSupportedChain.into()),
            }

            Ok(())
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
        /// A trustee set his proxy account. [who, chain, proxy_account]
        SetTrusteeProxy(T::AccountId, Chain, T::AccountId),
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
        /// Asset reward to trustee multi_account. [target, asset_id, reward_total]
        TransferAssetReward(T::AccountId, T::AssetId, T::Balance),
        /// The native asset of trustee multi_account is assigned. [who, multi_account, session_number, total_reward]
        AllocNativeReward(T::AccountId, u32, Balanceof<T>),
        /// The not native asset of trustee multi_account is assigned. [who, multi_account, session_number, asset_id, total_reward]
        AllocNotNativeReward(T::AccountId, u32, T::AssetId, T::Balance),
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
        /// invalid script signature
        InvalidScriptSig,
        /// invalid redeem script
        InvalidRedeemScript,
        /// unsupported chain
        NotSupportedChain,
        /// duplicated multisig
        DuplicatedMultiAddress,
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
        /// prevent transition when the withdrawal proposal exists.
        WithdrawalProposalExist,
        /// The trustee members was not enough.
        TrusteeMembersNotEnough,
        /// Exist in current trustee
        ExistCurrentTrustee,
    }

    /// The trustee multi substrate account.
    #[pallet::storage]
    #[pallet::getter(fn trustee_multisig_addr)]
    pub(crate) type TrusteeMultiSigAddr<T: Config> =
        StorageMap<_, Twox64Concat, Chain, T::AccountId, OptionQuery>;

    /// The trustee administrator.
    #[pallet::storage]
    #[pallet::getter(fn trustee_admin)]
    pub(crate) type TrusteeAdmin<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// The trustee multiply for signature record.
    #[pallet::storage]
    #[pallet::getter(fn trustee_admin_multiply)]
    pub(crate) type TrusteeAdminMultiply<T: Config> =
        StorageValue<_, u64, ValueQuery, DefaultForTrusteeAdminMultiply>;

    #[pallet::type_value]
    pub fn DefaultForTrusteeAdminMultiply() -> u64 {
        11
    }

    /// Storage trustee aggregate pubkey to related accounts for bitcoin taproot.
    #[pallet::storage]
    #[pallet::getter(fn agg_pubkey_info)]
    pub(crate) type AggPubkeyInfo<T: Config> =
        StorageMap<_, Twox64Concat, Vec<u8>, Vec<T::AccountId>, ValueQuery>;

    /// Storage trustee pubkey to account for dogecoin p2sh.
    #[pallet::storage]
    #[pallet::getter(fn hot_pubkey_info)]
    pub(crate) type HotPubkeyInfo<T: Config> =
        StorageMap<_, Twox64Concat, Vec<u8>, T::AccountId, OptionQuery>;

    /// Record the amount of the trust signature, which is easy to allocate rewards.
    #[pallet::storage]
    #[pallet::getter(fn trustee_sig_record)]
    pub(crate) type TrusteeSigRecord<T: Config> =
        StorageDoubleMap<_, Twox64Concat, Chain, Twox64Concat, T::AccountId, u64>;

    /// Trustee info config of the corresponding chain.
    #[pallet::storage]
    #[pallet::getter(fn trustee_info_config_of)]
    pub(crate) type TrusteeInfoConfigOf<T: Config> =
        StorageMap<_, Twox64Concat, Chain, TrusteeInfoConfig, ValueQuery>;

    /// Current Trustee session info number of the chain.
    ///
    /// Auto generate a new session number (0) when generate new trustee of a chain.
    /// If the trustee of a chain is changed, the corresponding number will increase by 1.
    ///
    /// NOTE: The number can't be modified by users.
    #[pallet::storage]
    #[pallet::getter(fn trustee_session_info_len)]
    pub(crate) type TrusteeSessionInfoLen<T: Config> =
        StorageMap<_, Twox64Concat, Chain, u32, ValueQuery, DefaultForTrusteeSessionInfoLen>;

    #[pallet::type_value]
    pub fn DefaultForTrusteeSessionInfoLen() -> u32 {
        0
    }

    /// Trustee session info of the corresponding chain and number.
    #[pallet::storage]
    #[pallet::getter(fn trustee_session_info_of)]
    pub(crate) type TrusteeSessionInfoOf<T: Config> = StorageDoubleMap<
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
    pub(crate) type TrusteeIntentionPropertiesOf<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        Chain,
        GenericTrusteeIntentionProps<T::AccountId>,
    >;

    /// The account of the corresponding chain and chain address.
    #[pallet::storage]
    pub(crate) type AddressBindingOf<T: Config> =
        StorageDoubleMap<_, Twox64Concat, Chain, Blake2_128Concat, ChainAddress, T::AccountId>;

    /// The bound address of the corresponding account and chain.
    #[pallet::storage]
    pub(crate) type BoundAddressOf<T: Config> = StorageDoubleMap<
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
    pub(crate) type ReferralBindingOf<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, Chain, T::AccountId>;

    /// The status of the of the trustee transition
    #[pallet::storage]
    #[pallet::getter(fn trustee_transition_status)]
    pub(crate) type TrusteeTransitionStatus<T: Config> =
        StorageMap<_, Twox64Concat, Chain, bool, ValueQuery>;

    /// Members not participating in trustee elections.
    ///
    /// The current trustee members did not conduct multiple signings and put the members in the
    /// little black room. Filter out the member in the next trustee election
    #[pallet::storage]
    #[pallet::getter(fn little_black_house)]
    pub(crate) type LittleBlackHouse<T: Config> =
        StorageMap<_, Twox64Concat, Chain, Vec<T::AccountId>, ValueQuery>;

    /// When the trust exchange begins, the total cross-chain assets of a certain AssetId
    #[pallet::storage]
    #[pallet::getter(fn pre_total_supply)]
    pub(crate) type PreTotalSupply<T: Config> =
        StorageMap<_, Twox64Concat, T::AssetId, T::Balance, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub trustees: Vec<(
            Chain,
            TrusteeInfoConfig,
            Vec<(T::AccountId, Text, Vec<u8>, Vec<u8>)>,
        )>,
        pub genesis_trustee_transition_status: bool,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                trustees: Default::default(),
                genesis_trustee_transition_status: Default::default(),
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
                TrusteeTransitionStatus::<T>::insert(
                    Chain::Bitcoin,
                    &config.genesis_trustee_transition_status,
                );
                TrusteeTransitionStatus::<T>::insert(
                    Chain::Dogecoin,
                    &config.genesis_trustee_transition_status,
                );
            };
            extra_genesis_builder(self);
        }
    }
}

/// Withdrawal
impl<T: Config> Pallet<T> {
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
            Chain::Dogecoin => {
                // dogecoin do not need memo
                T::Dogecoin::check_addr(addr, b"")?;
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
}

/// Trustee Setup
impl<T: Config> Pallet<T> {
    pub fn setup_trustee_impl(
        who: T::AccountId,
        proxy_account: Option<T::AccountId>,
        chain: Chain,
        about: Text,
        hot_entity: Vec<u8>,
        cold_entity: Vec<u8>,
    ) -> DispatchResult {
        Self::is_valid_about(&about)?;

        let (hot, cold) = match chain {
            Chain::Bitcoin => {
                let hot = T::BitcoinTrustee::check_trustee_entity(&hot_entity)?;
                let cold = T::BitcoinTrustee::check_trustee_entity(&cold_entity)?;
                (hot.into(), cold.into())
            }
            Chain::Dogecoin => {
                let hot = T::DogecoinTrustee::check_trustee_entity(&hot_entity)?;
                let cold = T::DogecoinTrustee::check_trustee_entity(&cold_entity)?;
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
            TrusteeIntentionPropertiesOf::<T>::mutate(&who, chain, |t| *t = Some(props.clone()));
        } else {
            TrusteeIntentionPropertiesOf::<T>::insert(&who, chain, props.clone());
        }
        Self::deposit_event(Event::<T>::SetTrusteeProps(who, chain, props));
        Ok(())
    }

    pub fn set_trustee_proxy_impl(who: &T::AccountId, proxy_account: T::AccountId, chain: Chain) {
        TrusteeIntentionPropertiesOf::<T>::mutate(who, chain, |t| {
            if let Some(props) = t {
                props.0.proxy_account = Some(proxy_account.clone());
            }
        });
        Self::deposit_event(Event::<T>::SetTrusteeProxy(
            who.clone(),
            chain,
            proxy_account,
        ));
    }

    pub fn ensure_not_current_trustee(chain: Chain, who: &T::AccountId) -> bool {
        match chain {
            Chain::Bitcoin => {
                if let Ok(info) = T::BitcoinTrusteeSessionProvider::current_trustee_session() {
                    !info.trustee_list.into_iter().any(|n| &n.0 == who)
                } else {
                    true
                }
            }
            Chain::Dogecoin => {
                if let Ok(info) = T::DogecoinTrusteeSessionProvider::current_trustee_session() {
                    !info.trustee_list.into_iter().any(|n| &n.0 == who)
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    fn set_referral_binding(chain: Chain, who: T::AccountId, referral: T::AccountId) {
        ReferralBindingOf::<T>::insert(&who, &chain, referral.clone());
        Self::deposit_event(Event::<T>::ReferralBinded(who, chain, referral))
    }

    pub fn is_valid_about(about: &[u8]) -> DispatchResult {
        ensure!(about.len() <= 128, Error::<T>::InvalidAboutLen);

        xp_runtime::xss_check(about)
    }
}

/// Trustee Transition
impl<T: Config> Pallet<T> {
    pub fn do_trustee_election(chain: Chain) -> DispatchResult {
        ensure!(
            !Self::trustee_transition_status(chain),
            Error::<T>::LastTransitionNotCompleted
        );

        match chain {
            Chain::Bitcoin => ensure!(
                T::BitcoinWithdrawalProposal::get_withdrawal_proposal().is_none(),
                Error::<T>::WithdrawalProposalExist,
            ),
            Chain::Dogecoin => ensure!(
                T::DogecoinWithdrawalProposal::get_withdrawal_proposal().is_none(),
                Error::<T>::WithdrawalProposalExist,
            ),
            _ => return Err(Error::<T>::NotSupportedChain.into()),
        }

        // Current trustee list
        let old_trustee_candidate: Vec<T::AccountId> = match chain {
            Chain::Bitcoin => match T::BitcoinTrusteeSessionProvider::current_trustee_session() {
                Ok(info) => info.trustee_list.into_iter().unzip::<_, _, _, Vec<u64>>().0,
                Err(_) => vec![],
            },
            Chain::Dogecoin => match T::DogecoinTrusteeSessionProvider::current_trustee_session() {
                Ok(info) => info.trustee_list.into_iter().unzip::<_, _, _, Vec<u64>>().0,
                Err(_) => vec![],
            },
            _ => return Err(Error::<T>::NotSupportedChain.into()),
        };

        let filter_members: Vec<T::AccountId> = Self::little_black_house(chain);

        let all_trustee_pool = Self::generate_trustee_pool();

        let new_trustee_pool: Vec<T::AccountId> = all_trustee_pool
            .iter()
            .filter_map(|who| {
                match filter_members.contains(who) || !Self::ensure_set_address(who, chain) {
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
        Self::transition_trustee_session_impl(chain, new_trustee_candidate)?;
        LittleBlackHouse::<T>::insert(chain, remain_filter_members);
        if Self::trustee_session_info_len(chain) != 1 {
            TrusteeTransitionStatus::<T>::insert(chain, true);
            match chain {
                Chain::Bitcoin => {
                    let total_supply = T::BitcoinTotalSupply::total_supply();
                    PreTotalSupply::<T>::insert(T::BtcAssetId::get(), total_supply);
                }
                Chain::Dogecoin => {
                    let total_supply = T::DogecoinTotalSupply::total_supply();
                    PreTotalSupply::<T>::insert(T::DogeAssetId::get(), total_supply);
                }
                _ => (),
            }
        }
        Ok(())
    }

    // Make sure the hot and cold pubkey are set and do not check the validity of the address
    pub fn ensure_set_address(who: &T::AccountId, chain: Chain) -> bool {
        Self::trustee_intention_props_of(who, chain).is_some()
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
            Chain::Dogecoin => {
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
                let session_info =
                    T::DogecoinTrustee::generate_trustee_session_info(props, config)?;

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
        let session_number = Self::trustee_session_info_len(chain)
            .checked_add(1)
            .unwrap_or(0u32);
        let mut session_info = Self::try_generate_session_info(chain, new_trustees)?;
        Self::alter_trustee_session(chain, session_number, &mut session_info)
    }

    fn cancel_trustee_transition_impl(chain: Chain) -> DispatchResult {
        let session_number = Self::trustee_session_info_len(chain).saturating_sub(1);
        let trustee_info = Self::trustee_session_info_of(chain, session_number)
            .ok_or(Error::<T>::InvalidTrusteeSession)?;

        let trustees = trustee_info
            .0
            .trustee_list
            .clone()
            .into_iter()
            .unzip::<_, _, _, Vec<u64>>()
            .0;

        let mut session_info = Self::try_generate_session_info(chain, trustees)?;
        session_info.0 = trustee_info;

        Self::alter_trustee_session(chain, session_number, &mut session_info)
    }

    fn alter_trustee_session(
        chain: Chain,
        session_number: u32,
        session_info: &mut (
            GenericTrusteeSessionInfo<T::AccountId, T::BlockNumber>,
            ScriptInfo<T::AccountId>,
        ),
    ) -> DispatchResult {
        let multi_addr = Self::generate_multisig_addr(chain, &session_info.0)?;
        session_info.0 .0.multi_account = Some(multi_addr.clone());

        TrusteeSessionInfoLen::<T>::insert(chain, session_number);
        TrusteeSessionInfoOf::<T>::insert(chain, session_number, session_info.0.clone());
        TrusteeMultiSigAddr::<T>::insert(chain, multi_addr);
        // Remove the information of the previous aggregate public key，Withdrawal is prohibited at this time.
        match chain {
            Chain::Bitcoin => {
                AggPubkeyInfo::<T>::remove_all(None);
                for index in 0..session_info.1.agg_pubkeys.len() {
                    AggPubkeyInfo::<T>::insert(
                        &session_info.1.agg_pubkeys[index],
                        session_info.1.personal_accounts[index].clone(),
                    );
                }
            }
            Chain::Dogecoin => {
                HotPubkeyInfo::<T>::remove_all(None);
                let trustees = session_info.0 .0.trustee_list.clone();
                for (trustee, _) in trustees {
                    if let Some(trustee_info) = Self::trustee_intention_props_of(&trustee, chain) {
                        let hot_key = trustee_info.0.hot_entity;
                        HotPubkeyInfo::<T>::insert(hot_key, trustee.clone());
                    }
                }
            }
            _ => return Err(Error::<T>::NotSupportedChain.into()),
        }

        TrusteeAdmin::<T>::kill();

        Self::deposit_event(Event::<T>::TrusteeSetChanged(
            chain,
            session_number,
            session_info.0.clone(),
            session_info.1.agg_pubkeys.len() as u32,
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
            return Err(Error::<T>::DuplicatedMultiAddress.into());
        }
        Ok(multi_addr)
    }

    pub fn trustee_multisigs() -> BTreeMap<Chain, T::AccountId> {
        TrusteeMultiSigAddr::<T>::iter().collect()
    }
}

/// Trustee Reward
impl<T: Config> Pallet<T> {
    pub fn apply_claim_trustee_reward<TrusteeAddrInfo: BytesLike>(
        chain: Chain,
        session_num: u32,
        trustee_info: TrusteeSessionInfo<T::AccountId, T::BlockNumber, TrusteeAddrInfo>,
    ) -> DispatchResult {
        let multi_account = match trustee_info.multi_account.clone() {
            None => return Err(Error::<T>::InvalidMultiAccount.into()),
            Some(n) => n,
        };

        match Self::alloc_native_reward(&multi_account, &trustee_info) {
            Ok(total_native_reward) => {
                if !total_native_reward.is_zero() {
                    Self::deposit_event(Event::<T>::AllocNativeReward(
                        multi_account.clone(),
                        session_num,
                        total_native_reward,
                    ));
                }
            }
            Err(e) => return Err(e),
        }
        let asset_id = match chain {
            Chain::Bitcoin => T::BtcAssetId::get(),
            Chain::Dogecoin => T::DogeAssetId::get(),
            _ => return Err(Error::<T>::NotSupportedChain.into()),
        };
        match Self::alloc_not_native_reward(&multi_account, asset_id, &trustee_info) {
            Ok(total_asset_reward) => {
                if !total_asset_reward.is_zero() {
                    Self::deposit_event(Event::<T>::AllocNotNativeReward(
                        multi_account,
                        session_num,
                        asset_id,
                        total_asset_reward,
                    ));
                }
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn compute_reward<Balance, TrusteeAddrInfo>(
        reward: Balance,
        trustee_info: &TrusteeSessionInfo<T::AccountId, T::BlockNumber, TrusteeAddrInfo>,
    ) -> Result<RewardInfo<T::AccountId, Balance>, DispatchError>
    where
        Balance: Saturating + CheckedDiv + Zero + Copy,
        u64: UniqueSaturatedInto<Balance>,
        TrusteeAddrInfo: BytesLike,
    {
        let sum_weight = trustee_info
            .trustee_list
            .iter()
            .map(|n| n.1)
            .sum::<u64>()
            .saturated_into::<Balance>();

        let trustee_len = trustee_info.trustee_list.len();
        let mut reward_info = RewardInfo { rewards: vec![] };
        let mut acc_balance = Balance::zero();
        for i in 0..trustee_len - 1 {
            let trustee_weight = trustee_info.trustee_list[i].1.saturated_into::<Balance>();
            let amount = reward
                .saturating_mul(trustee_weight)
                .checked_div(&sum_weight)
                .ok_or(Error::<T>::InvalidTrusteeWeight)?;
            reward_info
                .rewards
                .push((trustee_info.trustee_list[i].0.clone(), amount));
            acc_balance = acc_balance.saturating_add(amount);
        }
        let amount = reward.saturating_sub(acc_balance);
        reward_info
            .rewards
            .push((trustee_info.trustee_list[trustee_len - 1].0.clone(), amount));
        Ok(reward_info)
    }

    fn alloc_native_reward<TrusteeAddrInfo: BytesLike>(
        from: &T::AccountId,
        trustee_info: &TrusteeSessionInfo<T::AccountId, T::BlockNumber, TrusteeAddrInfo>,
    ) -> Result<Balanceof<T>, DispatchError> {
        let total_reward = <T as xpallet_gateway_records::Config>::Currency::free_balance(from);
        if total_reward.is_zero() {
            return Ok(Balanceof::<T>::zero());
        }
        let reward_info = Self::compute_reward(total_reward, trustee_info)?;
        for (acc, amount) in reward_info.rewards.iter() {
            <T as xpallet_gateway_records::Config>::Currency::transfer(
                from,
                acc,
                *amount,
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|e| {
                error!(
                    target: "runtime::gateway::common",
                    "[apply_claim_trustee_reward] error {:?}, sum_balance:{:?}, reward_info:{:?}.",
                    e, total_reward, reward_info.clone()
                );
                e
            })?;
        }
        Ok(total_reward)
    }

    fn alloc_not_native_reward<TrusteeAddrInfo: BytesLike>(
        from: &T::AccountId,
        asset_id: T::AssetId,
        trustee_info: &TrusteeSessionInfo<T::AccountId, T::BlockNumber, TrusteeAddrInfo>,
    ) -> Result<T::Balance, DispatchError> {
        let total_reward = pallet_assets::Pallet::<T>::balance(asset_id, from);
        if total_reward.is_zero() {
            return Ok(T::Balance::zero());
        }
        let reward_info = Self::compute_reward(total_reward, trustee_info)?;
        for (acc, amount) in reward_info.rewards.iter() {
            <pallet_assets::Pallet<T> as fungibles::Transfer<T::AccountId>>::transfer(
                asset_id, from, acc, *amount, false,
            )
            .map_err(|e| {
                error!(
                    target: "runtime::gateway::common",
                    "[apply_claim_trustee_reward] error {:?}, sum_balance:{:?}, asset_id: {:?},reward_info:{:?}.",
                    e, total_reward, asset_id, reward_info.clone()
                );
                e
            })?;
        }
        Ok(total_reward)
    }
}

/// Ensure trustee admin
impl<T: Config> Pallet<T> {
    fn try_ensure_trustee_admin(origin: OriginFor<T>) -> Result<(), OriginFor<T>> {
        match ensure_signed(origin.clone()) {
            Ok(who) => {
                if Some(who) != Self::trustee_admin() {
                    return Err(origin);
                }
            }
            Err(_) => return Err(origin),
        }

        Ok(())
    }
}

/// Rpc Calls
impl<T: Config> Pallet<T> {
    pub fn withdrawal_limit(
        asset_id: &T::AssetId,
    ) -> Result<WithdrawalLimit<T::Balance>, DispatchError> {
        let chain = xpallet_gateway_records::Pallet::<T>::chain_of(asset_id)?;
        match chain {
            Chain::Bitcoin => T::Bitcoin::withdrawal_limit(asset_id),
            Chain::Dogecoin => T::Dogecoin::withdrawal_limit(asset_id),
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
        > = xpallet_gateway_records::Pallet::<T>::pending_withdrawal_set()
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
}
