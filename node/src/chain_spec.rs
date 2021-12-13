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
use runtime_common::{AccountId, AuraId, Balance, BlockNumber, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::collections::BTreeMap;

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
                true,
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
                false
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
    load_genesis: bool,
) -> sherpax_runtime::GenesisConfig {
    let (balances, vesting) = if load_genesis {
        let (balances, vesting) = load_genesis_config();
        let other_balances = endowed_accounts
            .iter()
            .cloned()
            .map(|k| (k, SHERPAX_UNITS * 4096))
            .collect();

        (vec![balances, other_balances].concat(), vesting)
    } else {
        let balances = endowed_accounts
            .iter()
            .cloned()
            .map(|k| (k, SHERPAX_UNITS * 4096))
            .collect();

        (balances, Default::default())
    };

    sherpax_runtime::GenesisConfig {
        system: sherpax_runtime::SystemConfig {
            code: sherpax_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: sherpax_runtime::BalancesConfig { balances },
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
        sudo: sherpax_runtime::SudoConfig { key: root_key },
        vesting: sherpax_runtime::VestingConfig { vesting },
        evm: Default::default(),
        ethereum: Default::default(),
        assets: Default::default(),
        assets_bridge: sherpax_runtime::AssetsBridgeConfig { admin_key: None },
    }
}

fn load_genesis_config() -> (Vec<(AccountId, Balance)>, Vec<(AccountId, BlockNumber, BlockNumber, Balance)>) {
    let non_zero_balances = include_bytes!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/genesis_config/balances/non_zero_airdrop_18016_10500000000000000000000000.json"
        )
    ).to_vec();

    let vestings = include_bytes!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/genesis_config/vesting/vesting_airdrop_18015_943318423258727000000000.json"
        )
    ).to_vec();

    let balances_configs: Vec<sherpax_runtime::BalancesConfig> =
        config_from_json_bytes(vec![non_zero_balances]).unwrap();

    let vesting_configs: Vec<sherpax_runtime::VestingConfig> =
        config_from_json_bytes(vec![vestings]).unwrap();

    let mut total_issuance: Balance = 0u128;
    let balances = balances_configs
        .into_iter()
        .flat_map(|bc| bc.balances)
        .fold(BTreeMap::<AccountId, Balance>::new(), |mut acc, (account_id, amount)| {
            if let Some(balance) = acc.get_mut(&account_id) {
                *balance = balance
                    .checked_add(amount)
                    .expect("balance cannot overflow when building genesis");
            } else {
                acc.insert(account_id.clone(), amount);
            }

            total_issuance = total_issuance
                .checked_add(amount)
                .expect("total insurance cannot overflow when building genesis");
            acc
        })
        .into_iter()
        .collect();

    assert_eq!(total_issuance, 10_500_000* SHERPAX_UNITS, "total issuance must be equal to 10_500_000 KSX");

    let vestings: Vec<(AccountId, BlockNumber, BlockNumber, Balance)> = vesting_configs.into_iter().flat_map(|vc| vc.vesting).collect();
    let vesting_liquid = vestings
        .iter()
        .map(|(_,_,_,free)|free)
        .sum::<u128>();

    assert_eq!(vestings.len(), 18015, "total vesting accounts must be equal 18_015.");
    assert_eq!(vesting_liquid, 943318423258727000000000, "total vesting liquid must be equal 943_318.423258727 KSX");

    (balances, vestings)
}

fn config_from_json_bytes<T: DeserializeOwned>(bytes: Vec<Vec<u8>>) -> Result<Vec<T>, String> {
    let mut configs = vec![];

    for raw in bytes {
        let config = serde_json::from_slice(&raw)
            .map_err(|e| format!("Error parsing config file: {}", e))?;

        configs.push(config)
    }

    Ok(configs)
}
