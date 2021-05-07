use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

use chainx_primitives::AddrStr;
use xp_assets_registrar::Chain;

#[derive(Encode, Decode, Default, Clone, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct ChainAddress {
    pub chain: Chain,
    pub address: AddrStr,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Vault<Balance> {
    /// Number of tokens pending issue
    pub to_be_issued_tokens: Balance,
    /// Number of tokens pending redeem
    pub to_be_redeemed_tokens: Balance,
    /// Outer chain address of this Vault (P2PKH, P2SH, P2PKH, P2WSH)
    pub wallet: ChainAddress,
}

impl<Balance: Default> Vault<Balance> {
    pub(crate) fn new(address: ChainAddress) -> Self {
        Self { wallet: address, ..Default::default() }
    }
}
