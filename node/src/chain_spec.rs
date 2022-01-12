use crate::bitcoin::{
    btc_genesis_params, BtcGenesisParams, BtcParams, BtcTrusteeParams, BtcTxVerifier, Chain,
    TrusteeInfoConfig,
};
use frame_benchmarking::frame_support::PalletId;
use hex_literal::hex;
use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
pub use sherpax_runtime::{
    constants::currency::UNITS, opaque::SessionKeys, AccountId, AssetsBridgeConfig, AssetsConfig,
    AuraConfig, Balance, BalancesConfig, BlockNumber, EthereumChainIdConfig, EthereumConfig,
    EvmConfig, GenesisAccount, GenesisConfig, GrandpaConfig, SessionConfig, Signature, SudoConfig,
    SystemConfig, TechnicalMembershipConfig, VestingConfig, DAYS, WASM_BINARY,
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::UncheckedInto;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{AccountIdConversion, IdentifyAccount, Verify};
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
type AssetId = u32;
type AssetName = Vec<u8>;
type AssetSymbol = Vec<u8>;
type AssetDecimals = u8;
type AssetSufficient = bool;
type AssetMinBalance = Balance;

/// Asset registration
fn sbtc() -> (Chain, AssetId) {
    (Chain::Bitcoin, 1)
}

#[allow(clippy::type_complexity)]
fn reserved_assets(
    root_key: &AccountId,
) -> (
    Vec<(AssetId, AccountId, AssetSufficient, AssetMinBalance)>,
    Vec<(AssetId, AssetName, AssetSymbol, AssetDecimals)>,
) {
    (
        vec![
            (0, root_key.clone(), true, 10_000_000_000u128),
            (1, root_key.clone(), true, 1u128),
            (2, root_key.clone(), true, 10_000_000_000u128),
            (3, root_key.clone(), true, 10_000_000_000u128),
            (4, root_key.clone(), true, 10_000_000_000u128),
            (5, root_key.clone(), true, 10_000_000_000u128),
            (6, root_key.clone(), true, 10_000_000_000u128),
            (7, root_key.clone(), true, 10_000_000_000u128),
            (8, root_key.clone(), true, 10_000_000_000u128),
            (9, root_key.clone(), true, 10_000_000_000u128),
        ],
        vec![
            (
                0,
                "Reserved0".to_string().into_bytes(),
                "RSV0".to_string().into_bytes(),
                18,
            ),
            (
                1,
                "SBTC".to_string().into_bytes(),
                "SBTC".to_string().into_bytes(),
                8,
            ),
            (
                2,
                "Reserved2".to_string().into_bytes(),
                "RSV2".to_string().into_bytes(),
                18,
            ),
            (
                3,
                "Reserved3".to_string().into_bytes(),
                "RSV3".to_string().into_bytes(),
                18,
            ),
            (
                4,
                "Reserved4".to_string().into_bytes(),
                "RSV4".to_string().into_bytes(),
                18,
            ),
            (
                5,
                "Reserved5".to_string().into_bytes(),
                "RSV5".to_string().into_bytes(),
                18,
            ),
            (
                6,
                "Reserved6".to_string().into_bytes(),
                "RSV6".to_string().into_bytes(),
                18,
            ),
            (
                7,
                "Reserved7".to_string().into_bytes(),
                "RSV7".to_string().into_bytes(),
                18,
            ),
            (
                8,
                "Reserved8".to_string().into_bytes(),
                "RSV8".to_string().into_bytes(),
                18,
            ),
            (
                9,
                "Reserved9".to_string().into_bytes(),
                "RSV9".to_string().into_bytes(),
                18,
            ),
        ],
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
                vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
                true,
                btc_genesis_params(include_str!(
                    "../res/genesis_config/gateway/btc_genesis_params_testnet.json"
                )),
                crate::bitcoin::dev_trustees(),
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
                crate::bitcoin::mainnet_trustees(),
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
                        // Ff3b3gdWT2nwC9BSjcys1S8Tth2XBayEgHZkX8pbCrErqgf
                        hex!("884f4d6638c1f70ed80537be200df124efc384e8177f74377a2be919513dcc3a")
                            .into(),
                        hex!("e07d42d9b6a3403be406efaaaf952981c2e124cabc305b49b179546d5cfe7f0e")
                            .unchecked_into(),
                        hex!("67b4639b336f7fcefc2b7696be57dbf5059208d01ad67e08ff9688d97efdb519")
                            .unchecked_into(),
                    ),
                    (
                        // J1SDJ7KvkESXfT8RjSP9Sy8TfUh2UMstRTh7CN7be9NipQB
                        hex!("f054d6fd1444f2e78f2839dc4ec5e4f35f0fc003cf006f3f712f659cdc2ecb63")
                            .into(),
                        hex!("86a185b97c75744c614355991d5faac5ea8a57eb6b24a4baf352246f5eb58221")
                            .unchecked_into(),
                        hex!("c17b592b9ccf92127726607881c51304df8b8bb002caff9cd864a046cc85d4d0")
                            .unchecked_into(),
                    ),
                ],
                // Sudo account
                // FCcnKcbTe5EYDZXDCKwbhkPAoYyakG1iRBxS9Ai5m2uFTfn
                hex!("74276b30236e3ffc822c0e5ec0ac8b02933dac11fcefc88733c8a61cdaa45a59").into(),
                // Pre-funded accounts
                vec![
                    hex!("74276b30236e3ffc822c0e5ec0ac8b02933dac11fcefc88733c8a61cdaa45a59").into(),
                ],
                true,
                btc_genesis_params(include_str!(
                    "../res/genesis_config/gateway/btc_genesis_params_testnet.json"
                )),
                crate::bitcoin::mainnet_trustees(),
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

fn sherpax_session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
    SessionKeys { aura, grandpa }
}

