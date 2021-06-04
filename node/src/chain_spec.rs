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

use hex_literal::hex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

use cumulus_primitives_core::ParaId;

use xp_protocol::{C_BTC as X_BTC, C_DOGE as X_DOGE, S_BTC, S_DOGE, X_ETH};
use xpallet_assets::AssetRestrictions;
use xpallet_assets_registrar::{AssetInfo, Chain};
use xpallet_gateway_bitcoin::{BtcParams, BtcTxVerifier};
use xpallet_gateway_bitcoin_v2::types::TradingPrice;

use crate::bitcoin::BtcGenesisParams;
use dev_parachain_primitives::{AccountId, Signature};
use dev_parachain_runtime::{constants::currency::DOTS, *};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<dev_parachain_runtime::GenesisConfig, Extensions>;

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

pub fn get_chain_spec(id: ParaId) -> Result<ChainSpec, String> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "PCX".into());
    properties.insert("tokenDecimals".into(), 8.into());

    Ok(ChainSpec::from_genesis(
        "SherpaX PC1",
        "sherpax",
        ChainType::Local,
        move || {
            testnet_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    hex!["18ec21f2ee09b23cc0be299d316fe0688b42c3904500f0690bae24328433a025"].into(),
                ],
                id,
                crate::bitcoin::btc_genesis_params(include_str!(
                    "res/bitcoin_testnet_genesis.json"
                )),
                crate::bitcoin::btc_genesis_params(include_str!(
                    "res/dogecoin_testnet_genesis.json"
                )),
                vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
            )
        },
        vec![],
        None,
        Some("chainx"),
        Some(properties),
        Extensions { relay_chain: "westend-dev".into(), para_id: id.into() },
    ))
}

type AssetId = u32;

const PCX: AssetId = 0;
const PCX_DECIMALS: u8 = 8;

fn pcx_restrictions() -> AssetRestrictions {
    AssetRestrictions::DEPOSIT
        | AssetRestrictions::WITHDRAW
        | AssetRestrictions::DESTROY_WITHDRAWAL
        | AssetRestrictions::DESTROY_USABLE
}

fn pcx_asset_info() -> AssetInfo {
    AssetInfo::new::<Runtime>(
        b"PCX".to_vec(),
        b"Polkadot ChainX".to_vec(),
        Chain::ChainX,
        PCX_DECIMALS,
        b"ChainX's crypto currency in Polkadot ecology".to_vec(),
    )
    .unwrap()
}

const BTC_DECIMALS: u8 = 8;
const X_BTC_ASSETRESTRICTIONS: AssetRestrictions = AssetRestrictions::DESTROY_USABLE;

fn sbtc_restrictions() -> AssetRestrictions {
    AssetRestrictions::TRANSFER
}

fn xbtc_asset_info() -> AssetInfo {
    AssetInfo::new::<Runtime>(
        b"XBTC".to_vec(),
        b"ChainX Bitcoin".to_vec(),
        Chain::Bitcoin,
        BTC_DECIMALS,
        b"ChainX's Cross-chain Bitcoin".to_vec(),
    )
    .unwrap()
}

const ETH_DECIMALS: u8 = 8;
const X_ETH_ASSETRESTRICTIONS: AssetRestrictions = AssetRestrictions::DESTROY_USABLE;
fn xeth_asset_info() -> AssetInfo {
    AssetInfo::new::<Runtime>(
        b"XETH".to_vec(),
        b"ChainX ETH".to_vec(),
        Chain::Ethereum,
        ETH_DECIMALS,
        b"ChainX's Cross-chain ETH".to_vec(),
    )
    .unwrap()
}
fn sbtc_asset_info() -> AssetInfo {
    AssetInfo::new::<Runtime>(
        b"SBTC".to_vec(),
        b"ChainX Shadow Bitcoin".to_vec(),
        Chain::Bitcoin,
        BTC_DECIMALS,
        b"Shadow token of ChainX's Cross-chain Bitcoin".to_vec(),
    )
    .unwrap()
}

