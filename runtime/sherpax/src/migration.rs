use sp_runtime::traits::{Convert, Saturating, Zero};
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;
use frame_support::pallet_prelude::*;
use frame_support::traits::{
    Currency, LockableCurrency, LockIdentifier, StoredMap, WithdrawReasons
};
use runtime_common::genesis_config::{
    balances_decode_all, vesting_decode_all
};

pub struct OnRuntimeUpgrade<Runtime>(
    PhantomData<Runtime>
);

impl<Runtime> frame_support::traits::OnRuntimeUpgrade
for OnRuntimeUpgrade<Runtime>
    where
        Runtime: pallet_balances::Config + pallet_vesting::Config,
{
    fn on_runtime_upgrade() -> u64 {
        let balances_config: Vec<(Runtime::AccountId, Runtime::Balance)>
            = balances_decode_all::<Runtime>()
            .unwrap_or_default()
            .into_iter()
            .flat_map(|s| s.balances)
            .collect();

        let total = balances_config
            .iter()
            .fold(Zero::zero(), |acc: Runtime::Balance, &(_, n)| acc + n);

        pallet_balances::TotalIssuance::<Runtime>::mutate(
            |t|{ *t += total }
        );

        for &(ref who, free) in balances_config.iter() {
            assert!(
                <Runtime as pallet_balances::Config>::AccountStore::insert(
                    who,
                    pallet_balances::AccountData { free, ..Default::default() }
                ).is_ok()
            );
        }

        log::info!(
            target: "runtime::sherpax",
            "✅Update Balances: total_accounts={}, total_balance={:?}✅",
            balances_config.len(),
            total
        );

        const VESTING_ID: LockIdentifier = *b"vesting ";
        type BalanceOf<T> = <<T as pallet_vesting::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

        let vesting_config: Vec<(Runtime::AccountId, Runtime::BlockNumber, Runtime::BlockNumber, BalanceOf<Runtime>)>
            = vesting_decode_all::<Runtime>()
            .unwrap_or_default()
            .into_iter()
            .flat_map(|s| s.vesting)
            .collect();

        let total_liquid = vesting_config
            .iter()
            .fold(Zero::zero(), |acc: BalanceOf<Runtime>, &(_, _, _, n)| acc + n);

        // Generate initial vesting configuration
        // * who - Account which we are generating vesting configuration for
        // * begin - Block when the account will start to vest
        // * length - Number of blocks from `begin` until fully vested
        // * liquid - Number of units which can be spent before vesting begins
        for &(ref who, begin, length, liquid) in vesting_config.iter() {
            let balance = <Runtime as pallet_vesting::Config>::Currency::free_balance(who);
            assert!(!balance.is_zero(), "Currencies must be init'd before vesting");
            // Total genesis `balance` minus `liquid` equals funds locked for vesting
            let locked = balance.saturating_sub(liquid);
            let length_as_balance = <Runtime as pallet_vesting::Config>::BlockNumberToBalance::convert(length);
            let per_block = locked / length_as_balance.max(sp_runtime::traits::One::one());
            let vesting_info = pallet_vesting::VestingInfo::<BalanceOf<Runtime>, Runtime::BlockNumber>::new(locked, per_block, begin);
            if !vesting_info.is_valid() {
                panic!("Invalid VestingInfo params at genesis")
            };

            pallet_vesting::Vesting::<Runtime>::try_append(who, vesting_info)
                .expect("Too many vesting schedules at genesis.");

            let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
            <Runtime as pallet_vesting::Config>::Currency::set_lock(VESTING_ID, who, locked, reasons);
        }

        log::info!(
            target: "runtime::sherpax",
            "✅Update Vesting: total_accounts={}, total_liquid={:?}✅",
            vesting_config.len(),
            total_liquid
        );

        Runtime::BlockWeights::get().max_block
    }
}
