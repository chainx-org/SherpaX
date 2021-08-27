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
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public, H160, U256};
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
                vec![
                    // Alith
                    H160::from(hex_literal::hex!["f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"]),
                    // Baltathar
                    H160::from(hex_literal::hex!["3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"]),
                    // Charleth
                    H160::from(hex_literal::hex!["798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc"]),
                    // Dorothy
                    H160::from(hex_literal::hex!["773539d4Ac0e786233D90A233654ccEE26a613D9"]),
                    // Ethan
                    H160::from(hex_literal::hex!["Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB"]),
                    // Faith
                    H160::from(hex_literal::hex!["C0F0f4ab324C46e55D02D0033343B4Be8A55532d"]),
                    // Goliath
                    H160::from(hex_literal::hex!["7BF369283338E12C90514468aa3868A551AB2929"]),
                    // Heath
                    H160::from(hex_literal::hex!["931f3600a299fd9B24cEfB3BfF79388D19804BeA"]),
                    // Ida
                    H160::from(hex_literal::hex!["C41C5F1123ECCd5ce233578B2e7ebd5693869d73"]),
                    // Judith
                    H160::from(hex_literal::hex!["2898FE7a42Be376C8BC7AF536A940F7Fd5aDd423"]),
                    // Gerald
                    H160::from(hex_literal::hex!["6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"]),
                ],
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
                vec![
                    // Alith
                    H160::from(hex_literal::hex!["f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"]),
                    // Baltathar
                    H160::from(hex_literal::hex!["3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"]),
                    // Charleth
                    H160::from(hex_literal::hex!["798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc"]),
                    // Dorothy
                    H160::from(hex_literal::hex!["773539d4Ac0e786233D90A233654ccEE26a613D9"]),
                    // Ethan
                    H160::from(hex_literal::hex!["Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB"]),
                    // Faith
                    H160::from(hex_literal::hex!["C0F0f4ab324C46e55D02D0033343B4Be8A55532d"]),
                    // Goliath
                    H160::from(hex_literal::hex!["7BF369283338E12C90514468aa3868A551AB2929"]),
                    // Heath
                    H160::from(hex_literal::hex!["931f3600a299fd9B24cEfB3BfF79388D19804BeA"]),
                    // Ida
                    H160::from(hex_literal::hex!["C41C5F1123ECCd5ce233578B2e7ebd5693869d73"]),
                    // Judith
                    H160::from(hex_literal::hex!["2898FE7a42Be376C8BC7AF536A940F7Fd5aDd423"]),
                    // Gerald
                    H160::from(hex_literal::hex!["6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"]),
                ],
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
                // TODO: initial 4 collators.
                vec![(
                         hex!("2a077c909d0c5dcb3748cc11df2fb406ab8f35901b1a93010b78353e4a2bde0d").into(),
                         hex!("2a077c909d0c5dcb3748cc11df2fb406ab8f35901b1a93010b78353e4a2bde0d").unchecked_into()
                     )
                ],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                ],
                id,
                vec![]
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
    addresses: Vec<H160>,
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
        sudo: sherpax_runtime::SudoConfig { key: root_key.clone() },
        coming_id: sherpax_runtime::ComingIdConfig {
            // Assign network admin rights.
            high_admin_key: root_key.clone(),
            medium_admin_key: root_key.clone(),
            low_admin_key: root_key,
        },
        ethereum_chain_id: sherpax_runtime::EthereumChainIdConfig { chain_id: 1500u64 },
        evm: sherpax_runtime::EVMConfig {
            accounts: addresses
                .into_iter()
                .map(|addr| {
                    (
                        addr,
                        sherpax_runtime::GenesisAccount {
                            balance: U256::from(1_000_000_000_000_000_000_000u128),
                            nonce: Default::default(),
                            code: Default::default(),
                            storage: Default::default(),
                        },
                    )
                })
                .collect()
        },
        ethereum: sherpax_runtime::EthereumConfig {},
    }
}
