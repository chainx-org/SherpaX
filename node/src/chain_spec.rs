use crate::bitcoin::{
    btc_genesis_params, BtcGenesisParams, BtcParams, BtcTrusteeParams, BtcTxVerifier, Chain,
    TrusteeInfoConfig,
};
use hex_literal::hex;
use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
pub use sherpax_runtime::{
    constants::currency::UNITS, opaque::SessionKeys, AccountId, AssetsBridgeConfig, AssetsConfig,
    AuraConfig, Balance, BalancesConfig, BlockNumber, EthereumConfig, EvmConfig, GenesisAccount,
    GenesisConfig, GrandpaConfig, SessionConfig, Signature, SudoConfig, SystemConfig,
    VestingConfig, DAYS, WASM_BINARY,
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::collections::BTreeMap;
use sp_core::crypto::UncheckedInto;

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
    properties.insert("tokenDecimals".into(), 18i32.into());
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
                btc_genesis_params(include_str!(
                    "../res/genesis_config/gateway/btc_genesis_params_testnet.json"
                )),
                crate::bitcoin::benchmarks_trustees(),
                hex!("d4dcddf3586f5d60568cddcda61b4f1395f22adda5920f5ac60434911b535076").into(),
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
    properties.insert("tokenDecimals".into(), 18i32.into());
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
                btc_genesis_params(include_str!(
                    "../res/genesis_config/gateway/btc_genesis_params_testnet.json"
                )),
                crate::bitcoin::local_testnet_trustees(),
                hex!("d4dcddf3586f5d60568cddcda61b4f1395f22adda5920f5ac60434911b535076").into(),
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
    properties.insert("tokenDecimals".into(), 18i32.into());
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
                btc_genesis_params(include_str!(
                    "../res/genesis_config/gateway/btc_genesis_params_testnet.json"
                )),
                vec![],
                hex!("d4dcddf3586f5d60568cddcda61b4f1395f22adda5920f5ac60434911b535076").into(),
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

#[allow(unused)]
pub fn testnet_config() -> Result<ChainSpec, String> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "KSX".into());
    properties.insert("tokenDecimals".into(), 18i32.into());
    properties.insert(
        "ss58Format".into(),
        sherpax_runtime::SS58Prefix::get().into(),
    );

    Ok(ChainSpec::from_genesis(
        // Name
        "SherpaX Testnet",
        // ID
        "sherpax_testnet",
        ChainType::Live,
        move || {
            sherpax_genesis(
                // Initial PoA authorities
                vec![
                    (
                        hex!("30c72a127fbbadf95c6b0ef5f27c8471e7fc602d8ceaf6e28f9519354b99a63d").into(),
                        hex!("e07d42d9b6a3403be406efaaaf952981c2e124cabc305b49b179546d5cfe7f0e").unchecked_into(),
                        hex!("67b4639b336f7fcefc2b7696be57dbf5059208d01ad67e08ff9688d97efdb519").unchecked_into(),
                    ),
                    (
                        hex!("a4c41a8cce0963ae34319687d5f6b52be531586e49448d63b9366b86f7455438").into(),
                        hex!("86a185b97c75744c614355991d5faac5ea8a57eb6b24a4baf352246f5eb58221").unchecked_into(),
                        hex!("c17b592b9ccf92127726607881c51304df8b8bb002caff9cd864a046cc85d4d0").unchecked_into(),
                    ),
                ],
                // Sudo account
                hex!("a62add1af3bcf9256aa2def0fea1b9648cb72517ccee92a891dc2903a9093e52").into(),
                // Pre-funded accounts
                vec![
                    hex!("a62add1af3bcf9256aa2def0fea1b9648cb72517ccee92a891dc2903a9093e52").into(),
                ],
                true,
                btc_genesis_params(include_str!(
                    "../res/genesis_config/gateway/btc_genesis_params_testnet.json"
                )),
                vec![],
                hex!("d4dcddf3586f5d60568cddcda61b4f1395f22adda5920f5ac60434911b535076").into(),
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

pub fn live_testnet_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/sherpax-testnet-raw.json")[..])
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
    bitcoin: BtcGenesisParams,
    trustees: Vec<(Chain, TrusteeInfoConfig, Vec<BtcTrusteeParams>)>,
    relayer: AccountId,
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

    let btc_genesis_trustees = if trustees.is_empty() {
        vec![]
    } else {
        trustees
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
            .expect("bitcoin trustees generation can not fail; qed")
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
            key: root_key.clone(),
        },
        vesting: VestingConfig { vesting },
        evm: Default::default(),
        ethereum: Default::default(),
        assets: Default::default(),
        assets_bridge: AssetsBridgeConfig { admin_key: None },
        council: sherpax_runtime::CouncilConfig::default(),
        elections: sherpax_runtime::ElectionsConfig::default(),
        x_gateway_common: sherpax_runtime::XGatewayCommonConfig {
            trustees,
            genesis_trustee_transition_duration: 30 * DAYS,
            genesis_trustee_transition_status: false,
            relayer,
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
