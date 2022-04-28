// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use std::convert::TryFrom;

use serde::Deserialize;

use sp_core::sr25519;

use sherpax_primitives::AccountId;
pub use sherpax_runtime::{
    h256_rev, trustees, BtcCompact, BtcHash, BtcHeader, BtcNetwork, BtcParams, BtcTxVerifier,
    Chain, TrusteeInfoConfig,
};

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

// (account_id, about, hot_key, cold_key)
pub type BtcTrusteeParams = (AccountId, Vec<u8>, Vec<u8>, Vec<u8>);

macro_rules! btc_trustee_key {
    ($btc_pubkey:expr) => {{
        trustees::bitcoin::BtcTrusteeType::try_from(
            hex::decode($btc_pubkey).expect("hex decode failed"),
        )
        .expect("btc trustee generation failed")
        .into()
    }};
}

fn btc_trustee_gen(seed: &str, hot_pubkey: &str, cold_pubkey: &str) -> BtcTrusteeParams {
    (
        get_account_id_from_seed::<sr25519::Public>(seed), // Account Id
        seed.as_bytes().to_vec(),                          // Seed Bytes.
        btc_trustee_key!(hot_pubkey),                      // Hot Key
        btc_trustee_key!(cold_pubkey),                     // Cold Key
    )
}

pub fn dev_trustees() -> Vec<(Chain, TrusteeInfoConfig, Vec<BtcTrusteeParams>)> {
    let btc_config = TrusteeInfoConfig {
        min_trustee_count: 3,
        max_trustee_count: 15,
    };

    let btc_trustees = vec![
        btc_trustee_gen(
            "Alice",
            "0283f579dd2380bd31355d066086e1b4d46b518987c1f8a64d4c0101560280eae2",
            "0300849497d4f88ebc3e1bc2583677c5abdbd3b63640b3c5c50cd4628a33a2a2ca",
        ),
        btc_trustee_gen(
            "Bob",
            "027a0868a14bd18e2e45ff3ad960f892df8d0edd1a5685f0a1dc63c7986d4ad55d",
            "032122032ae9656f9a133405ffe02101469a8d62002270a33ceccf0e40dda54d08",
        ),
        btc_trustee_gen(
            "Charlie",
            "02c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565f",
            "02b3cc747f572d33f12870fa6866aebbfd2b992ba606b8dc89b676b3697590ad63",
        ),
    ];

    vec![
        (Chain::Bitcoin, btc_config.clone(), btc_trustees.clone()),
        (Chain::Dogecoin, btc_config, btc_trustees),
    ]
}

#[cfg(feature = "runtime-benchmarks")]
pub fn benchmarks_trustees() -> Vec<(Chain, TrusteeInfoConfig, Vec<BtcTrusteeParams>)> {
    let btc_config = TrusteeInfoConfig {
        min_trustee_count: 3,
        max_trustee_count: 15,
    };

    let btc_trustees = vec![
        btc_trustee_gen(
            "Alice",
            "0283f579dd2380bd31355d066086e1b4d46b518987c1f8a64d4c0101560280eae2",
            "0300849497d4f88ebc3e1bc2583677c5abdbd3b63640b3c5c50cd4628a33a2a2ca",
        ),
        btc_trustee_gen(
            "Bob",
            "027a0868a14bd18e2e45ff3ad960f892df8d0edd1a5685f0a1dc63c7986d4ad55d",
            "032122032ae9656f9a133405ffe02101469a8d62002270a33ceccf0e40dda54d08",
        ),
        btc_trustee_gen(
            "Charlie",
            "02c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565f",
            "02b3cc747f572d33f12870fa6866aebbfd2b992ba606b8dc89b676b3697590ad63",
        ),
    ];

    vec![(Chain::Bitcoin, btc_config, btc_trustees)]
}

pub fn mainnet_trustees() -> Vec<(Chain, TrusteeInfoConfig, Vec<BtcTrusteeParams>)> {
    let btc_config = TrusteeInfoConfig {
        min_trustee_count: 3,
        max_trustee_count: 15,
    };

    let btc_trustees = vec![];

    vec![(Chain::Bitcoin, btc_config, btc_trustees)]
}
