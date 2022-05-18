// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode, Error as CodecError};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;
use sp_std::{convert::TryFrom, fmt, prelude::Vec};

use super::{TrusteeMultisigProvider, TrusteeSessionManager};
use crate::{
    traits::ChainProvider,
    types::{TrusteeIntentionProps, TrusteeSessionInfo},
};
use xp_assets_registrar::Chain;

pub type DogeAddress = Vec<u8>;
pub type DogeTrusteeSessionInfo<AccountId, BlockNumber> =
    TrusteeSessionInfo<AccountId, BlockNumber, DogeTrusteeAddrInfo>;
pub type DogeTrusteeIntentionProps<AccountId> = TrusteeIntentionProps<AccountId, DogeTrusteeType>;
pub type DogeTrusteeSessionManager<T> = TrusteeSessionManager<T, DogeTrusteeAddrInfo>;
pub type DogeTrusteeMultisig<T> = TrusteeMultisigProvider<T, DogeTrusteeType>;

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct DogeTrusteeAddrInfo {
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_text"))]
    pub addr: DogeAddress,
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_hex"))]
    pub redeem_script: Vec<u8>,
}

impl fmt::Debug for DogeTrusteeAddrInfo {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redeem_script_in_hex = hex::encode(&self.redeem_script);
        if redeem_script_in_hex.len() > 16 {
            write!(
                f,
                "DogeTrusteeAddrInfo {{ addr: {}, redeem_script: 0x{}...{} }}",
                String::from_utf8_lossy(&self.addr),
                &redeem_script_in_hex[..8],
                &redeem_script_in_hex[redeem_script_in_hex.len() - 8..]
            )
        } else {
            write!(
                f,
                "DogeTrusteeAddrInfo {{ addr: {}, redeem_script: 0x{} }}",
                String::from_utf8_lossy(&self.addr),
                redeem_script_in_hex,
            )
        }
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DogeTrusteeAddrInfo {{ addr: {:?}, redeem_script: {:?} }}",
            self.addr, self.redeem_script
        )
    }
}

impl From<DogeTrusteeAddrInfo> for Vec<u8> {
    fn from(value: DogeTrusteeAddrInfo) -> Self {
        value.encode()
    }
}

impl TryFrom<Vec<u8>> for DogeTrusteeAddrInfo {
    type Error = CodecError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Decode::decode(&mut &value[..])
    }
}

impl ChainProvider for DogeTrusteeAddrInfo {
    fn chain() -> Chain {
        Chain::Dogecoin
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DogeTrusteeType(pub light_bitcoin::keys::Public);

impl From<DogeTrusteeType> for Vec<u8> {
    fn from(value: DogeTrusteeType) -> Self {
        value.0.to_vec()
    }
}

impl TryFrom<Vec<u8>> for DogeTrusteeType {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        light_bitcoin::keys::Public::from_slice(&value)
            .map(DogeTrusteeType)
            .map_err(|_| ())
    }
}

impl ChainProvider for DogeTrusteeType {
    fn chain() -> Chain {
        Chain::Dogecoin
    }
}

#[test]
fn test_serde_btc_trustee_type() {
    let pubkey = DogeTrusteeType(light_bitcoin::keys::Public::Compressed(Default::default()));
    let ser = serde_json::to_string(&pubkey).unwrap();
    assert_eq!(
        ser,
        "\"0x000000000000000000000000000000000000000000000000000000000000000000\""
    );
    let de = serde_json::from_str::<DogeTrusteeType>(&ser).unwrap();
    assert_eq!(de, pubkey);
}
