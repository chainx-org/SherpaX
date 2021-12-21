use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
pub use sherpax_runtime::{
    constants::currency::UNITS, opaque::SessionKeys, AccountId, AssetsBridgeConfig, AssetsConfig,
    AuraConfig, Balance, BalancesConfig, BlockNumber, EthereumConfig, EvmConfig, GenesisAccount,
    GenesisConfig, GrandpaConfig, SessionConfig, Signature, SudoConfig, SystemConfig,
    VestingConfig, WASM_BINARY,
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::collections::BTreeMap;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// The light sync state.
    ///
    /// This value will be set by the `sync-state rpc` implementation.
    pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
    (
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<AuraId>(s),
        get_from_seed::<GrandpaId>(s),
    )
}

#[cfg(feature = "runtime-benchmarks")]
pub fn benchmarks_config() -> Result<ChainSpec, String> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert(
        "ss58Format".into(),
        sherpax_runtime::SS58Prefix::get().into(),
    );

    Ok(ChainSpec::from_genesis(
        "Benchmarks",
        "benchmarks",
        ChainType::Development,
        move || {
            let caller: AccountId = frame_benchmarking::whitelisted_caller();
            sherpax_genesis(
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                caller.clone(),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    caller.clone(),
                ],
                false,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Default::default(),
    ))
}

pub fn development_config() -> Result<ChainSpec, String> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert(
        "ss58Format".into(),
        sherpax_runtime::SS58Prefix::get().into(),
    );

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            sherpax_genesis(
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Default::default(),
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert(
        "ss58Format".into(),
        sherpax_runtime::SS58Prefix::get().into(),
    );

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            sherpax_genesis(
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Default::default(),
    ))
}

fn sherpax_session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
    SessionKeys { aura, grandpa }
}

/// Configure initial storage state for FRAME modules.
pub fn sherpax_genesis(
    initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    load_genesis: bool,
) -> GenesisConfig {
    let (balances, vesting) = if load_genesis {
        let (balances, vesting) = load_genesis_config();
        let other_balances = endowed_accounts
            .iter()
            .cloned()
            .map(|k| (k, UNITS * 4096))
            .collect();

        (vec![balances, other_balances].concat(), vesting)
    } else {
        let balances = endowed_accounts
            .iter()
            .cloned()
            .map(|k| (k, UNITS * 4096))
            .collect();

        (balances, Default::default())
    };

    let wasm_binary = WASM_BINARY.unwrap();
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig { balances },
        aura: Default::default(),
        grandpa: Default::default(),
        session: SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        (x.0).clone(),
                        (x.0).clone(),
                        sherpax_session_keys(x.1.clone(), x.2.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        vesting: VestingConfig { vesting },
        evm: Default::default(),
        ethereum: Default::default(),
        assets: Default::default(),
        assets_bridge: AssetsBridgeConfig { admin_key: None },
    }
}

#[allow(clippy::type_complexity)]
fn load_genesis_config() -> (
    Vec<(AccountId, Balance)>,
    Vec<(AccountId, BlockNumber, BlockNumber, Balance)>,
) {
    let non_zero_balances = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/res/genesis_config/balances/non_zero_airdrop_18016_10500000000000000000000000.json"
    ))
    .to_vec();

    let vestings = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/res/genesis_config/vesting/vesting_airdrop_18015_943318423258727000000000.json"
    ))
    .to_vec();

    let balances_configs: Vec<sherpax_runtime::BalancesConfig> =
        config_from_json_bytes(vec![non_zero_balances]).unwrap();

    let vesting_configs: Vec<sherpax_runtime::VestingConfig> =
        config_from_json_bytes(vec![vestings]).unwrap();

    let mut total_issuance: Balance = 0u128;
    let balances = balances_configs
        .into_iter()
        .flat_map(|bc| bc.balances)
        .fold(
            BTreeMap::<AccountId, Balance>::new(),
            |mut acc, (account_id, amount)| {
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
            },
        )
        .into_iter()
        .collect();

    assert_eq!(
        total_issuance,
        10_500_000 * UNITS,
        "total issuance must be equal to 10_500_000 KSX"
    );

    let vestings: Vec<(AccountId, BlockNumber, BlockNumber, Balance)> = vesting_configs
        .into_iter()
        .flat_map(|vc| vc.vesting)
        .collect();
    let vesting_liquid = vestings.iter().map(|(_, _, _, free)| free).sum::<u128>();

    assert_eq!(
        vestings.len(),
        18015,
        "total vesting accounts must be equal 18_015."
    );
    assert_eq!(
        vesting_liquid, 943318423258727000000000,
        "total vesting liquid must be equal 943_318.423258727 KSX"
    );

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
