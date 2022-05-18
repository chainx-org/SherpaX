// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use std::convert::TryFrom;

use serde::Deserialize;

use sp_core::sr25519;

use sherpax_primitives::AccountId;
pub use sherpax_runtime::{
    h256_rev, trustees, BtcCompact, BtcHash, BtcHeader, BtcNetwork, BtcParams, Chain,
    TrusteeInfoConfig,
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
            "0483f579dd2380bd31355d066086e1b4d46b518987c1f8a64d4c0101560280eae2b16f3068e94333e11ee63770936eca9692a25f76012511d38ac30ece20f07dca",
            "0400849497d4f88ebc3e1bc2583677c5abdbd3b63640b3c5c50cd4628a33a2a2cab6b69094b5a213da80f9ef730fab39de770ca124f2d9a9cb161856be54b9adc5",
        ),
        btc_trustee_gen(
            "Bob",
            "047a0868a14bd18e2e45ff3ad960f892df8d0edd1a5685f0a1dc63c7986d4ad55d47c09531e4f2ca2ae7f9ed80c1f9df2edd8afa19188692724d2bc18c18d98c10",
            "042122032ae9656f9a133405ffe02101469a8d62002270a33ceccf0e40dda54d08c989b55f1b6b46a8dee284cf6737de0a377e410bcfd361a015528ae80a349529",
        ),
        btc_trustee_gen(
            "Charlie",
            "04c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565faced14acb5172ee19aee5417488fecdda33f4cfea9ff04f250e763e6f7458d5e",
            "04b3cc747f572d33f12870fa6866aebbfd2b992ba606b8dc89b676b3697590ad63d5ca398bdb6f8ee619f2e16997f21e5e8f0e0b00e2f275c7cb1253f381058d56",
        ),
    ];

    let doge_trustees = vec![
        btc_trustee_gen(
            "Alice",
            "042f7e2f0f3e912bf416234913b388393beb5092418fea986e45c0b9633adefd85168f3b1d13ae29651c29e424760b3795fc78152ac119e0dc4e2b9055329099b3",
            "0400849497d4f88ebc3e1bc2583677c5abdbd3b63640b3c5c50cd4628a33a2a2cab6b69094b5a213da80f9ef730fab39de770ca124f2d9a9cb161856be54b9adc5",
        ),
        btc_trustee_gen(
            "Bob",
            "0451e0dc3d9709d860c49785fc84b62909d991cffd81592f6994c452438f91b6a2e586541c4b3bc1ebeb5fb9fad2ed2e696b2175c54458ab6f103717cbeeb4e52c",
            "042122032ae9656f9a133405ffe02101469a8d62002270a33ceccf0e40dda54d08c989b55f1b6b46a8dee284cf6737de0a377e410bcfd361a015528ae80a349529",
        ),
        btc_trustee_gen(
            "Charlie",
            "04a09e8182977710bab64472c0ecaf9e52255a890554a00a62facd05c0b13817f8995bf590851c19914bfc939d53365b90cc2f0fcfddaca184f0c1e7ce1736f0b8",
            "04b3cc747f572d33f12870fa6866aebbfd2b992ba606b8dc89b676b3697590ad63d5ca398bdb6f8ee619f2e16997f21e5e8f0e0b00e2f275c7cb1253f381058d56",
        ),
    ];

    vec![
        (Chain::Bitcoin, btc_config.clone(), btc_trustees),
        (Chain::Dogecoin, btc_config, doge_trustees),
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
            "0483f579dd2380bd31355d066086e1b4d46b518987c1f8a64d4c0101560280eae2b16f3068e94333e11ee63770936eca9692a25f76012511d38ac30ece20f07dca",
            "0400849497d4f88ebc3e1bc2583677c5abdbd3b63640b3c5c50cd4628a33a2a2cab6b69094b5a213da80f9ef730fab39de770ca124f2d9a9cb161856be54b9adc5",
        ),
        btc_trustee_gen(
            "Bob",
            "047a0868a14bd18e2e45ff3ad960f892df8d0edd1a5685f0a1dc63c7986d4ad55d47c09531e4f2ca2ae7f9ed80c1f9df2edd8afa19188692724d2bc18c18d98c10",
            "042122032ae9656f9a133405ffe02101469a8d62002270a33ceccf0e40dda54d08c989b55f1b6b46a8dee284cf6737de0a377e410bcfd361a015528ae80a349529",
        ),
        btc_trustee_gen(
            "Charlie",
            "04c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565faced14acb5172ee19aee5417488fecdda33f4cfea9ff04f250e763e6f7458d5e",
            "04b3cc747f572d33f12870fa6866aebbfd2b992ba606b8dc89b676b3697590ad63d5ca398bdb6f8ee619f2e16997f21e5e8f0e0b00e2f275c7cb1253f381058d56",
        ),
    ];

    let doge_trustees = vec![
        btc_trustee_gen(
            "Alice",
            "042f7e2f0f3e912bf416234913b388393beb5092418fea986e45c0b9633adefd85168f3b1d13ae29651c29e424760b3795fc78152ac119e0dc4e2b9055329099b3",
            "0400849497d4f88ebc3e1bc2583677c5abdbd3b63640b3c5c50cd4628a33a2a2cab6b69094b5a213da80f9ef730fab39de770ca124f2d9a9cb161856be54b9adc5",
        ),
        btc_trustee_gen(
            "Bob",
            "0451e0dc3d9709d860c49785fc84b62909d991cffd81592f6994c452438f91b6a2e586541c4b3bc1ebeb5fb9fad2ed2e696b2175c54458ab6f103717cbeeb4e52c",
            "042122032ae9656f9a133405ffe02101469a8d62002270a33ceccf0e40dda54d08c989b55f1b6b46a8dee284cf6737de0a377e410bcfd361a015528ae80a349529",
        ),
        btc_trustee_gen(
            "Charlie",
            "04a09e8182977710bab64472c0ecaf9e52255a890554a00a62facd05c0b13817f8995bf590851c19914bfc939d53365b90cc2f0fcfddaca184f0c1e7ce1736f0b8",
            "04b3cc747f572d33f12870fa6866aebbfd2b992ba606b8dc89b676b3697590ad63d5ca398bdb6f8ee619f2e16997f21e5e8f0e0b00e2f275c7cb1253f381058d56",
        ),
    ];

    vec![
        (Chain::Bitcoin, btc_config.clone(), btc_trustees),
        (Chain::Dogecoin, btc_config, doge_trustees),
    ]
}

pub fn mainnet_trustees() -> Vec<(Chain, TrusteeInfoConfig, Vec<BtcTrusteeParams>)> {
    let btc_config = TrusteeInfoConfig {
        min_trustee_count: 3,
        max_trustee_count: 15,
    };

    let btc_trustees = vec![];

    vec![(Chain::Bitcoin, btc_config, btc_trustees)]
}
