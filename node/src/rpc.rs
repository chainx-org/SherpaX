// Copyright 2020-2021 ChainX
// Copyright 2021 Parity Technologies (UK) Ltd.
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

//! Parachain-specific RPCs implementation.

#![warn(missing_docs)]

use std::sync::Arc;

use sc_client_api::AuxStore;
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

use runtime_common::{AccountId, Balance, Block, Index as Nonce};

// EVM
use std::collections::BTreeMap;
use runtime_common::Hash;
use sp_runtime::traits::BlakeTwo256;
use sc_network::NetworkService;
use sc_client_api::{
    backend::{StorageProvider, Backend, StateBackend},
    client::BlockchainEvents
};
use sc_transaction_pool::{ChainApi, Pool};
use fc_rpc_core::types::FilterPool;
use jsonrpc_pubsub::manager::SubscriptionManager;
use pallet_ethereum::EthereumStorageSchema;
use fc_rpc::{
    StorageOverride, SchemaV1Override, OverrideHandle, RuntimeApiStorageOverride, EthBlockDataCache
};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Full client dependencies
pub struct FullDeps<C, P, A: ChainApi> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Graph pool instance.
    pub graph: Arc<Pool<A>>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// The Node authority flag
    pub is_authority: bool,
    /// Whether to enable dev signer
    pub enable_dev_signer: bool,
    /// Network service
    pub network: Arc<NetworkService<Block, Hash>>,
    /// EthFilterApi pool.
    pub filter_pool: Option<FilterPool>,
    /// Backend.
    pub backend: Arc<fc_db::Backend<Block>>,
    /// Maximum number of logs in a query.
    pub max_past_logs: u32,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, BE, A>(
    deps: FullDeps<C, P, A>,
    subscription_task_executor: SubscriptionTaskExecutor
) -> RpcExtension
    where
        C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + AuxStore
        + HeaderMetadata<Block, Error = BlockChainError>
        + Send
        + Sync
        + 'static,
        C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
        C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
        C::Api: BlockBuilder<Block>,
        C::Api: pallet_coming_id_rpc::ComingIdRuntimeApi<Block, AccountId>,
        P: TransactionPool<Block = Block> + Sync + Send + 'static,
        BE: Backend<Block> + 'static,
        BE::State: StateBackend<BlakeTwo256>,
        C: StorageProvider<Block, BE> + BlockchainEvents<Block>,
        C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
        A: ChainApi<Block = Block> + 'static,
{
    use substrate_frame_rpc_system::{FullSystem, SystemApi};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use pallet_coming_id_rpc::{ComingId, ComingIdApi};
    use fc_rpc::{
        EthApi, EthApiServer, EthFilterApi, EthFilterApiServer, NetApi, NetApiServer,
        EthPubSubApi, EthPubSubApiServer, Web3Api, Web3ApiServer, EthDevSigner, EthSigner,
        HexEncodedIdProvider,
    };

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        graph,
        deny_unsafe,
        is_authority,
        network,
        filter_pool,
        backend,
        max_past_logs,
        enable_dev_signer,
    } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool.clone(),
        deny_unsafe,
    )));
    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));
    io.extend_with(ComingIdApi::to_delegate(ComingId::new(client.clone())));

    let mut signers = Vec::new();
    if enable_dev_signer {
        signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
    }
    let mut overrides_map = BTreeMap::new();
    overrides_map.insert(
        EthereumStorageSchema::V1,
        Box::new(SchemaV1Override::new(client.clone())) as Box<dyn StorageOverride<_> + Send + Sync>
    );

    let overrides = Arc::new(OverrideHandle {
        schemas: overrides_map,
        fallback: Box::new(RuntimeApiStorageOverride::new(client.clone())),
    });

    let block_data_cache = Arc::new(EthBlockDataCache::new(50, 50));

    io.extend_with(
        EthApiServer::to_delegate(EthApi::new(
            client.clone(),
            pool.clone(),
            graph,
            sherpax_runtime::TransactionConverter,
            network.clone(),
            signers,
            overrides.clone(),
            backend.clone(),
            is_authority,
            max_past_logs,
            block_data_cache.clone()
        ))
    );

    if let Some(filter_pool) = filter_pool {
        io.extend_with(
            EthFilterApiServer::to_delegate(EthFilterApi::new(
                client.clone(),
                backend,
                filter_pool.clone(),
                500 as usize, // max stored filters
                overrides.clone(),
                max_past_logs,
                block_data_cache
            ))
        );
    }

    io.extend_with(
        NetApiServer::to_delegate(NetApi::new(
            client.clone(),
            network.clone(),
            // Whether to format the `peer_count` response as Hex (default) or not.
            true,
        ))
    );
    io.extend_with(
        Web3ApiServer::to_delegate(Web3Api::new(
            client.clone(),
        ))
    );
    io.extend_with(
        EthPubSubApiServer::to_delegate(EthPubSubApi::new(
            pool,
            client,
            network,
            SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
                HexEncodedIdProvider::default(),
                Arc::new(subscription_task_executor)
            ),
            overrides
        ))
    );

    io
}
