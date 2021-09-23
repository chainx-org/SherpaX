use crate as pallet_deposit;
pub use pallet_deposit::Config;

use frame_support::{parameter_types, traits::GenesisBuild};
use frame_system as system;
use sp_core::H256;
pub use sp_runtime::{
    testing::Header, AccountId32,
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
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
        ComingId: pallet_coming_id::{Pallet, Call, Config<T>, Storage, Event<T>},
        Deposit: pallet_deposit::{Pallet, Call},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
    pub const MaxCardSize: u32 = 1024*1024; // 1 MB
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
    type AccountId = AccountId32;
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
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = u128;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

impl pallet_coming_id::Config for Test {
    type Event = Event;
    type WeightInfo = ();
    type MaxCardSize = MaxCardSize;
}

impl pallet_deposit::Config for Test {
    type Currency = Balances;
    type ComingNFT = ComingId;
    type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
}

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

// Build test environment by setting the admin `key` for the Genesis.
pub fn new_test_ext(
    admin_key: <Test as frame_system::Config>::AccountId,
) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 10_000_000_000)
        ],
    }
        .assimilate_storage(&mut t)
        .unwrap();

    pallet_coming_id::GenesisConfig::<Test> {
        high_admin_key: admin_key.clone(),
        medium_admin_key: admin_key.clone(),
        low_admin_key: admin_key,
    }
        .assimilate_storage(&mut t)
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
