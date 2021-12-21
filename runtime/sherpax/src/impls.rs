use frame_support::{
    parameter_types,
    traits::{Currency, OnUnbalanced},
};
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_runtime::{FixedPointNumber, Perquintill};
use sp_staking::SessionIndex;
use sp_std::marker::PhantomData;

use crate::{AccountId, Authorship, Balances, Vec};

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

parameter_types! {
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000u128);
}

pub type SlowAdjustingFeeUpdate<R> =
    TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        Balances::resolve_creating(&Authorship::author(), amount);
    }
}

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(fees) = fees_then_tips.next() {
            Author::on_unbalanced(fees);
        }
    }
}

/// A convertor from aura id. Since this pallet does not have stash/controller, this is
/// just identity.
pub struct IdentityAura;
impl<T> sp_runtime::traits::Convert<T, Option<T>> for IdentityAura {
    fn convert(t: T) -> Option<T> {
        Some(t)
    }
}

pub struct IdentitySession<T>(PhantomData<T>);
impl<T: pallet_session::Config> pallet_session::SessionManager<T::ValidatorId>
    for IdentitySession<T>
{
    fn new_session(_: SessionIndex) -> Option<Vec<T::ValidatorId>> {
        let validators = pallet_session::Pallet::<T>::validators();
        if validators.is_empty() {
            None
        } else {
            Some(validators)
        }
    }
    fn end_session(_: SessionIndex) {}
    fn start_session(_: SessionIndex) {}
}
