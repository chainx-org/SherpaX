// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.into().

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_std::vec;
use xpallet_assets::Pallet as XAssets;

use crate::Pallet as Swap;

const PCX: u32 = 0;
const X_BTC: u32 = 1;

benchmarks! {

    create_pair {
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller.clone()), PCX, X_BTC)
    verify {
        assert!(Swap::<T>::swap_metadata((PCX, X_BTC)).is_some());
    }

    add_liquidity {
        let caller = whitelisted_caller();
        <T as xpallet_assets::Config>::Currency::make_free_balance_be(&caller, 1_000_000_000u32.into());
        assert_ok!(XAssets::<T>::issue(&X_BTC, &caller, 1_000_000_000u32.into()));
        assert_ok!(Swap::<T>::create_pair(RawOrigin::Signed(caller.clone()).into(), PCX, X_BTC));
    }: _(RawOrigin::Signed(caller.clone()), PCX, X_BTC, 50_000_000u32.into(), 40_000_000u32.into(), 0u32.into(), 0u32.into(), 100u32.into())
    verify {
        assert_eq!(Swap::<T>::swap_metadata((PCX, X_BTC)).map(|(_, total_liquidity_)| total_liquidity_).unwrap(), 44721359u32.into());
    }

    remove_liquidity {
        let caller = whitelisted_caller();
        <T as xpallet_assets::Config>::Currency::make_free_balance_be(&caller, 1_000_000_000u32.into());
        assert_ok!(XAssets::<T>::issue(&X_BTC, &caller, 1_000_000_000u32.into()));
        assert_ok!(Swap::<T>::create_pair(RawOrigin::Signed(caller.clone()).into(), PCX, X_BTC));
        assert_ok!(Swap::<T>::add_liquidity(RawOrigin::Signed(caller.clone()).into(), PCX, X_BTC,
            100_000_000u32.into(), 80_000_000u32.into(), 0u32.into(), 0u32.into(), 100u32.into()));
    }: _(RawOrigin::Signed(caller.clone()), PCX, X_BTC, 30_000_000u32.into(), 0u32.into(), 0u32.into(), T::Lookup::unlookup(caller.clone()), 100u32.into())
    verify {
        assert_eq!(Swap::<T>::swap_metadata((PCX, X_BTC)).map(|(_, total_liquidity_)| total_liquidity_).unwrap(), 59442719u32.into());
    }

    swap_exact_tokens_for_tokens {
        let caller = whitelisted_caller();
        <T as xpallet_assets::Config>::Currency::make_free_balance_be(&caller, 1_000_000_000u32.into());
        assert_ok!(XAssets::<T>::issue(&X_BTC, &caller, 1_000_000_000u32.into()));
        assert_ok!(Swap::<T>::create_pair(RawOrigin::Signed(caller.clone()).into(), PCX, X_BTC));
        assert_ok!(Swap::<T>::add_liquidity(RawOrigin::Signed(caller.clone()).into(), PCX, X_BTC,
            50_000_000u32.into(), 40_000_000u32.into(), 0u32.into(), 0u32.into(), 100u32.into()));
    }: _(RawOrigin::Signed(caller.clone()), 100_000_000u32.into(), 5_000_000u32.into(), vec![PCX, X_BTC], T::Lookup::unlookup(caller.clone()), 100u32.into())
    verify {
        let pair_account = Swap::<T>::swap_metadata((PCX, X_BTC)).map(|(pair_account_, _)| pair_account_).unwrap();
        let reserve_0 = T::MultiAsset::balance_of(PCX, &pair_account);
        let reserve_1 = T::MultiAsset::balance_of(X_BTC, &pair_account);
        assert_eq!(reserve_0, 150000000u32.into());
        assert_eq!(reserve_1, 13360054u32.into());
    }

    swap_tokens_for_exact_tokens {
        let caller = whitelisted_caller();
        <T as xpallet_assets::Config>::Currency::make_free_balance_be(&caller, 1_000_000_000u32.into());
        assert_ok!(XAssets::<T>::issue(&X_BTC, &caller, 1_000_000_000u32.into()));
        assert_ok!(Swap::<T>::create_pair(RawOrigin::Signed(caller.clone()).into(), PCX, X_BTC));
        assert_ok!(Swap::<T>::add_liquidity(RawOrigin::Signed(caller.clone()).into(), PCX, X_BTC, 50_000_000u32.into(),
            40_000_000u32.into(), 0u32.into(), 0u32.into(), 100u32.into()));
    }: _(RawOrigin::Signed(caller.clone()),20_000_000u32.into(), 100_000_000u32.into(), vec![PCX, X_BTC], T::Lookup::unlookup(caller.clone()),  100u32.into())
    verify {
        let pair_account = Swap::<T>::swap_metadata((PCX, X_BTC)).map(|(pair_account_, _)| pair_account_).unwrap();
        let reserve_0 = T::MultiAsset::balance_of(PCX, &pair_account);
        let reserve_1 = T::MultiAsset::balance_of(X_BTC, &pair_account);
        assert_eq!(reserve_0, 100_150_451u32.into());
        assert_eq!(reserve_1, 20_000_000u32.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{ExtBuilder, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        ExtBuilder::default().build_default().execute_with(|| {
            assert_ok!(test_benchmark_create_pair::<Test>());
            assert_ok!(test_benchmark_add_liquidity::<Test>());
            assert_ok!(test_benchmark_remove_liquidity::<Test>());
            assert_ok!(test_benchmark_swap_exact_tokens_for_tokens::<Test>());
            assert_ok!(test_benchmark_swap_tokens_for_exact_tokens::<Test>());
        });
    }
}
