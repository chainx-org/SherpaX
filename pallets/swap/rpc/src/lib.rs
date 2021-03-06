// Copyright 2019-2021 ChainX Project Authors. Licensed under GPL-3.0.

//! RPC interface for the pallet swap module.
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

use pallet_swap::{rpc::TokenInfo, AssetId};
use pallet_swap_rpc_runtime_api::SwapApi as SwapRuntimeApi;

#[rpc]
pub trait SwapApi<BlockHash, AccountId> {
    /// Return amount in price by amount out
    #[rpc(name = "swap_getAmountInPrice")]
    fn get_amount_in_price(
        &self,
        amount_out: String,
        path: Vec<AssetId>,
        at: Option<BlockHash>,
    ) -> Result<NumberOrHex>;

    /// Return amount out price by amount in
    #[rpc(name = "swap_getAmountOutPrice")]
    fn get_amount_out_price(
        &self,
        amount_in: String,
        path: Vec<AssetId>,
        at: Option<BlockHash>,
    ) -> Result<NumberOrHex>;

    /// Return all token list info
    #[rpc(name = "swap_getTokenList")]
    fn get_token_list(&self, at: Option<BlockHash>) -> Result<Vec<TokenInfo>>;

    /// Return balance of (asset_id, who)
    #[rpc(name = "swap_getBalance")]
    fn get_balance(
        &self,
        asset_id: AssetId,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Result<NumberOrHex>;
}

const RUNTIME_ERROR: i64 = 1;

pub struct Swap<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Swap<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

impl<C, Block, AccountId> SwapApi<<Block as BlockT>::Hash, AccountId> for Swap<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: SwapRuntimeApi<Block, AccountId>,
{
    fn get_amount_in_price(
        &self,
        amount_out: String,
        path: Vec<AssetId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<NumberOrHex> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let amount_out = amount_out.parse::<u128>().map_err(runtime_error_into_rpc_err)?;
        api.get_amount_in_price(&at, amount_out, path)
            .map(|price| price.into())
            .map_err(runtime_error_into_rpc_err)
    }

    fn get_amount_out_price(
        &self,
        amount_in: String,
        path: Vec<AssetId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<NumberOrHex> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let amount_in = amount_in.parse::<u128>().map_err(runtime_error_into_rpc_err)?;
        api.get_amount_out_price(&at, amount_in, path)
            .map(|price| price.into())
            .map_err(runtime_error_into_rpc_err)
    }

    fn get_token_list(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<TokenInfo>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.get_token_list(&at).map_err(runtime_error_into_rpc_err)
    }

    fn get_balance(
        &self,
        asset_id: AssetId,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<NumberOrHex> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        api.get_balance(&at, asset_id, account)
            .map(|balance| balance.into())
            .map_err(runtime_error_into_rpc_err)
    }
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(RUNTIME_ERROR),
        message: "Runtime trapped".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
