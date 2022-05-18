// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

#![allow(clippy::type_complexity)]
use codec::{Decode, Encode};
use std::{cell::RefCell, collections::BTreeMap, time::Duration};

use hex_literal::hex;

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;
use frame_support::{
    parameter_types, sp_io,
    traits::{LockIdentifier, UnixTime},
    weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::{blake2_256, H256};
use sp_keyring::sr25519;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32, Perbill,
};

use sherpax_primitives::AssetId;
use xp_assets_registrar::Chain;
pub use xp_protocol::{X_BTC, X_DOGE, X_ETH};
use xpallet_gateway_common::{trustees, types::TrusteeInfoConfig};

use light_bitcoin::{
    chain::BlockHeader,
    keys::Network,
    primitives::{h256_rev, Compact},
    serialization::{self, Reader},
};
use xpallet_support::traits::MultisigAddressFor;

use crate::{self as xpallet_gateway_dogecoin, types::DogeParams, Config, Error};

/// The AccountId alias in this test module.
pub(crate) type AccountId = AccountId32;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        XGatewayRecords: xpallet_gateway_records::{Pallet, Call, Storage, Event<T>},
        XGatewayCommon: xpallet_gateway_common::{Pallet, Call, Storage, Event<T>, Config<T>},
        XGatewayDogecoin: xpallet_gateway_dogecoin::{Pallet, Call, Storage, Event<T>, Config<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 0;
    pub const MaxReserves: u32 = 50;
}
impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

frame_support::parameter_types! {
    pub static VotingBondBase: u64 = 2;
    pub static VotingBondFactor: u64 = 0;
    pub static CandidacyBond: u64 = 3;
    pub static DesiredMembers: u32 = 2;
    pub static DesiredRunnersUp: u32 = 0;
    pub static TermDuration: u64 = 5;
    pub static Members: Vec<u64> = vec![];
    pub static Prime: Option<u64> = None;
}

impl pallet_elections_phragmen::Config for Test {
    type Event = ();
    type PalletId = ElectionsPhragmenPalletId;
    type Currency = Balances;
    type ChangeMembers = ();
    type InitializeMembers = ();
    type CurrencyToVote = frame_support::traits::SaturatingCurrencyToVote;
    type CandidacyBond = CandidacyBond;
    type VotingBondBase = VotingBondBase;
    type VotingBondFactor = VotingBondFactor;
    type LoserCandidate = ();
    type KickedMember = ();
    type DesiredMembers = DesiredMembers;
    type DesiredRunnersUp = DesiredRunnersUp;
    type TermDuration = TermDuration;
    type WeightInfo = ();
}
parameter_types! {
    pub const AssetDeposit: Balance = 1;
    pub const ApprovalDeposit: Balance = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 1;
    pub const MetadataDepositPerByte: Balance = 1;
}

impl pallet_assets::Config for Test {
    type Event = ();
    type Balance = Balance;
    type AssetId = AssetId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = XGatewayRecords;
    type Extra = ();
    type AssetAccountDeposit = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Test>;
}

// assets
parameter_types! {
    pub const DogeAssetId: AssetId = 9;
}

impl xpallet_gateway_records::Config for Test {
    type Event = ();
    type Currency = Balances;
    type BtcAssetId = ();
    type DogeAssetId = DogeAssetId;
    type WeightInfo = ();
}

pub struct MultisigAddr;
impl MultisigAddressFor<AccountId> for MultisigAddr {
    fn calc_multisig(who: &[AccountId], threshold: u16) -> AccountId {
        let entropy = (b"modlpy/utilisuba", who, threshold).using_encoded(blake2_256);
        AccountId::decode(&mut &entropy[..]).unwrap()
    }
}

impl xpallet_gateway_common::Config for Test {
    type Event = ();
    type Validator = ();
    type DetermineMultisigAddress = MultisigAddr;
    type CouncilOrigin = EnsureSigned<AccountId>;
    type Bitcoin = ();
    type BitcoinTrustee = ();
    type BitcoinTrusteeSessionProvider = ();
    type BitcoinTotalSupply = ();
    type BitcoinWithdrawalProposal = ();
    type Dogecoin = XGatewayDogecoin;
    type DogecoinTrustee = XGatewayDogecoin;
    type DogecoinTrusteeSessionProvider = trustees::dogecoin::DogeTrusteeSessionManager<Test>;
    type DogecoinTotalSupply = XGatewayDogecoin;
    type DogecoinWithdrawalProposal = XGatewayDogecoin;
    type WeightInfo = ();
}
thread_local! {
    pub static NOW: RefCell<Option<Duration>> = RefCell::new(None);
}

pub struct Timestamp;
impl UnixTime for Timestamp {
    fn now() -> Duration {
        NOW.with(|m| {
            m.borrow().unwrap_or_else(|| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let start = SystemTime::now();
                start
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
            })
        })
    }
}

