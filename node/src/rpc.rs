//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sherpax_runtime::{opaque::Block, AccountId, Balance, BlockNumber, Index};
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

// EVM
use fc_rpc::{
    EthBlockDataCacheTask, OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override,
    SchemaV2Override, SchemaV3Override, StorageOverride,
};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use fp_storage::EthereumStorageSchema;
use jsonrpc_pubsub::manager::SubscriptionManager;
use sc_client_api::{
    backend::{AuxStore, Backend, StateBackend, StorageProvider},
    client::BlockchainEvents,
};
use sc_network::NetworkService;
use sc_rpc::SubscriptionTaskExecutor;
use sc_transaction_pool::{ChainApi, Pool};
use sherpax_runtime::Hash;
use sp_runtime::traits::BlakeTwo256;
use std::collections::BTreeMap;

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
    /// Network service
    pub network: Arc<NetworkService<Block, Hash>>,
    /// EthFilterApi pool.
    pub filter_pool: Option<FilterPool>,
    /// Backend.
    pub backend: Arc<fc_db::Backend<Block>>,
    /// Maximum number of logs in a query.
    pub max_past_logs: u32,
    /// Maximum fee history cache size.
    pub fee_history_limit: u64,
    /// Fee history cache.
    pub fee_history_cache: FeeHistoryCache,
    /// Ethereum data access overrides.
    pub overrides: Arc<OverrideHandle<Block>>,
    /// Cache for Ethereum block data.
    pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
}

pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
where
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: sp_api::ApiExt<Block>
        + fp_rpc::EthereumRuntimeRPCApi<Block>
        + fp_rpc::ConvertTransactionRuntimeApi<Block>,
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
{
    let mut overrides_map = BTreeMap::new();
    overrides_map.insert(
        EthereumStorageSchema::V1,
        Box::new(SchemaV1Override::new(client.clone()))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );
    overrides_map.insert(
        EthereumStorageSchema::V2,
        Box::new(SchemaV2Override::new(client.clone()))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );

    overrides_map.insert(
        EthereumStorageSchema::V3,
        Box::new(SchemaV3Override::new(client.clone()))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );

    Arc::new(OverrideHandle {
        schemas: overrides_map,
        fallback: Box::new(RuntimeApiStorageOverride::new(client)),
    })
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, BE, A>(
    deps: FullDeps<C, P, A>,
    subscription_task_executor: SubscriptionTaskExecutor,
) -> RpcExtension
where
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
    C: BlockchainEvents<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: BlockBuilder<Block>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
    // C::Api: xpallet_gateway_bitcoin_rpc_runtime_api::XGatewayBitcoinApi<Block, AccountId>,
    // C::Api: xpallet_gateway_common_rpc_runtime_api::XGatewayCommonApi<
    //     Block,
    //     AccountId,
    //     Balance,
    //     BlockNumber,
    // >,
    // C::Api: xpallet_gateway_records_rpc_runtime_api::XGatewayRecordsApi<
    //     Block,
    //     AccountId,
    //     Balance,
    //     BlockNumber,
    // >,
    P: TransactionPool<Block = Block> + Sync + Send + 'static,
    A: ChainApi<Block = Block> + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};
    // use xpallet_gateway_bitcoin_rpc::{XGatewayBitcoin, XGatewayBitcoinApi};
    // use xpallet_gateway_common_rpc::{XGatewayCommon, XGatewayCommonApi};
    // use xpallet_gateway_records_rpc::{XGatewayRecords, XGatewayRecordsApi};

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
        fee_history_limit,
        fee_history_cache,
        overrides,
        block_data_cache,
    } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool.clone(),
        deny_unsafe,
    )));

    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));

    // eth api
    {
        use fc_rpc::{
            EthApi, EthApiServer, EthFilterApi, EthFilterApiServer, EthPubSubApi,
            EthPubSubApiServer, HexEncodedIdProvider, NetApi, NetApiServer, Web3Api, Web3ApiServer,
        };

        io.extend_with(EthApiServer::to_delegate(EthApi::new(
            client.clone(),
            pool.clone(),
            graph,
            Some(sherpax_runtime::TransactionConverter),
            network.clone(),
            Vec::new(),
            overrides.clone(),
            backend.clone(),
            is_authority,
            max_past_logs,
            block_data_cache.clone(),
            fc_rpc::format::Geth,
            fee_history_limit,
            fee_history_cache,
        )));

        if let Some(filter_pool) = filter_pool {
            io.extend_with(EthFilterApiServer::to_delegate(EthFilterApi::new(
                client.clone(),
                backend,
                filter_pool,
                500_usize, // max stored filters
                max_past_logs,
                block_data_cache,
            )));
        }

        io.extend_with(NetApiServer::to_delegate(NetApi::new(
            client.clone(),
            network.clone(),
            // Whether to format the `peer_count` response as Hex (default) or not.
            true,
        )));

        io.extend_with(Web3ApiServer::to_delegate(Web3Api::new(client.clone())));

        io.extend_with(EthPubSubApiServer::to_delegate(EthPubSubApi::new(
            pool,
            client.clone(),
            network,
            SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
                HexEncodedIdProvider::default(),
                Arc::new(subscription_task_executor),
            ),
            overrides,
        )));
    }

    // io.extend_with(XGatewayBitcoinApi::to_delegate(XGatewayBitcoin::new(
    //     client.clone(),
    // )));
    // io.extend_with(XGatewayRecordsApi::to_delegate(XGatewayRecords::new(
    //     client.clone(),
    // )));
    // io.extend_with(XGatewayCommonApi::to_delegate(XGatewayCommon::new(client)));

    io
}
