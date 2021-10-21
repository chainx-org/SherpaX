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

const SHERPAX_ED: SherpaXBalance = basic_runtime::constants::currency::EXISTENTIAL_DEPOSIT;
const SHERPAX_UNITS: SherpaXBalance = basic_runtime::constants::currency::UNITS;

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
pub fn sherpax_basic_session_keys(keys: AuraId) -> basic_runtime::SessionKeys {
    basic_runtime::SessionKeys { aura: keys }
}

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type BasicChainSpec = sc_service::GenericChainSpec<basic_runtime::GenesisConfig, Extensions>;

pub fn basic_dev_config(id: ParaId) -> BasicChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());

    BasicChainSpec::from_genesis(
        // Name
        "SherpaX",
        // ID
        "sherpax-basic",
        ChainType::Development,
        move || {
            basic_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    )
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

pub fn basic_config(id: ParaId) -> BasicChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());

    BasicChainSpec::from_genesis(
        // Name
        "SherpaX",
        // ID
        "sherpax-basic",
        ChainType::Live,
        move || {
            basic_genesis(
                hex!("2a077c909d0c5dcb3748cc11df2fb406ab8f35901b1a93010b78353e4a2bde0d").into(),
                vec![
                    (
                        hex!("e852253a693b64215c58209a1698114c3d73ae41d1c60d67a6aa55ca63f9ce13").into(),
                        hex!("e852253a693b64215c58209a1698114c3d73ae41d1c60d67a6aa55ca63f9ce13").unchecked_into(),
                    )
                ],
                vec![
                    hex!("2a077c909d0c5dcb3748cc11df2fb406ab8f35901b1a93010b78353e4a2bde0d").into(),
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

fn basic_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> basic_runtime::GenesisConfig {
    basic_runtime::GenesisConfig {
        system: basic_runtime::SystemConfig {
            code: basic_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: basic_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, SHERPAX_UNITS * 4096))
                .collect(),
        },
        parachain_info: basic_runtime::ParachainInfoConfig { parachain_id: id },
        collator_selection: basic_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: SHERPAX_ED * 16,
            ..Default::default()
        },
        session: basic_runtime::SessionConfig {
            keys: invulnerables.iter().cloned().map(|(acc, aura)| (
                acc.clone(), // account id
                acc, // validator id
                sherpax_basic_session_keys(aura), // session keys
            )).collect()
        },
        aura: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        sudo: basic_runtime::SudoConfig { key: root_key },
    }
}
