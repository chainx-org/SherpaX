pub use migration_mainnet_1_1_0::migrate_to_new_cid_details;
pub use migration_testnet::{
    high_key, low_key, medium_key, migrate_to_new_admin_keys, OldAdminKey,
};

pub mod migration_testnet {
    use crate::*;
    use frame_support::pallet_prelude::{StorageValue, ValueQuery};

    pub struct __OldAdminKey<T>(sp_std::marker::PhantomData<T>);
    impl<T: Config> frame_support::traits::StorageInstance for __OldAdminKey<T> {
        fn pallet_prefix() -> &'static str {
            "ComingId"
        }
        const STORAGE_PREFIX: &'static str = "Key";
    }

    pub type OldAdminKey<T> =
        StorageValue<__OldAdminKey<T>, <T as frame_system::Config>::AccountId, ValueQuery>;

    /// A storage migration that remove old admin key and set new admin keys
    pub fn migrate_to_new_admin_keys<T: Config>(
        high_key: T::AccountId,
        medium_key: T::AccountId,
        low_key: T::AccountId,
    ) -> Weight {
        let mut writes = 0;
        let mut reads = 0;

        let is_exists = OldAdminKey::<T>::try_get().is_ok();
        log::info!("ComingId: old key is exists? {}", is_exists);

        if is_exists {
            log::info!("ComingId: update high key");
            HighKey::<T>::put(high_key);
            log::info!("ComingId: update medium key");
            MediumKey::<T>::put(medium_key);
            log::info!("ComingId: update low key");
            LowKey::<T>::put(low_key);

            log::info!("ComingId: kill old key");
            OldAdminKey::<T>::kill();

            writes += 3;
        }

        reads += 1;

        T::DbWeight::get().writes(writes) + T::DbWeight::get().reads(reads)
    }

    pub fn high_key<AccountId: Decode + Default>() -> AccountId {
        AccountId::decode(
            &mut &[
                252, 78, 161, 70, 191, 31, 25, 188, 123, 130, 140, 25, 190, 31, 125, 118, 76, 85,
                16, 140, 138, 175, 96, 117, 208, 12, 159, 167, 218, 30, 202, 117,
            ][..],
        )
        .unwrap_or_default()
    }

    pub fn medium_key<AccountId: Decode + Default>() -> AccountId {
        AccountId::decode(
            &mut &[
                116, 9, 45, 229, 24, 198, 57, 77, 94, 194, 216, 145, 92, 34, 130, 45, 13, 98, 204,
                105, 156, 232, 217, 23, 124, 56, 232, 18, 163, 237, 53, 101,
            ][..],
        )
        .unwrap_or_default()
    }

    pub fn low_key<AccountId: Decode + Default>() -> AccountId {
        AccountId::decode(
            &mut &[
                244, 18, 253, 40, 226, 131, 86, 145, 4, 122, 73, 216, 54, 8, 193, 146, 73, 113, 27,
                54, 208, 156, 97, 198, 52, 86, 108, 0, 59, 59, 198, 96,
            ][..],
        )
        .unwrap_or_default()
    }
}

pub mod migration_mainnet_1_1_0 {
    use crate::*;
    pub fn migrate_to_new_cid_details<T: Config>() -> Weight {
        use frame_support::migration::storage_key_iter;

        #[derive(Clone, Eq, PartialEq, Encode, Decode)]
        pub struct OldBondData {
            pub bond_type: BondType,
            pub data: Vec<u8>,
        }

        #[derive(Clone, Eq, PartialEq, Encode, Decode)]
        pub struct OldCidDetails<AccountId> {
            pub owner: AccountId,
            pub bonds: Vec<OldBondData>,
        }

        let mut migration_count = 0;

        for (cid, old_cid_details) in storage_key_iter::<
            Cid,
            OldCidDetails<T::AccountId>,
            Blake2_128Concat,
        >(b"ComingId", b"Distributed")
        .drain()
        {
            let new_bonds = old_cid_details
                .bonds
                .into_iter()
                .map(|bond_data| BondData {
                    bond_type: bond_data.bond_type,
                    data: bond_data.data.into(),
                })
                .collect();
            let new_cid_details = CidDetails {
                owner: old_cid_details.owner,
                bonds: new_bonds,
                card: Vec::new().into(),
            };

            Distributed::<T>::insert(cid, new_cid_details);

            migration_count += 1;
        }

        log::info!("ComingId: migrate_to_new_cid_details {}", migration_count);

        if migration_count > 0 {
            T::BlockWeights::get().max_block
        } else {
            0
        }
    }
}
