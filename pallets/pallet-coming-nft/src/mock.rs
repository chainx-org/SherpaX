use crate as pallet_coming_nft;
use frame_support::{parameter_types, traits::GenesisBuild};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

pub use pallet_coming_id::{BondData, CidDetails, Error, Event as ComingIdEvent};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        ComingId: pallet_coming_id::{Pallet, Call, Config<T>, Storage, Event<T>},
        ComingNFT: pallet_coming_nft::{Pallet, Call},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
    pub const MaxCardSize: u32 = 1024 * 1024;
}

impl system::Config for Test {
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
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

impl pallet_coming_id::Config for Test {
    type Event = Event;
    type WeightInfo = ();
    type MaxCardSize = MaxCardSize;
}

impl pallet_coming_nft::Config for Test {
    type ComingNFT = ComingId;
    type WeightInfo = ();
}

// Build test environment by setting the admin `key` for the Genesis.
pub fn new_test_ext(
    admin_key: <Test as frame_system::Config>::AccountId,
) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_coming_id::GenesisConfig::<Test> {
        high_admin_key: admin_key,
        medium_admin_key: admin_key,
        low_admin_key: admin_key,
    }
    .assimilate_storage(&mut t)
    .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub(crate) fn last_event() -> Event {
    system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

pub(crate) fn expect_event<E: Into<Event>>(e: E) {
    assert_eq!(last_event(), e.into());
}
