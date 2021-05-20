// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use std::convert::TryFrom;

use serde::Deserialize;

use sp_core::sr25519;

use chainx_primitives::AccountId;
use light_bitcoin::script::h256_rev;
use xpallet_assets_registrar::Chain;
use xpallet_gateway_bitcoin::{
    hash_rev, BtcHeader, BtcNetwork, BtcParams, BtcTxVerifier, Compact as BtcCompact,
    H256 as BtcHash,
};
use xpallet_gateway_common::types::TrusteeInfoConfig;

use crate::chain_spec::get_account_id_from_seed;

#[derive(Debug, Deserialize)]
pub struct BtcGenesisParams {
    pub network: BtcNetwork,
    pub confirmation_number: u32,
    pub height: u32,
    hash: String,
    version: u32,
    previous_header_hash: String,
    merkle_root_hash: String,
    time: u32,
    bits: BtcCompact,
    nonce: u32,
}

impl BtcGenesisParams {
    /// Return the block hash.
    ///
    /// Indicating user-visible serializations of this hash should be backward.
    pub fn hash(&self) -> BtcHash {
        h256_rev(&self.hash)
    }

    /// Return the block header.
    ///
    /// Indicating user-visible serializations of `previous_header_hash` and `merkle_root_hash`
    /// should be backward.
    pub fn header(&self) -> BtcHeader {
        BtcHeader {
            version: self.version,
            previous_header_hash: h256_rev(&self.previous_header_hash),
            merkle_root_hash: h256_rev(&self.merkle_root_hash),
            time: self.time,
            bits: self.bits,
            nonce: self.nonce,
        }
    }
}

pub fn btc_genesis_params(res: &str) -> BtcGenesisParams {
    let params: BtcGenesisParams = serde_json::from_str(res).expect("JSON was not well-formatted");
    assert_eq!(params.header().hash(), params.hash());
    params
}
