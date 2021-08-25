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

use runtime_common::Balance as SherpaXBalance;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type SherpaxChainSpec = sc_service::GenericChainSpec<sherpax_runtime::GenesisConfig, Extensions>;

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
pub fn sherpax_session_keys(keys: AuraId) -> sherpax_runtime::opaque::SessionKeys {
    sherpax_runtime::opaque::SessionKeys { aura: keys }
}

pub fn sherpax_development_config(id: ParaId) -> SherpaxChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());

    SherpaxChainSpec::from_genesis(
        // Name
        "SherpaX Development",
        // ID
        "sherpax_dev",
        ChainType::Local,
        move || {
            sherpax_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
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

pub fn sherpax_local_config(id: ParaId) -> SherpaxChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());

    SherpaxChainSpec::from_genesis(
        // Name
        "SherpaX Local",
        // ID
        "sherpax_local",
        ChainType::Local,
        move || {
            sherpax_genesis(
                // initial collators.
                get_account_id_from_seed::<sr25519::Public>("Alice"),
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

pub fn sherpax_config(id: ParaId) -> SherpaxChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());

    SherpaxChainSpec::from_genesis(
        // Name
        "SherpaX",
        // ID
        "sherpax",
        ChainType::Live,
        move || {
            sherpax_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
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
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                ],
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

fn sherpax_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> sherpax_runtime::GenesisConfig {
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
            keys: invulnerables.iter().cloned().map(|(acc, aura)| (
                acc.clone(), // account id
                acc.clone(), // validator id
                sherpax_session_keys(aura), // session keys
            )).collect()
        },
        aura: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        sudo: sherpax_runtime::SudoConfig { key: root_key },
    }
}