fn asset_endowed() -> BTreeMap<u32, Vec<(AccountId, u128)>> {
    let mut endowed = BTreeMap::new();

    let endowed_info = vec![
        (get_account_id_from_seed::<sr25519::Public>("Alice"), 1000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Bob"), 2000_000_000_000),
    ];
    endowed.insert(X_BTC, endowed_info);

    let endowed_info = vec![
        (get_account_id_from_seed::<sr25519::Public>("Alice"), 2000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Bob"), 4000_000_000_000),
    ];
    endowed.insert(X_ETH, endowed_info);

    endowed
}

const DOGE_DECIMALS: u8 = 8;
const X_DOGE_ASSETRESTRICTIONS: AssetRestrictions = AssetRestrictions::DESTROY_USABLE;

fn sdoge_restrictions() -> AssetRestrictions {
    AssetRestrictions::TRANSFER
}

fn xdoge_asset_info() -> AssetInfo {
    AssetInfo::new::<Runtime>(
        b"XDOGE".to_vec(),
        b"ChainX Dogecoin".to_vec(),
        Chain::Dogecoin,
        DOGE_DECIMALS,
        b"ChainX's Cross-chain Dogecoin".to_vec(),
    )
    .unwrap()
}

fn sdoge_asset_info() -> AssetInfo {
    AssetInfo::new::<Runtime>(
        b"SDOGE".to_vec(),
        b"ChainX Shadow Dogecin".to_vec(),
        Chain::Dogecoin,
        DOGE_DECIMALS,
        b"Shadow token of ChainX's Cross-chain Dogecoin".to_vec(),
    )
    .unwrap()
}

fn testnet_genesis(
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
    bitcoin: BtcGenesisParams,
    dogecoin: BtcGenesisParams,
    initial_authorities: Vec<AuraId>,
) -> GenesisConfig {
    const ENDOWMENT: u128 = 1_000_000 * DOTS;
    const STASH: u128 = 100 * DOTS;

    GenesisConfig {
        frame_system: SystemConfig {
            code: dev_parachain_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| match k {
                    k if k == root_key => (k, ENDOWMENT),
                    k => (k, STASH),
                })
                .collect(),
        },
        pallet_sudo: SudoConfig { key: root_key },
        parachain_info: ParachainInfoConfig { parachain_id: id },
        xpallet_assets_registrar: XAssetsRegistrarConfig {
            assets: vec![
                (PCX, pcx_asset_info(), true, false),
                (X_ETH, xeth_asset_info(), true, true),
                (X_BTC, xbtc_asset_info(), true, true),
                (S_BTC, sbtc_asset_info(), true, true),
                (X_DOGE, xdoge_asset_info(), true, true),
                (S_DOGE, sdoge_asset_info(), true, true),
            ],
        },
        xpallet_assets: XAssetsConfig {
            assets_restrictions: vec![
                (PCX, pcx_restrictions()),
                (X_ETH, X_ETH_ASSETRESTRICTIONS),
                (X_BTC, X_BTC_ASSETRESTRICTIONS),
                (S_BTC, sbtc_restrictions()),
                (X_DOGE, X_DOGE_ASSETRESTRICTIONS),
                (S_DOGE, sdoge_restrictions()),
            ],
            endowed: Default::default(), // FIXME: maybe issue some asset balances?
        },

        xpallet_gateway_bitcoin_v2_pallet_Instance1: XGatewayBitcoinBridgeConfig {
            exchange_rate: TradingPrice { price: 1, decimal: 2 },
            issue_griefing_fee: 10,
            ..Default::default()
        },
        xpallet_gateway_bitcoin_v2_pallet_Instance2: XGatewayDogecoinBridgeConfig {
            exchange_rate: TradingPrice { price: 1000, decimal: 2 },
            issue_griefing_fee: 10,
            ..Default::default()
        },
        xpallet_gateway_bitcoin_Instance1: XGatewayBitcoinConfig {
            network_id: bitcoin.network,
            confirmation_number: bitcoin.confirmation_number,
            genesis_hash: bitcoin.hash(),
            genesis_info: (bitcoin.header(), bitcoin.height),
            params_info: BtcParams::new(
                486604799,            // max_bits
                2 * 60 * 60,          // block_max_future
                2 * 7 * 24 * 60 * 60, // target_timespan_seconds
                10 * 60,              // target_spacing_seconds
                4,                    // retargeting_factor
            ), // retargeting_factor
            btc_withdrawal_fee: 500000,
            max_withdrawal_count: 100,
            verifier: BtcTxVerifier::Recover,
            ..Default::default()
        },
        xpallet_gateway_bitcoin_Instance2: XGatewayDogecoinConfig {
            network_id: dogecoin.network,
            confirmation_number: dogecoin.confirmation_number,
            genesis_hash: dogecoin.hash(),
            genesis_info: (dogecoin.header(), dogecoin.height),
            params_info: BtcParams::new(
                486604799,            // max_bits
                2 * 60 * 60,          // block_max_future
                2 * 7 * 24 * 60 * 60, // target_timespan_seconds
                10 * 60,              // target_spacing_seconds
                4,                    // retargeting_factor
            ), // retargeting_factor
            btc_withdrawal_fee: 5_000_000_000,
            max_withdrawal_count: 100,
            verifier: BtcTxVerifier::Recover,
            ..Default::default()
        },
        pallet_aura: AuraConfig { authorities: initial_authorities },
        cumulus_pallet_aura_ext: Default::default(),
    }
}
