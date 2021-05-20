use crate as pallet_swap;
use frame_support::{parameter_types, sp_io, PalletId};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        XAssetsRegistrar: xpallet_assets_registrar::{Pallet, Call, Storage, Event, Config},
        XAssets: xpallet_assets::{Pallet, Call, Storage, Event<T>, Config<T>},
        Swap: pallet_swap::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

parameter_types! {
    pub const TransactionByteFee: u128 = 1_000_000;
    pub const ExistentialDeposit: u128 = 0;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type MaxLocks = MaxLocks;
}

parameter_types! {
    pub const PcxAssetId: u32 = 0;
}

impl xpallet_assets_registrar::Config for Test {
    type Event = Event;
    type NativeAssetId = PcxAssetId;
    type RegistrarHandler = ();
    type WeightInfo = xpallet_assets_registrar::weights::SubstrateWeight<Test>;
}

impl xpallet_assets::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type Amount = i128;
    type TreasuryAccount = ();
    type OnCreatedAccount = frame_system::Provider<Test>;
    type OnAssetChanged = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const SwapPalletId: PalletId = PalletId(*b"//swap//");
}

impl pallet_swap::Config for Test {
    type Event = Event;
    type NativeAssetId = PcxAssetId;
    type MultiAsset = pallet_swap::SimpleMultiAsset<Self>;
    type PalletId = SwapPalletId;
}

pub(crate) type AccountId = u64;
pub(crate) type AssetId = u32;
pub(crate) type Balance = u128;
pub(crate) type BlockNumber = u32;
pub const PCX: AssetId = 0;
pub const X_BTC: AssetId = 1;
pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;

use std::collections::BTreeMap;
use xpallet_assets::{AssetInfo, AssetRestrictions, Chain};

pub(crate) fn pcx() -> (AssetId, AssetInfo, AssetRestrictions) {
    (
        PCX,
        AssetInfo::new::<Test>(
            b"PCX".to_vec(),
            b"PCX".to_vec(),
            Chain::ChainX,
            8,
            b"ChainX's PCX".to_vec(),
        )
        .unwrap(),
        AssetRestrictions::DESTROY_USABLE,
    )
}

pub(crate) fn btc() -> (AssetId, AssetInfo, AssetRestrictions) {
    (
        X_BTC,
        AssetInfo::new::<Test>(
            b"X-BTC".to_vec(),
            b"X-BTC".to_vec(),
            Chain::Bitcoin,
            8,
            b"ChainX's cross-chain Bitcoin".to_vec(),
        )
        .unwrap(),
        AssetRestrictions::DESTROY_USABLE,
    )
}

#[derive(Default)]
pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build(
        self,
        assets: Vec<(AssetId, AssetInfo, AssetRestrictions, bool, bool)>,
        endowed: BTreeMap<AssetId, Vec<(AccountId, Balance)>>,
        balances: Vec<(AccountId, Balance)>,
    ) -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

        let mut init_assets = vec![];
        let mut assets_restrictions = vec![];
        for (a, b, c, d, e) in assets {
            init_assets.push((a, b, d, e));
            assets_restrictions.push((a, c))
        }
        let init_pcx = pcx();
        init_assets.push((init_pcx.0, init_pcx.1, true, true));
        let _ = xpallet_assets_registrar::GenesisConfig { assets: init_assets }
            .assimilate_storage::<Test>(&mut storage);

        let _ = xpallet_assets::GenesisConfig::<Test> { assets_restrictions, endowed }
            .assimilate_storage(&mut storage);

        let _ =
            pallet_balances::GenesisConfig::<Test> { balances }.assimilate_storage(&mut storage);

        let ext = sp_io::TestExternalities::new(storage);
        ext
    }
    pub fn build_default(self) -> sp_io::TestExternalities {
        let btc_assets = btc();
        let assets = vec![(btc_assets.0, btc_assets.1, btc_assets.2, true, true)];
        let mut endowed = BTreeMap::new();
        let endowed_info =
            vec![(ALICE, 100_000), (BOB, 200_000), (CHARLIE, 300_000), (DAVE, 400_000)];
        endowed.insert(btc_assets.0, endowed_info);
        let balances =
            vec![(ALICE, 10_000_000), (BOB, 20_000_000), (CHARLIE, 30_000_000), (DAVE, 40_000_000)];
        self.build(assets, endowed, balances)
    }
}
