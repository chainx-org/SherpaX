// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! sherpax runtime.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod constants;
mod migration;

use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};

use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use constants::{currency::*, fee::WeightToFee};
use frame_support::{
    construct_runtime, parameter_types,
    traits::Contains,
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight},
        DispatchClass, Weight,
    },
    PalletId,
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
use runtime_common::{
    impls::DealWithFees, opaque, AccountId, AuraId, Balance, BlockNumber, Hash, Header, Index,
    Signature, AVERAGE_ON_INITIALIZE_RATIO, HOURS, MINUTES, NORMAL_DISPATCH_RATIO, SLOT_DURATION,
    WEIGHT_PER_SECOND,
};
use sp_runtime::{traits::ConvertInto, Perbill};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot imports
use polkadot_runtime_common::{BlockHashCount, RocksDbWeight, SlowAdjustingFeeUpdate};

// evm
mod precompiles;
pub use precompiles::SherpaXPrecompiles;

use codec::{Decode, Encode};
use fp_rpc::TransactionStatus;
use pallet_ethereum::{Call::transact, Transaction as EthereumTransaction};
#[cfg(feature = "std")]
pub use pallet_evm::GenesisAccount;
use pallet_evm::{
    Account as EVMAccount, EnsureAddressNever, EnsureAddressRoot, FeeCalculator,
    HashedAddressMapping, Runner,
};
use sp_core::{H160, H256, U256};
use sp_runtime::{
    traits::{Dispatchable, PostDispatchInfoOf},
    transaction_validity::TransactionValidityError,
};

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
    }
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("sherpax"),
    impl_name: create_runtime_str!("sherpax"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    pub const SS58Prefix: u8 = 44;
}

pub struct BaseFilter;
impl Contains<Call> for BaseFilter {
    fn contains(_c: &Call) -> bool {
        true
    }
}

// Configure FRAME pallets to include in runtime.
impl frame_system::Config for Runtime {
    type BaseCallFilter = BaseFilter;
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = RuntimeBlockLength;
    type AccountId = AccountId;
    type Call = Call;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Event = Event;
    type Origin = Origin;
    type BlockHashCount = BlockHashCount;
    type DbWeight = RocksDbWeight;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = (CollatorSelection,);
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    /// Relay Chain `TransactionByteFee` / 10
    pub const TransactionByteFee: Balance = MILLICENTS;
    pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction =
        pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees<Runtime>>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = WeightToFee;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type WeightInfo = ();
}

parameter_types! {
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 2;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type OnValidationData = ();
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type OutboundXcmpMessageSource = ();
    type DmpMessageHandler = ();
    type ReservedDmpWeight = ReservedDmpWeight;
    type XcmpMessageHandler = ();
    type ReservedXcmpWeight = ();
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    pub const Period: u32 = 6 * HOURS;
    pub const Offset: u32 = 0;
    pub const DevPeriod: u32 = 2 * MINUTES;
    pub const MaxAuthorities: u32 = 100_000;
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    // we don't have stash and controller, thus we don't need the convert as well.
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    #[cfg(feature = "sherpax-dev")]
    type ShouldEndSession = pallet_session::PeriodicSessions<DevPeriod, Offset>;
    #[cfg(not(feature = "sherpax-dev"))]
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    #[cfg(feature = "sherpax-dev")]
    type NextSessionRotation = pallet_session::PeriodicSessions<DevPeriod, Offset>;
    #[cfg(not(feature = "sherpax-dev"))]
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;

    type SessionManager = CollatorSelection;
    // Essentially just Aura, but lets be pedantic.
    type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
    pub const PotId: PalletId = PalletId(*b"PotStake");
    pub const MaxCandidates: u32 = 1000;
    pub const MinCandidates: u32 = 5;
    pub const MaxInvulnerables: u32 = 100;
}

impl pallet_collator_selection::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type UpdateOrigin = EnsureRoot<AccountId>;
    type PotId = PotId;
    type MaxCandidates = MaxCandidates;
    type MinCandidates = MinCandidates;
    type MaxInvulnerables = MaxInvulnerables;
    // should be a multiple of session or things will get inconsistent
    #[cfg(feature = "sherpax-dev")]
    type KickThreshold = DevPeriod;
    #[cfg(not(feature = "sherpax-dev"))]
    type KickThreshold = Period;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ValidatorRegistration = Session;
    type WeightInfo = ();
}

