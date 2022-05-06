// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

//! RPC interface for the transaction verification.
use codec::Codec;
use jsonrpc_derive::rpc;
use std::sync::Arc;
use std::vec::Vec;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use xp_rpc::{runtime_error_into_rpc_err, Result};
use xpallet_gateway_dogecoin_rpc_runtime_api::{
    BlockHeader, DogeHeaderInfo, DogeWithdrawalProposal,
    XGatewayDogecoinApi as XGatewayDogecoinRuntimeApi, H256,
};

pub struct XGatewayDogecoin<C, B, AccountId> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<(B, AccountId)>,
}

impl<C, B, AccountId> XGatewayDogecoin<C, B, AccountId> {
    /// Create new `XGatewayDogecoin` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

#[rpc]
pub trait XGatewayDogecoinApi<BlockHash, AccountId> {
    /// Verify transaction is valid
    #[rpc(name = "XGatewayDogecoin_verifyTxValid")]
    fn verify_tx_valid(
        &self,
        raw_tx: String,
        withdrawal_id_list: Vec<u32>,
        full_amount: bool,
        at: Option<BlockHash>,
    ) -> Result<bool>;

    /// Get withdrawal proposal
    #[rpc(name = "XGatewayDogecoin_getWithdrawalProposal")]
    fn get_withdrawal_proposal(
        &self,
        at: Option<BlockHash>,
    ) -> Result<Option<DogeWithdrawalProposal<AccountId>>>;

    /// Get genesis info
    #[rpc(name = "XGatewayDogecoin_getGenesisInfo")]
    fn get_genesis_info(&self, at: Option<BlockHash>) -> Result<(BlockHeader, u32)>;

    /// Get block header
    #[rpc(name = "XGatewayDogecoin_getDogeBlockHeader")]
    fn get_btc_block_header(
        &self,
        txid: H256,
        at: Option<BlockHash>,
    ) -> Result<Option<DogeHeaderInfo>>;
}

impl<C, Block, AccountId> XGatewayDogecoinApi<<Block as BlockT>::Hash, AccountId>
    for XGatewayDogecoin<C, Block, AccountId>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: XGatewayDogecoinRuntimeApi<Block, AccountId>,
    AccountId: Codec + Send + Sync + 'static,
{
    fn verify_tx_valid(
        &self,
        raw_tx: String,
        withdrawal_id_list: Vec<u32>,
        full_amount: bool,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<bool> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let raw_tx = hex::decode(raw_tx).map_err(runtime_error_into_rpc_err)?;
        let result = api
            .verify_tx_valid(&at, raw_tx, withdrawal_id_list, full_amount)
            .map_err(runtime_error_into_rpc_err)?
            .map_err(runtime_error_into_rpc_err)?;
        Ok(result)
    }

    fn get_withdrawal_proposal(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<DogeWithdrawalProposal<AccountId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let result = api
            .get_withdrawal_proposal(&at)
            .map_err(runtime_error_into_rpc_err)?;
        Ok(result)
    }

    fn get_genesis_info(&self, at: Option<<Block as BlockT>::Hash>) -> Result<(BlockHeader, u32)> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let result = api
            .get_genesis_info(&at)
            .map_err(runtime_error_into_rpc_err)?;
        Ok(result)
    }

    fn get_btc_block_header(
        &self,
        txid: H256,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<DogeHeaderInfo>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let reslut = api
            .get_doge_block_header(&at, txid)
            .map_err(runtime_error_into_rpc_err)?;
        Ok(reslut)
    }
}
