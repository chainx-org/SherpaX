// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;

use sherpax_primitives::ReferralId;

use light_bitcoin::keys::Address;

/// (hot trustee address, cold trustee address)
pub type TrusteePair = (Address, Address);

/// The bitcoin transaction type.
#[doc(hidden)]
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DogeTxType {
    Withdrawal,
    Deposit,
    HotAndCold,
    TrusteeTransition,
    Irrelevance,
}

impl Default for DogeTxType {
    fn default() -> Self {
        DogeTxType::Irrelevance
    }
}

/// The transaction type with deposit info.
#[doc(hidden)]
#[derive(PartialEq, Eq, Clone, RuntimeDebug, TypeInfo)]
pub enum DogeTxMetaType<AccountId> {
    Withdrawal,
    Deposit(DogeDepositInfo<AccountId>),
    HotAndCold,
    TrusteeTransition,
    Irrelevance,
}

impl<AccountId> DogeTxMetaType<AccountId> {
    /// Convert the MetaTxType as DogeTxType.
    pub fn ref_into(&self) -> DogeTxType {
        match self {
            DogeTxMetaType::Withdrawal => DogeTxType::Withdrawal,
            DogeTxMetaType::Deposit(_) => DogeTxType::Deposit,
            DogeTxMetaType::HotAndCold => DogeTxType::HotAndCold,
            DogeTxMetaType::TrusteeTransition => DogeTxType::TrusteeTransition,
            DogeTxMetaType::Irrelevance => DogeTxType::Irrelevance,
        }
    }
}

/// The info of deposit transaction.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, TypeInfo)]
pub struct DogeDepositInfo<AccountId> {
    /// The deposit value.
    pub deposit_value: u64,
    /// The parsed op_return data.
    pub op_return: Option<(AccountId, Option<ReferralId>)>,
    /// The input address of deposit transaction.
    pub input_addr: Option<Address>,
}