impl pallet_vesting::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type BlockNumberToBalance = ConvertInto;
    type MinVestedTransfer = ExistentialDeposit;
    type WeightInfo = ();
    // `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
    // highest number of schedules that encodes less than 2^10.
    const MAX_VESTING_SCHEDULES: u32 = 28;
}

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;

/// Maximum weight per block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

parameter_types! {
    pub BlockGasLimit: U256
        = U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WEIGHT_PER_GAS);
    pub PrecompilesValue: SherpaXPrecompiles<Runtime> = SherpaXPrecompiles::<_>::new();
    pub const ChainId: u64 = 1510;
}

pub struct SherpaXGasWeightMapping;
impl pallet_evm::GasWeightMapping for SherpaXGasWeightMapping {
    fn gas_to_weight(gas: u64) -> Weight {
        gas.saturating_mul(WEIGHT_PER_GAS)
    }
    fn weight_to_gas(weight: Weight) -> u64 {
        weight.wrapping_div(WEIGHT_PER_GAS)
    }
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
    fn min_gas_price() -> U256 {
        4_500_000_000_000u128.into()
    }
}

impl pallet_evm::Config for Runtime {
    type FeeCalculator = FixedGasPrice;
    type GasWeightMapping = SherpaXGasWeightMapping;
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
    type CallOrigin = EnsureAddressRoot<AccountId>;
    type WithdrawOrigin = EnsureAddressNever<AccountId>;
    type AddressMapping = HashedAddressMapping<BlakeTwo256>;
    type Currency = Balances;
    type Event = Event;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type PrecompilesType = SherpaXPrecompiles<Runtime>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = ChainId;
    type OnChargeTransaction = ();
    type BlockGasLimit = BlockGasLimit;
    type FindAuthor = ();
}

impl pallet_ethereum::Config for Runtime {
    type Event = Event;
    type StateRoot = pallet_ethereum::IntermediateStateRoot;
}

parameter_types! {
    pub const AssetDeposit: Balance = 100 * UNITS;
    pub const ApprovalDeposit: Balance = UNITS;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 10 * UNITS;
    pub const MetadataDepositPerByte: Balance = UNITS;
}

impl pallet_assets::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type AssetId = u32;
    type Currency = Balances;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = ();
}

parameter_types! {
    // 0x1111111111111111111111111111111111111111
    pub EvmCaller: H160 = H160::from_slice(&[17u8;20][..]);
}
impl pallet_assets_bridge::Config for Runtime {
    type Event = Event;
    type EvmCaller = EvmCaller;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        // System support stuff.
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        ParachainSystem: cumulus_pallet_parachain_system::{
            Pallet, Call, Config, Storage, Inherent, Event<T>,
        } = 1,
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 2,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
        ParachainInfo: parachain_info::{Pallet, Storage, Config} = 4,
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 5,

        // Monetary stuff.
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 11,
        Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 12,
        Assets: pallet_assets::{Pallet, Call, Storage, Config<T>, Event<T>} = 13,

        // Collator support. the order of these 4 are important and shall not change.
        Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
        CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
        Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,
        AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 24,

        // Handy utilities.
        Utility: pallet_utility::{Pallet, Call, Event} = 40,
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 41,

        // Ethereum compatibility
        Evm: pallet_evm::{Pallet, Config, Call, Storage, Event<T>} = 50,
        Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, Origin} = 51,
        AssetsBridge: pallet_assets_bridge::{Pallet, Call, Storage, Config<T>, Event<T>} = 53,
    }
);

pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
    fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
        UncheckedExtrinsic::new_unsigned(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        )
    }
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
    fn convert_transaction(
        &self,
        transaction: pallet_ethereum::Transaction,
    ) -> opaque::UncheckedExtrinsic {
        let extrinsic = UncheckedExtrinsic::new_unsigned(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        );
        let encoded = extrinsic.encode();
        opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
            .expect("Encoded extrinsic is always valid")
    }
}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    fp_self_contained::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = fp_self_contained::CheckedExtrinsic<AccountId, Call, SignedExtra, H160>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPallets,
>;

impl fp_self_contained::SelfContainedCall for Call {
    type SignedInfo = H160;