fn technical_committee_membership() -> Vec<AccountId> {
    vec![
        // 5TPu4DCQRSbNS9ESUcNGUn9HcF9AzrHiDP395bDxM9ZAqSD8
        hex!["a62add1af3bcf9256aa2def0fea1b9648cb72517ccee92a891dc2903a9093e52"].into(),
        // 5GxS3YuwjhZZtmPmLEJuGPuz14gEJsunabqNLYTthXfThRwG
        hex!["d86477344ad5c27a45c4c178c7cca1b7b111380a4fbe7e23b3488a42ce56ca30"].into(),
        // DokRDMoUT1ZmTaG18MHKunBMoBv1vqR6xyypU1QRWLc7UH5
        hex!["367a3f0acb9dcb2b000c8bc9deb93c4613512604c0847ff2c1ecd478e7e46714"].into(),
    ]
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
        load_genesis_config(&root_key)
    } else {
        let balances = endowed_accounts
            .iter()
            .cloned()
            .map(|k| (k, UNITS * 4096))
            .collect();

        (balances, Default::default())
    };

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
    let sbtc_info = sbtc();
    let assets_info = reserved_assets(&root_key);
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
        ethereum_chain_id: EthereumChainIdConfig { chain_id: 1506u64 },
        evm: Default::default(),
        ethereum: Default::default(),
        assets: sherpax_runtime::AssetsConfig {
            assets: assets_info.0,
            metadata: assets_info.1,
            accounts: vec![],
        },
        assets_bridge: AssetsBridgeConfig { admin_key: None },
        council: Default::default(),
        elections: Default::default(),
        democracy: Default::default(),
        technical_committee: Default::default(),
        technical_membership: TechnicalMembershipConfig {
            members: technical_committee_membership(),
            phantom: Default::default(),
        },
        treasury: Default::default(),
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
            initial_asset_chain: vec![(sbtc_info.1, sbtc_info.0)],
        },
    }
}

#[allow(clippy::type_complexity)]
fn load_genesis_config(
    root_key: &AccountId,
) -> (
    Vec<(AccountId, Balance)>,
    Vec<(AccountId, BlockNumber, BlockNumber, Balance)>,
) {
    let chainx_snapshot = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/res/genesis_config/balances/genesis_balances_chainx_snapshot_7418_7868415220855310000000000.json"
    ))
        .to_vec();

    let comingchat_miners = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/res/genesis_config/balances/genesis_balances_comingchat_miners_334721_2140742819000000000000000.json"
    ))
        .to_vec();

    let sherpax_contributors = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/res/genesis_config/balances/genesis_balances_sherpax_contributors_1873_94046984872650000000000.json"
    ))
        .to_vec();

    let vestings = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/res/genesis_config/vesting/genesis_vesting_342133_894769078020746000000000.json"
    ))
    .to_vec();

    let balances_configs: Vec<sherpax_runtime::BalancesConfig> = config_from_json_bytes(vec![
        chainx_snapshot,
        comingchat_miners,
        sherpax_contributors,
    ])
    .unwrap();

    let mut mutated_balances: Vec<(AccountId, u128)> = balances_configs
        .into_iter()
        .flat_map(|bc| bc.balances)
        .collect();

    // total transfer vesting balances
    let transfer_balances = 2631584779144690000000000u128;
    // 30000 ksx + transfer vesting balances
    let root_balance = 30000000000000000000000u128.saturating_add(transfer_balances);

    let back_to_treasury = 21000000000000000000000000u128
        .saturating_sub(root_balance)
        .saturating_sub(10103205024727960000000000u128);

    // 5S7WgdAXVK7mh8REvXfk9LdHs3Xqu9B2E9zzY8e4LE8Gg2ZX
    let treasury_account: AccountId = PalletId(*b"pcx/trsy").into_account();

    mutated_balances.push((root_key.clone(), root_balance));
    mutated_balances.push((treasury_account, back_to_treasury));

    let vesting_configs: Vec<sherpax_runtime::VestingConfig> =
        config_from_json_bytes(vec![vestings]).unwrap();

    let mut total_issuance: Balance = 0u128;
    let balances: Vec<(AccountId, u128)> = mutated_balances
        .into_iter()
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
        balances.len(),
        342133 + 1 + 1 + 1873 - 35,
        "total accounts must be equal to 344013"
    );

    assert_eq!(
        total_issuance,
        21000000 * UNITS,
        "total issuance must be equal to 21000000000000000000000000"
    );

    let vestings: Vec<(AccountId, BlockNumber, BlockNumber, Balance)> = vesting_configs
        .into_iter()
        .flat_map(|vc| vc.vesting)
        .collect();
    let vesting_liquid = vestings.iter().map(|(_, _, _, free)| free).sum::<u128>();

    assert_eq!(
        vestings.len(),
        342133,
        "total vesting accounts must be equal 342138."
    );
    assert_eq!(
        vesting_liquid, 894769078020746000000000u128,
        "total vesting liquid must be equal 894769078020746000000000"
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
