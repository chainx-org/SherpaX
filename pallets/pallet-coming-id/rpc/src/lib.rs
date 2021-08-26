// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! RPC interface for the pallet-coming-id module.

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
pub use pallet_coming_id_rpc_runtime_api::ComingIdApi as ComingIdRuntimeApi;
use pallet_coming_id_rpc_runtime_api::{Cid, CidDetails};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait ComingIdApi<BlockHash, AccountId> {
    #[rpc(name = "get_account_id")]
    fn get_account_id(&self, cid: Cid, at: Option<BlockHash>) -> Result<Option<AccountId>>;

    #[rpc(name = "get_cids")]
    fn get_cids(&self, account: AccountId, at: Option<BlockHash>) -> Result<Vec<Cid>>;

    #[rpc(name = "get_bond_data")]
    fn get_bond_data(
        &self,
        cid: Cid,
        at: Option<BlockHash>,
    ) -> Result<Option<CidDetails<AccountId>>>;

    #[rpc(name = "get_card")]
    fn get_card(&self, cid: Cid, at: Option<BlockHash>) -> Result<Option<Bytes>>;
}

/// A struct that implements the [`ComingIdApi`].
pub struct ComingId<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> ComingId<C, P> {
    /// Create new `ComingId` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

impl<C, Block, AccountId> ComingIdApi<<Block as BlockT>::Hash, AccountId> for ComingId<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ComingIdRuntimeApi<Block, AccountId>,
{
    fn get_account_id(
        &self,
        cid: Cid,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<AccountId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        api.get_account_id(&at, cid).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to get bond.".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_cids(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<Cid>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        api.get_cids(&at, account).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to get bonds.".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_bond_data(
        &self,
        cid: Cid,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<CidDetails<AccountId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        api.get_bond_data(&at, cid).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to get bond.".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_card(&self, cid: Cid, at: Option<<Block as BlockT>::Hash>) -> Result<Option<Bytes>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        api.get_card(&at, cid).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to get card.".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