    fn is_self_contained(&self) -> bool {
        match self {
            Call::Ethereum(call) => call.is_self_contained(),
            _ => false,
        }
    }

    fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.check_self_contained(),
            _ => None,
        }
    }

    fn validate_self_contained(&self, info: &Self::SignedInfo) -> Option<TransactionValidity> {
        match self {
            Call::Ethereum(call) => call.validate_self_contained(info),
            _ => None,
        }
    }

    fn pre_dispatch_self_contained(
        &self,
        info: &Self::SignedInfo,
    ) -> Option<Result<(), TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.pre_dispatch_self_contained(info),
            _ => None,
        }
    }

    fn apply_self_contained(
        self,
        info: Self::SignedInfo,
    ) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
        match self {
            call @ Call::Ethereum(pallet_ethereum::Call::transact { .. }) => Some(call.dispatch(
                Origin::from(pallet_ethereum::RawOrigin::EthereumTransaction(info)),
            )),
            _ => None,
        }
    }
}

impl_runtime_apis! {
    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().into_inner()
        }
    }

    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
            ParachainSystem::collect_collation_info()
        }
    }

    impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
        fn chain_id() -> u64 {
            <Runtime as pallet_evm::Config>::ChainId::get()
        }

        fn account_basic(address: H160) -> EVMAccount {
            Evm::account_basic(&address)
        }

        fn gas_price() -> U256 {
            <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price()
        }

        fn account_code_at(address: H160) -> Vec<u8> {
            Evm::account_codes(address)
        }

        fn author() -> H160 {
            <pallet_evm::Pallet<Runtime>>::find_author()
        }

        fn storage_at(address: H160, index: U256) -> H256 {
            let mut tmp = [0u8; 32];
            index.to_big_endian(&mut tmp);
            Evm::account_storages(address, H256::from_slice(&tmp[..]))
        }

        #[allow(clippy::redundant_closure)]
        fn call(
            from: H160,
            to: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
        ) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                Some(config)
            } else {
                None
            };

            <Runtime as pallet_evm::Config>::Runner::call(
                from,
                to,
                data,
                value,
                gas_limit.low_u64(),
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                Vec::new(),
                config.as_ref().unwrap_or_else(|| <Runtime as pallet_evm::Config>::config()),
            ).map_err(|err| err.into())
        }

        #[allow(clippy::redundant_closure)]
        fn create(
            from: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
        ) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                Some(config)
            } else {
                None
            };

            <Runtime as pallet_evm::Config>::Runner::create(
                from,
                data,
                value,
                gas_limit.low_u64(),
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                Vec::new(),
                config.as_ref().unwrap_or_else(|| <Runtime as pallet_evm::Config>::config()),
            ).map_err(|err| err.into())
        }

        fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
            Ethereum::current_transaction_statuses()
        }

        fn current_block() -> Option<pallet_ethereum::Block> {
            Ethereum::current_block()
        }

        fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
            Ethereum::current_receipts()
        }

        fn current_all() -> (
            Option<pallet_ethereum::Block>,
            Option<Vec<pallet_ethereum::Receipt>>,
            Option<Vec<TransactionStatus>>
        ) {
            (
                Ethereum::current_block(),
                Ethereum::current_receipts(),
                Ethereum::current_transaction_statuses()
            )
        }

        fn extrinsic_filter(
            xts: Vec<<Block as BlockT>::Extrinsic>,
        ) -> Vec<EthereumTransaction> {
            xts.into_iter().filter_map(|xt| match xt.0.function {
                Call::Ethereum(transact { transaction }) => Some(transaction),
                _ => None
            }).collect::<Vec<EthereumTransaction>>()
        }
    }
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
    fn check_inherents(
        block: &Block,
        relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
    ) -> sp_inherents::CheckInherentsResult {
        let relay_chain_slot = relay_state_proof
            .read_slot()
            .expect("Could not read the relay chain slot from the proof");

        let inherent_data =
            cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
                relay_chain_slot,
                sp_std::time::Duration::from_secs(6),
            )
            .create_inherent_data()
            .expect("Could not create the timestamp inherent data");

        inherent_data.check_extrinsics(block)
    }
}

cumulus_pallet_parachain_system::register_validate_block! {
    Runtime = Runtime,
    BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
    CheckInherents = CheckInherents,
}
