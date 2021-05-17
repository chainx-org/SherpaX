// Copyright 2020-2021 ChainX
// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use frame_system::EnsureSignedBy;
use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, Perbill,
};
use sp_std::{
    prelude::{Box, Vec},
    vec,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
use frame_support::{
    construct_runtime,
    instances::{Instance1, Instance2},
    parameter_types,
    traits::Randomness,
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
        DispatchClass, IdentityFee, Weight,
    },
    PalletId,
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_transaction_payment_rpc_runtime_api::{FeeDetails, RuntimeDispatchInfo};

use dev_parachain_primitives::*;
use xgateway_bitcoin_bridge::pallet as xgateway_bitcoin_bridge_pallet;

/// Constant values used within the runtime.
pub mod constants;
use constants::{currency::*, time::*};

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub type SessionHandlers = ();

impl_opaque_keys! {
    pub struct SessionKeys {}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("chainx-parachain"),
    impl_name: create_runtime_str!("chainx-parachain"),
    authoring_version: 1,
    spec_version: 50,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
    pub const SS58Prefix: u8 = 42;
    pub const BlockHashCount: BlockNumber = 250;
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

}

impl frame_system::Config for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Runtime version.
    type Version = Version;
    /// Converts a module to an index of this module in the runtime.
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = RuntimeBlockLength;
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
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TransactionByteFee: Balance = 10 * MILLICENTS;
    pub const ExistentialDeposit: Balance = 100 * MILLICENTS;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type MaxLocks = MaxLocks;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
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
    type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

impl xpallet_gateway_records::Config for Runtime {
    type Event = Event;
    type WeightInfo = xpallet_gateway_records::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}
parameter_types! {
    pub const DustCollateral: Balance = 1000;
    pub const SecureThreshold: u16 = 300;
    pub const PremiumThreshold: u16 = 250;
    pub const LiquidationThreshold: u16 = 180;
    pub const IssueRequestExpiredPeriod: BlockNumber = 48 * 600;
    pub const RedeemRequestExpiredPeriod: BlockNumber = 48 * 600;
    pub const ExchangeRateExpiredPeriod: BlockNumber = 48 * 600;
}

parameter_types! {
    //bitcoin
    pub const BridgeBtcAssetId: u32 = xp_protocol::C_BTC;
    pub const BridgeTokenBtcAssetId: u32 = xp_protocol::S_BTC;
    pub const BtcMinimumRedeemValue: Balance = 10000;

    //dogecoin
    pub const BridgeDogeAssetId: u32 = xp_protocol::C_DOGE;
    pub const BridgeTokenDogeAssetId: u32 = xp_protocol::S_DOGE;
    pub const DogeMinimumRedeemValue: Balance = 100000000;
}

impl xgateway_bitcoin_bridge::pallet::Config<Instance1> for Runtime {
    type Event = Event;
    type TargetAssetId = BridgeBtcAssetId;
    type TokenAssetId = BridgeTokenBtcAssetId;
    type MinimumRedeemValue = BtcMinimumRedeemValue;
    type DustCollateral = DustCollateral;
    type SecureThreshold = SecureThreshold;
    type PremiumThreshold = PremiumThreshold;
    type LiquidationThreshold = LiquidationThreshold;
    type IssueRequestExpiredPeriod = IssueRequestExpiredPeriod;
    type RedeemRequestExpiredPeriod = RedeemRequestExpiredPeriod;
    type ExchangeRateExpiredPeriod = ExchangeRateExpiredPeriod;
}

impl xgateway_bitcoin_bridge::pallet::Config<Instance2> for Runtime {
    type Event = Event;
    type TargetAssetId = BridgeDogeAssetId;
    type TokenAssetId = BridgeTokenDogeAssetId;
    type MinimumRedeemValue = BtcMinimumRedeemValue;
    type DustCollateral = DustCollateral;
    type SecureThreshold = SecureThreshold;
    type PremiumThreshold = PremiumThreshold;
    type LiquidationThreshold = LiquidationThreshold;
    type IssueRequestExpiredPeriod = IssueRequestExpiredPeriod;
    type RedeemRequestExpiredPeriod = RedeemRequestExpiredPeriod;
    type ExchangeRateExpiredPeriod = ExchangeRateExpiredPeriod;
}

pub struct TrusteeProvider<AccountId: Ord>(sp_std::marker::PhantomData<AccountId>);
impl<AccountId: Ord> frame_support::traits::SortedMembers<AccountId>
    for TrusteeProvider<AccountId>
{
    fn sorted_members() -> Vec<AccountId> {
        vec![]
    }
}

impl xpallet_gateway_bitcoin::Config<Instance1> for Runtime {
    type Event = Event;
    type UnixTime = Timestamp;
    type AccountExtractor = xp_gateway_bitcoin::OpReturnExtractor;
    type TrusteeSessionProvider = ();
    type TrusteeOrigin = EnsureSignedBy<TrusteeProvider<AccountId>, AccountId>;
    type ReferralBinding = ();
    type AddressBinding = ();
    type WeightInfo = xpallet_gateway_bitcoin::weights::SubstrateWeight<Runtime>;
}

impl xpallet_gateway_bitcoin::Config<Instance2> for Runtime {
    type Event = Event;
    type UnixTime = Timestamp;
    type AccountExtractor = xp_gateway_bitcoin::OpReturnExtractor;
    type TrusteeSessionProvider = ();
    type TrusteeOrigin = EnsureSignedBy<TrusteeProvider<AccountId>, AccountId>;
    type ReferralBinding = ();
    type AddressBinding = ();
    type WeightInfo = xpallet_gateway_bitcoin::weights::SubstrateWeight<Runtime>;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type OnValidationData = ();
    type SelfParaId = parachain_info::Module<Runtime>;
    type DownwardMessageHandlers = ();
    type OutboundXcmpMessageSource = ();
    type XcmpMessageHandler = ();
    type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Runtime {}

parameter_types! {
    pub const PcxAssetId: u32 = 0;
}

impl xpallet_assets_registrar::Config for Runtime {
    type Event = Event;
    type NativeAssetId = PcxAssetId;
    type RegistrarHandler = ();
    type WeightInfo = xpallet_assets_registrar::weights::SubstrateWeight<Runtime>;
}

impl xpallet_assets::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type Amount = Amount;
    type TreasuryAccount = ();
    type OnCreatedAccount = frame_system::Provider<Runtime>;
    type OnAssetChanged = ();
    type WeightInfo = xpallet_assets::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const SwapPalletId: PalletId = PalletId(*b"//swap//");
}

impl pallet_swap::Config for Runtime {
    type Event = Event;
    type NativeAssetId = PcxAssetId;
    type MultiAsset = pallet_swap::SimpleMultiAsset<Self>;
    type PalletId = SwapPalletId;
}

construct_runtime! {
    pub enum Runtime where
        Block = Block,
        NodeBlock = dev_parachain_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 1,
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 2,
        Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Call, Storage} = 4,
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>} = 5,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 6,
        ParachainInfo: parachain_info::{Pallet, Storage, Config} = 7,
        Utility: pallet_utility::{Pallet, Call, Event} = 8,
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 9,

        XAssetsRegistrar: xpallet_assets_registrar::{Pallet, Call, Storage, Event, Config} = 10,
        XAssets: xpallet_assets::{Pallet, Call, Storage, Event<T>, Config<T>} = 11,

        Swap: pallet_swap::{Pallet, Call, Storage, Event<T>} = 12,
        XGatewayBitcoin: xpallet_gateway_bitcoin::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 13,
        XGatewayDogecoin: xpallet_gateway_bitcoin::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>} = 14,
        XGatewayBitcoinBridge: xgateway_bitcoin_bridge_pallet::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 15,
        XGatewayDogecoinBridge: xgateway_bitcoin_bridge_pallet::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>} = 16,
        XGatewayRecord: xpallet_gateway_records::{Pallet, Call, Storage, Event<T>} = 17,
    }
}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
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
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPallets,
>;

impl_runtime_apis! {
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
            Runtime::metadata().into()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(
            extrinsic: <Block as BlockT>::Extrinsic,
        ) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed().0
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }

        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block,
        Balance,
    > for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }
}

cumulus_pallet_parachain_system::register_validate_block!(Runtime, Executive);
