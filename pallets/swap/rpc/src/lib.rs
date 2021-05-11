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

use pallet_swap::AssetId;
use pallet_swap_rpc_runtime_api::SwapApi as SwapRuntimeApi;

#[rpc]
pub trait SwapApi<BlockHash, AccountId> {
    #[rpc(name = "Swap_getAmountInPrice")]
    fn get_amount_in_price(
        &self,
        supply: u128,
        path: Vec<AssetId>,
        at: Option<BlockHash>,
    ) -> Result<NumberOrHex>;

    #[rpc(name = "Swap_getAmountOutPrice")]
    fn get_amount_out_price(
        &self,
        supply: u128,
        path: Vec<AssetId>,
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

impl<C, Block, AccountId> SwapApi<<Block as BlockT>::Hash, AccountId>
for Swap<C, Block>
    where
        Block: BlockT,
        AccountId: Codec,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: SwapRuntimeApi<Block, AccountId>,
{
    //buy amount token price
    fn get_amount_in_price(
        &self,
        supply: u128,
        path: Vec<AssetId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<NumberOrHex> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        api.get_amount_in_price(&at, supply, path)
            .map(|price| price.into())
            .map_err(runtime_error_into_rpc_err)
    }

    //sell amount token price
    fn get_amount_out_price(
        &self,
        supply: u128,
        path: Vec<AssetId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<NumberOrHex> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        api.get_amount_out_price(&at, supply, path)
            .map(|price| price.into())
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
