// Copyright 2020-2021 ChainX
// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use runtime_common::{AccountId, AuraId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

use crate::bitcoin::{
    btc_genesis_params, BtcGenesisParams, BtcParams, BtcTrusteeParams, BtcTxVerifier, Chain,
    TrusteeInfoConfig,
};
use sherpax_runtime::DAYS;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

use runtime_common::Balance as SherpaXBalance;

const SHERPAX_ED: SherpaXBalance = sherpax_runtime::constants::currency::EXISTENTIAL_DEPOSIT;
const SHERPAX_UNITS: SherpaXBalance = sherpax_runtime::constants::currency::UNITS;

/// Helper function to generate a crypto pair from seed
pub fn get_pair_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
    get_pair_from_seed::<AuraId>(seed)
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn sherpax_session_keys(keys: AuraId) -> sherpax_runtime::SessionKeys {
    sherpax_runtime::SessionKeys { aura: keys }
}

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<sherpax_runtime::GenesisConfig, Extensions>;

#[cfg(feature = "runtime-benchmarks")]
pub fn benchmarks_config(id: ParaId) -> Result<ChainSpec, String> {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert(
        "ss58Format".into(),
        sherpax_runtime::SS58Prefix::get().into(),
    );
    Ok(ChainSpec::from_genesis(
        "Benchmarks",
        "sherpax",
        ChainType::Development,
        move || {
            sherpax_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![(
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_collator_keys_from_seed("Alice"),
                )],
                vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
                id,
                btc_genesis_params(include_str!("../res/btc_genesis_params_testnet.json")),
                crate::bitcoin::local_testnet_trustees(),
            )
        },
        vec![],
        None,
        None,
        Some(properties),
        Extensions {
            relay_chain: "kusama".into(),
            para_id: id.into(),
        },
    ))
}

pub fn dev_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert(
        "ss58Format".into(),
        sherpax_runtime::SS58Prefix::get().into(),
    );

    ChainSpec::from_genesis(
        // Name
        "SherpaX",
        // ID
        "sherpax",
        ChainType::Development,
        move || {
            sherpax_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![(
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_collator_keys_from_seed("Alice"),
                )],
                vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
                id,
                btc_genesis_params(include_str!("../res/btc_genesis_params_testnet.json")),
                crate::bitcoin::local_testnet_trustees(),
            )
        },
        vec![],
        None,
        None,
        Some(properties),
        Extensions {
            relay_chain: "kusama".into(),
            para_id: id.into(),
        },
    )
}

pub fn sherpax_staging_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());

    ChainSpec::from_genesis(
        // Name
        "SherpaX",
        // ID
        "sherpax",
        ChainType::Live,
        move || {
            sherpax_genesis(
                hex!("2a077c909d0c5dcb3748cc11df2fb406ab8f35901b1a93010b78353e4a2bde0d").into(),
                vec![(
                    hex!("e852253a693b64215c58209a1698114c3d73ae41d1c60d67a6aa55ca63f9ce13").into(),
                    hex!("e852253a693b64215c58209a1698114c3d73ae41d1c60d67a6aa55ca63f9ce13")
                        .unchecked_into(),
                )],
                vec![
                    hex!("2a077c909d0c5dcb3748cc11df2fb406ab8f35901b1a93010b78353e4a2bde0d").into(),
                ],
                id,
                btc_genesis_params(include_str!("../res/btc_genesis_params_testnet.json")),
                crate::bitcoin::local_testnet_trustees(),
            )
        },
        vec![],
        None,
        None,
        Some(properties),
        Extensions {
            relay_chain: "kusama".into(),
            para_id: id.into(),
        },
    )
}

pub fn live_mainnet_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/sherpax-raw.json")[..])
}

fn sherpax_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
    bitcoin: BtcGenesisParams,
    trustees: Vec<(Chain, TrusteeInfoConfig, Vec<BtcTrusteeParams>)>,
) -> sherpax_runtime::GenesisConfig {
    let btc_genesis_trustees = trustees
        .iter()
        .find_map(|(chain, _, trustee_params)| {
            if *chain == Chain::Bitcoin {
                Some(
                    trustee_params
                        .iter()
                        .map(|i| (i.0).clone())
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .expect("bitcoin trustees generation can not fail; qed");

    sherpax_runtime::GenesisConfig {
        system: sherpax_runtime::SystemConfig {
            code: sherpax_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: sherpax_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, SHERPAX_UNITS * 4096))
                .collect(),
        },
        parachain_info: sherpax_runtime::ParachainInfoConfig { parachain_id: id },
        collator_selection: sherpax_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: SHERPAX_ED * 16,
            ..Default::default()
        },
        session: sherpax_runtime::SessionConfig {
            keys: invulnerables
                .iter()
                .cloned()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                // account id
                        acc,                        // validator id
                        sherpax_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        aura: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        sudo: sherpax_runtime::SudoConfig {
            key: root_key.clone(),
        },
        council: sherpax_runtime::CouncilConfig::default(),
        elections: sherpax_runtime::ElectionsConfig::default(),
        x_gateway_common: sherpax_runtime::XGatewayCommonConfig {
            trustees,
            genesis_trustee_transition_duration: 30 * DAYS,
            genesis_trustee_transition_status: false,
        },
        x_gateway_bitcoin: sherpax_runtime::XGatewayBitcoinConfig {
            genesis_trustees: btc_genesis_trustees,
            network_id: bitcoin.network,
            confirmation_number: bitcoin.confirmation_number,
            genesis_hash: bitcoin.hash(),
            genesis_info: (bitcoin.header(), bitcoin.height),
            params_info: BtcParams::new(
                // for signet and regtest
                545259519,            // max_bits
                2 * 60 * 60,          // block_max_future
                2 * 7 * 24 * 60 * 60, // target_timespan_seconds
                10 * 60,              // target_spacing_seconds
                4,                    // retargeting_factor
            ), // retargeting_factor
            btc_withdrawal_fee: 500000,
            max_withdrawal_count: 100,
            verifier: BtcTxVerifier::Recover,
        },
        x_gateway_records: sherpax_runtime::XGatewayRecordsConfig {
            initial_asset_chain: vec![(root_key, 1, Chain::Bitcoin)],
        },
    }
}
