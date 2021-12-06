use frame_support::pallet_prelude::*;
use frame_support::traits::StoredMap;
use runtime_common::genesis_config::balances_decode_all;
use sp_runtime::traits::Zero;
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

pub struct OnRuntimeUpgrade<Runtime>(PhantomData<Runtime>);

impl<Runtime> frame_support::traits::OnRuntimeUpgrade for OnRuntimeUpgrade<Runtime>
where
    Runtime: pallet_balances::Config,
{
    fn on_runtime_upgrade() -> u64 {
        let balances_config: Vec<(Runtime::AccountId, Runtime::Balance)> =
            balances_decode_all::<Runtime>()
                .unwrap_or_default()
                .into_iter()
                .flat_map(|s| s.balances)
                .collect();

        let total = balances_config
            .iter()
            .fold(Zero::zero(), |acc: Runtime::Balance, &(_, n)| acc + n);

        pallet_balances::TotalIssuance::<Runtime>::mutate(|t| *t += total);

        for &(ref who, free) in balances_config.iter() {
            assert!(
                <Runtime as pallet_balances::Config>::AccountStore::mutate(who, |store| {
                    pallet_balances::AccountData {
                        free: store.free + free,
                        reserved: store.reserved,
                        misc_frozen: store.misc_frozen,
                        fee_frozen: store.fee_frozen,
                    }
                })
                .is_ok()
            );
        }

        log::info!(
            target: "runtime::sherpax",
            "✅Update Balances: total_accounts={}, total_balance={:?}✅",
            balances_config.len(),
            total
        );

        Runtime::BlockWeights::get().max_block
    }
}
