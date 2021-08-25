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
use rococo_parachain_runtime::{AccountId, AuraId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<rococo_parachain_runtime::GenesisConfig, Extensions>;

/// Specialized `ChainSpec` for the shell parachain runtime.
pub type ShellChainSpec = sc_service::GenericChainSpec<shell_runtime::GenesisConfig, Extensions>;

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
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_chain_spec(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_from_seed::<AuraId>("Alice"),
                    get_from_seed::<AuraId>("Bob"),
                ],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                id,
            )
        },
        vec![],
        None,
        None,
        None,
        Extensions {
            relay_chain: "westend".into(),
            para_id: id.into(),
        },
    )
}

pub fn get_shell_chain_spec(id: ParaId) -> ShellChainSpec {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());

    ShellChainSpec::from_genesis(
        "SherpaX",
        "shell_local_testnet",
        ChainType::Local,
        move || shell_testnet_genesis(
            hex!["2a077c909d0c5dcb3748cc11df2fb406ab8f35901b1a93010b78353e4a2bde0d"].into(),
            vec![
                hex!["2a077c909d0c5dcb3748cc11df2fb406ab8f35901b1a93010b78353e4a2bde0d"].into()
            ],
            id
        ),
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

fn testnet_genesis(
    root_key: AccountId,
    initial_authorities: Vec<AuraId>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> rococo_parachain_runtime::GenesisConfig {
    rococo_parachain_runtime::GenesisConfig {
        system: rococo_parachain_runtime::SystemConfig {
            code: rococo_parachain_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: rococo_parachain_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        sudo: rococo_parachain_runtime::SudoConfig { key: root_key },
        parachain_info: rococo_parachain_runtime::ParachainInfoConfig { parachain_id: id },
        aura: rococo_parachain_runtime::AuraConfig {
            authorities: initial_authorities,
        },
        aura_ext: Default::default(),
        parachain_system: Default::default(),
    }
}

fn shell_testnet_genesis(
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    parachain_id: ParaId
) -> shell_runtime::GenesisConfig {
    const ENDOWMENT: u128 = 21_000_000_000_000;

    shell_runtime::GenesisConfig {
        system: shell_runtime::SystemConfig {
            code: shell_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        parachain_info: shell_runtime::ParachainInfoConfig { parachain_id },
        balances: shell_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, ENDOWMENT))
                .collect(),
        },
        sudo: shell_runtime::SudoConfig { key: root_key },
        parachain_system: Default::default()
    }
}

use runtime_common::Balance as StatemintBalance;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type StatemineChainSpec = sc_service::GenericChainSpec<statemine_runtime::GenesisConfig, Extensions>;

const STATEMINE_ED: StatemintBalance = statemine_runtime::constants::currency::EXISTENTIAL_DEPOSIT;

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
pub fn statemine_session_keys(keys: AuraId) -> statemine_runtime::opaque::SessionKeys {
    statemine_runtime::opaque::SessionKeys { aura: keys }
}

pub fn statemine_development_config(id: ParaId) -> StatemineChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSM".into());
    properties.insert("tokenDecimals".into(), 12.into());

    StatemineChainSpec::from_genesis(
        // Name
        "Statemine Development",
        // ID
        "statemine_dev",
        ChainType::Local,
        move || {
            statemine_genesis(
                // initial collators.
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    )
                ],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                id,
            )
        },
        vec![],
        None,
        None,
        Some(properties),
        Extensions {
            relay_chain: "kusama-dev".into(),
            para_id: id.into(),
        },
    )
}

pub fn statemine_local_config(id: ParaId) -> StatemineChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSM".into());
    properties.insert("tokenDecimals".into(), 12.into());

    StatemineChainSpec::from_genesis(
        // Name
        "Statemine Local",
        // ID
        "statemine_local",
        ChainType::Local,
        move || {
            statemine_genesis(
                // initial collators.
                vec![(
                         get_account_id_from_seed::<sr25519::Public>("Alice"),
                         get_collator_keys_from_seed("Alice")
                     ),
                     (
                         get_account_id_from_seed::<sr25519::Public>("Bob"),
                         get_collator_keys_from_seed("Bob")
                     ),
                ],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                id,
            )
        },
        vec![],
        None,
        None,
        Some(properties),
        Extensions {
            relay_chain: "kusama-local".into(),
            para_id: id.into(),
        },
    )
}

pub fn statemine_config(id: ParaId) -> StatemineChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSM".into());
    properties.insert("tokenDecimals".into(), 12.into());

    StatemineChainSpec::from_genesis(
        // Name
        "Statemine",
        // ID
        "statemine",
        ChainType::Live,
        move || {
            statemine_genesis(
                // initial collators.
                vec![(
                         hex!("50673d59020488a4ffc9d8c6de3062a65977046e6990915617f85fef6d349730").into(),
                         hex!("50673d59020488a4ffc9d8c6de3062a65977046e6990915617f85fef6d349730").unchecked_into()
                     ),
                     (
                         hex!("fe8102dbc244e7ea2babd9f53236d67403b046154370da5c3ea99def0bd0747a").into(),
                         hex!("fe8102dbc244e7ea2babd9f53236d67403b046154370da5c3ea99def0bd0747a").unchecked_into()
                     ),
                     (
                         hex!("38144b5398e5d0da5ec936a3af23f5a96e782f676ab19d45f29075ee92eca76a").into(),
                         hex!("38144b5398e5d0da5ec936a3af23f5a96e782f676ab19d45f29075ee92eca76a").unchecked_into()
                     ),
                     (
                         hex!("3253947640e309120ae70fa458dcacb915e2ddd78f930f52bd3679ec63fc4415").into(),
                         hex!("3253947640e309120ae70fa458dcacb915e2ddd78f930f52bd3679ec63fc4415").unchecked_into()
                     ),
                ],
                vec![],
                id,
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

fn statemine_genesis(
    invulnerables: Vec<(AccountId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> statemine_runtime::GenesisConfig {
    statemine_runtime::GenesisConfig {
        system: statemine_runtime::SystemConfig {
            code: statemine_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: statemine_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, STATEMINE_ED * 4096))
                .collect(),
        },
        parachain_info: statemine_runtime::ParachainInfoConfig { parachain_id: id },
        collator_selection: statemine_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: STATEMINE_ED * 16,
            ..Default::default()
        },
        session: statemine_runtime::SessionConfig {
            keys: invulnerables.iter().cloned().map(|(acc, aura)| (
                acc.clone(), // account id
                acc.clone(), // validator id
                statemine_session_keys(aura), // session keys
            )).collect()
        },
        aura: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
    }
}
