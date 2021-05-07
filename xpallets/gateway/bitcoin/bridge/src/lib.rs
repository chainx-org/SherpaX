//! SherpaX bridge pallet.  
//! Bridge assets from outer bitcoinlike-chain into SherpaX.
//!
//! The bridge use collateral based model.
//!
//! Terminology:
//! - `Collateral`: Reserved Native assets.

mod ext;
mod traits;
mod types;

#[frame_support::pallet]
pub mod pallet {
    use chainx_primitives::{AddrStr, Balance};
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        storage::types::{StorageDoubleMap, StorageMap, ValueQuery},
        traits::IsType,
        Blake2_128Concat, Twox64Concat,
    };
    use frame_system::pallet_prelude::OriginFor;
    use sp_std::marker::PhantomData;

    use frame_support::traits::Hooks;
    use frame_system::pallet_prelude::BlockNumberFor;

    use crate::types::{ChainAddress, Vault};

    #[cfg(feature = "std")]
    use frame_support::traits::GenesisBuild;

    use xp_assets_registrar::Chain;

    #[pallet::pallet]
    #[pallet::generate_store(pub(crate) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register sender as vault with initial collateral and
        #[pallet::weight(0)]
        pub fn register_vault(
            origin: OriginFor<T>,
            collateral: Balance,
            address: ChainAddress,
        ) -> DispatchResultWithPostInfo {
            //TODO(wangyafei):
            // ensure signed
            // ensure address is valid
            // ensure balance enough
            // ensure not registered
            // ensure address is unique
            Ok(().into())
        }

        /// Add collateral for sender.
        ///
        /// Errors:
        /// - when sender is not a vault in specific chain.
        #[pallet::weight(0)]
        pub fn add_extra_collateral(
            origin: OriginFor<T>,
            collateral: Balance,
            chain: Chain,
        ) -> DispatchResultWithPostInfo {
            // TODO(wangyafei):
            //  ensure signed
            //  ensure is vault
            //  ensure balance enough
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T> {
        PlaceHolder(PhantomData<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        PlaceHolder,
    }

    /// Total collateral for each chain.
    #[pallet::storage]
    pub(crate) type TotalCollateral<T: Config> = StorageMap<
        _,
        Twox64Concat,
        Chain,
        Balance, /*FIXME(wangyafei): use BalanceOF<T> when `xpallet-assets` available.*/
        ValueQuery,
    >;

    /// Store all vaults, group by chain and account_id
    #[pallet::storage]
    pub(crate) type Vaults<T: Config> =
        StorageDoubleMap<_, Twox64Concat, Chain, Blake2_128Concat, T::AccountId, Vault<Balance>>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub liquidator_id: T::AccountId,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                // TODO(wangyafei): generate from fixed seed.
                liquidator_id: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {}
    }
}