impl Config for Test {
    type Event = ();
    type UnixTime = Timestamp;
    type CouncilOrigin = EnsureSigned<AccountId>;
    type AccountExtractor = xp_gateway_dogecoin::OpReturnExtractor;
    type TrusteeSessionProvider =
        xpallet_gateway_common::trustees::dogecoin::DogeTrusteeSessionManager<Test>;
    type TrusteeInfoUpdate = XGatewayCommon;
    type ReferralBinding = XGatewayCommon;
    type AddressBinding = XGatewayCommon;
    type WeightInfo = ();
}

pub type XGatewayDogecoinErr = Error<Test>;

pub struct ExtBuilder;
impl Default for ExtBuilder {
    fn default() -> Self {
        Self
    }
}
impl ExtBuilder {
    pub fn build_mock(
        self,
        btc_genesis: (BlockHeader, u32),
        btc_network: Network,
    ) -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        // let (genesis_info, genesis_hash, network_id) = load_mock_btc_genesis_header_info();
        let genesis_hash = btc_genesis.0.hash();
        let network_id = btc_network;
        let _ = xpallet_gateway_dogecoin::GenesisConfig::<Test> {
            genesis_trustees: vec![],
            genesis_info: btc_genesis,
            genesis_hash,
            network_id,
            params_info: DogeParams::new(
                545259519,            // max_bits
                2 * 60 * 60,          // block_max_future
                2 * 7 * 24 * 60 * 60, // target_timespan_seconds
                10 * 60,              // target_spacing_seconds
                4,                    // retargeting_factor
            ), // retargeting_factor
            confirmation_number: 4,
            doge_withdrawal_fee: 0,
            max_withdrawal_count: 100,
        }
        .assimilate_storage(&mut storage);

        sp_io::TestExternalities::new(storage)
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let info = trustees_info();
        let genesis_trustees = info
            .iter()
            .find_map(|(chain, _, trustee_params)| {
                if *chain == Chain::Dogecoin {
                    Some(
                        trustee_params
                            .iter()
                            .map(|i| (i.0).clone())
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .unwrap();

        let _ = xpallet_gateway_common::GenesisConfig::<Test> {
            trustees: info,
            genesis_trustee_transition_status: Default::default(),
        }
        .assimilate_storage(&mut storage);

        let (genesis_info, genesis_hash, network_id) = load_dogecoin_genesis_header_info();

        let _ = xpallet_gateway_dogecoin::GenesisConfig::<Test> {
            genesis_trustees,
            genesis_info,
            genesis_hash,
            network_id,
            params_info: DogeParams::new(
                545259519,            // max_bits
                2 * 60 * 60,          // block_max_future
                2 * 7 * 24 * 60 * 60, // target_timespan_seconds
                10 * 60,              // target_spacing_seconds
                4,                    // retargeting_factor
            ), // retargeting_factor
            confirmation_number: 4,
            doge_withdrawal_fee: 0,
            max_withdrawal_count: 100,
        }
        .assimilate_storage(&mut storage);

        let _ = xpallet_gateway_records::GenesisConfig::<Test> {
            initial_asset_chain: vec![
                (X_BTC, Chain::Bitcoin),
                (X_ETH, Chain::Ethereum),
                (X_DOGE, Chain::Dogecoin),
            ],
        }
        .assimilate_storage(&mut storage);

        let _ = pallet_assets::GenesisConfig::<Test> {
            assets: vec![
                (X_BTC, alice(), true, 1),
                (X_ETH, alice(), true, 1),
                (X_DOGE, alice(), true, 1),
            ],
            metadata: vec![
                (
                    X_BTC,
                    "XBTC".to_string().into_bytes(),
                    "XBTC".to_string().into_bytes(),
                    8,
                ),
                (
                    X_ETH,
                    "XETH".to_string().into_bytes(),
                    "XETH".to_string().into_bytes(),
                    18,
                ),
                (
                    X_DOGE,
                    "XDOGE".to_string().into_bytes(),
                    "XDOGE".to_string().into_bytes(),
                    8,
                ),
            ],
            accounts: vec![],
        }
        .assimilate_storage(&mut storage);

        sp_io::TestExternalities::new(storage)
    }
    pub fn build_and_execute(self, test: impl FnOnce()) {
        let mut ext = self.build();
        ext.execute_with(|| System::set_block_number(1));
        ext.execute_with(test);
    }
}

pub fn alice() -> AccountId32 {
    sr25519::Keyring::Alice.to_account_id()
}
pub fn bob() -> AccountId32 {
    sr25519::Keyring::Bob.to_account_id()
}
pub fn charlie() -> AccountId32 {
    sr25519::Keyring::Charlie.to_account_id()
}
pub fn trustees() -> Vec<(AccountId32, Vec<u8>, Vec<u8>, Vec<u8>)> {
    vec![
        (
            alice(),
            b"Alice".to_vec(),
            hex!("042f7e2f0f3e912bf416234913b388393beb5092418fea986e45c0b9633adefd85168f3b1d13ae29651c29e424760b3795fc78152ac119e0dc4e2b9055329099b3").to_vec(),
            hex!("0400849497d4f88ebc3e1bc2583677c5abdbd3b63640b3c5c50cd4628a33a2a2cab6b69094b5a213da80f9ef730fab39de770ca124f2d9a9cb161856be54b9adc5").to_vec(),
        ),
        (
            bob(),
            b"Bob".to_vec(),
            hex!("0451e0dc3d9709d860c49785fc84b62909d991cffd81592f6994c452438f91b6a2e586541c4b3bc1ebeb5fb9fad2ed2e696b2175c54458ab6f103717cbeeb4e52c").to_vec(),
            hex!("042122032ae9656f9a133405ffe02101469a8d62002270a33ceccf0e40dda54d08c989b55f1b6b46a8dee284cf6737de0a377e410bcfd361a015528ae80a349529").to_vec(),
        ),
        (
            charlie(),
            b"Charlie".to_vec(),
            hex!("04a09e8182977710bab64472c0ecaf9e52255a890554a00a62facd05c0b13817f8995bf590851c19914bfc939d53365b90cc2f0fcfddaca184f0c1e7ce1736f0b8").to_vec(),
            hex!("04b3cc747f572d33f12870fa6866aebbfd2b992ba606b8dc89b676b3697590ad63d5ca398bdb6f8ee619f2e16997f21e5e8f0e0b00e2f275c7cb1253f381058d56").to_vec(),
        ),
    ]
}

pub fn load_dogecoin_genesis_header_info() -> ((BlockHeader, u32), H256, Network) {
    (
        (
            BlockHeader {
                version: 6422788,
                previous_header_hash: h256_rev(
                    "763abbde93dfda3034a74ae7db661a6bacbd4569d5b1d81972c9ab566366b3e5",
                ),
                merkle_root_hash: h256_rev(
                    "ff758444fe236b999c58b4f0653ec6ad8978e8ec3e12beadf7c544cc87b592c7",
                ),
                time: 1652868482,
                bits: Compact::new(494032418),
                nonce: 0,
            },
            3836100,
        ),
        h256_rev("97e7095b5d8cfa2722618c7c1c755965754a1acaad81954b975017c31f61e9c5"),
        Network::DogeCoinTestnet,
    )
}

fn trustees_info() -> Vec<(
    Chain,
    TrusteeInfoConfig,
    Vec<(AccountId, Vec<u8>, Vec<u8>, Vec<u8>)>,
)> {
    let btc_trustees = trustees();
    let btc_config = TrusteeInfoConfig {
        min_trustee_count: 3,
        max_trustee_count: 15,
    };
    vec![(Chain::Dogecoin, btc_config, btc_trustees)]
}

pub fn generate_blocks_3836100_3836160() -> BTreeMap<u32, BlockHeader> {
    let headers = include_str!("./res/headers-3836100-3836160.json");
    let headers: Vec<(u32, String)> = serde_json::from_str(headers).unwrap();
    headers
        .into_iter()
        .map(|(height, header_hex)| {
            let data = hex::decode(header_hex).unwrap();
            let header = serialization::deserialize(Reader::new(&data)).unwrap();
            (height, header)
        })
        .collect()
}

pub fn generate_blocks_478557_478563() -> (u32, Vec<BlockHeader>, Vec<BlockHeader>) {
    let b0 = BlockHeader {
        version: 0x20000002,
        previous_header_hash: h256_rev(
            "0000000000000000004801aaa0db00c30a6c8d89d16fd30a2115dda5a9fc3469",
        ),
        merkle_root_hash: h256_rev(
            "b2f6c37fb65308f2ff12cfc84e3b4c8d49b02534b86794d7f1dd6d6457327200",
        ),
        time: 1501593084,
        bits: Compact::new(0x18014735),
        nonce: 0x7a511539,
    }; // 478557  btc/bch common use

    let b1: BlockHeader = BlockHeader {
        version: 0x20000002,
        previous_header_hash: h256_rev(
            "000000000000000000eb9bc1f9557dc9e2cfe576f57a52f6be94720b338029e4",
        ),
        merkle_root_hash: h256_rev(
            "5b65144f6518bf4795abd428acd0c3fb2527e4e5c94b0f5a7366f4826001884a",
        ),
        time: 1501593374,
        bits: Compact::new(0x18014735),
        nonce: 0x7559dd16,
    }; //478558  bch forked from here

    let b2: BlockHeader = BlockHeader {
        version: 0x20000002,
        previous_header_hash: h256_rev(
            "0000000000000000011865af4122fe3b144e2cbeea86142e8ff2fb4107352d43",
        ),
        merkle_root_hash: h256_rev(
            "5fa62e1865455037450b7275d838d04f00230556129a4e86621a6bc4ad318c18",
        ),
        time: 1501593780,
        bits: Compact::new(0x18014735),
        nonce: 0xb78dbdba,
    }; // 478559

    let b3: BlockHeader = BlockHeader {
        version: 0x20000002,
        previous_header_hash: h256_rev(
            "00000000000000000019f112ec0a9982926f1258cdcc558dd7c3b7e5dc7fa148",
        ),
        merkle_root_hash: h256_rev(
            "8bd5e10005d8e01aa60278def2025d39b5a441261d934a24bd39e7423866787c",
        ),
        time: 1501594184,
        bits: Compact::new(0x18014735),
        nonce: 0x43628196,
    }; // 478560

    let b4: BlockHeader = BlockHeader {
        version: 0x20000002,
        previous_header_hash: h256_rev(
            "000000000000000000e512213f7303f72c5f7446e6e295f73c28cb024dd79e34",
        ),
        merkle_root_hash: h256_rev(
            "aaa533386910909ed6e6319a3ed2bb86774a8d1d9b373f975d53daad6b12170e",
        ),
        time: 1501594485,
        bits: Compact::new(0x18014735),
        nonce: 0xdabcc394,
    }; // 478561

    let b5: BlockHeader = BlockHeader {
        version: 0x20000002,
        previous_header_hash: h256_rev(
            "0000000000000000008876768068eea31f8f34e2f029765cd2ac998bdc3a2b2d",
        ),
        merkle_root_hash: h256_rev(
            "a51effefcc9eaac767ea211c661e5393d38bf3577b5b7e2d54471098b0ac4e35",
        ),
        time: 1501594711,
        bits: Compact::new(0x18014735),
        nonce: 0xa07f1745,
    }; // 478562

    let b2_fork: BlockHeader = BlockHeader {
        version: 0x20000000,
        previous_header_hash: h256_rev(
            "0000000000000000011865af4122fe3b144e2cbeea86142e8ff2fb4107352d43",
        ),
        merkle_root_hash: h256_rev(
            "c896c91a0be4d3eed5568bab4c3084945e5e06669be38ec06b1c8ca4d84baaab",
        ),
        time: 1501611161,
        bits: Compact::new(0x18014735),
        nonce: 0xe84aca22,
    }; // 478559

    let b3_fork: BlockHeader = BlockHeader {
        version: 0x20000000,
        previous_header_hash: h256_rev(
            "000000000000000000651ef99cb9fcbe0dadde1d424bd9f15ff20136191a5eec",
        ),
        merkle_root_hash: h256_rev(
            "088a7d29c4c6b95a74e362d64a801f492e748369a4fec1ca4e1ab47eefc8af82",
        ),
        time: 1501612386,
        bits: Compact::new(0x18014735),
        nonce: 0xcb72a740,
    }; // 478560
    let b4_fork: BlockHeader = BlockHeader {
        version: 0x20000002,
        previous_header_hash: h256_rev(
            "000000000000000000b15ad892af8f6aca4462d46d0b6e5884cadc033c8f257b",
        ),
        merkle_root_hash: h256_rev(
            "f64de8adf8dac328fb8f1dcb4ba19b6e94de7abc8c4eeaae83df8f62504e8758",
        ),
        time: 1501612639,
        bits: Compact::new(0x18014735),
        nonce: 0x0310f5e2,
    }; // 478561
    let b5_fork: BlockHeader = BlockHeader {
        version: 0x20000000,
        previous_header_hash: h256_rev(
            "00000000000000000013ee8874665f73862a3a0b6a30f895fe34f4c94d3e8a15",
        ),
        merkle_root_hash: h256_rev(
            "a464516af1dab6eadb963b62c5df0e503c8908af503dfff7a169b9d3f9851b11",
        ),
        time: 1501613578,
        bits: Compact::new(0x18014735),
        nonce: 0x0a24f4c4,
    }; // 478562
    let b6_fork: BlockHeader = BlockHeader {
        version: 0x20000000,
        previous_header_hash: h256_rev(
            "0000000000000000005c6e82aa704d326a3a2d6a4aa09f1725f532da8bb8de4d",
        ),
        merkle_root_hash: h256_rev(
            "a27fac4ab26df6e12a33b2bb853140d7e231326ddbc9a1d6611b553b0645a040",
        ),
        time: 1501616264,
        bits: Compact::new(0x18014735),
        nonce: 0x6bd75df1,
    }; // 478563

    (
        478557,
        vec![b0, b1, b2, b3, b4, b5],
        vec![b0, b1, b2_fork, b3_fork, b4_fork, b5_fork, b6_fork],
    )
}
