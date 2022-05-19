// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

extern crate alloc;
use crate::{
    types::DogeHeaderIndex, BestIndex, BlockHashFor, Config, ConfirmationNumber, DogeHeaderInfo,
    DogeMinDeposit, DogeParams, DogeWithdrawalFee, GenesisInfo, Headers, MainChain,
    MaxWithdrawalCount, NetworkId, ParamsInfo,
};
use alloc::string::{String, ToString};
use frame_support::{log::info, traits::Get, weights::Weight};
use light_bitcoin::{
    chain::{h256_rev, BlockHeader, Compact},
    keys::Network,
};
use sp_core::H256;
use sp_std::vec;

#[derive(Debug)]
pub struct DogeGenesisParams {
    pub network: Network,
    pub confirmation_number: u32,
    pub height: u32,
    hash: String,
    version: u32,
    previous_header_hash: String,
    merkle_root_hash: String,
    time: u32,
    bits: Compact,
    nonce: u32,
}

impl DogeGenesisParams {
    /// Return the block hash.
    ///
    /// Indicating user-visible serializations of this hash should be backward.
    pub fn hash(&self) -> H256 {
        h256_rev(&self.hash)
    }

    /// Return the block header.
    ///
    /// Indicating user-visible serializations of `previous_header_hash` and `merkle_root_hash`
    /// should be backward.
    pub fn header(&self) -> BlockHeader {
        BlockHeader {
            version: self.version,
            previous_header_hash: h256_rev(&self.previous_header_hash),
            merkle_root_hash: h256_rev(&self.merkle_root_hash),
            time: self.time,
            bits: self.bits,
            nonce: self.nonce,
        }
    }
}

pub fn doge_mainnet_genesis_params() -> DogeGenesisParams {
    DogeGenesisParams {
        network: Network::DogeCoinMainnet,
        confirmation_number: 6,
        height: 4230635,
        hash: "8a5efa8007ae46db2f6073b31d329031afe1ecd966ffe2440e05964af7bfd4de".to_string(),
        version: 6422788,
        previous_header_hash: "87cc425af5a4365c7484f1b94c932323c6dd2309444683086b7284d49500b7c6"
            .to_string(),
        merkle_root_hash: "4a57bcb025773314a42fe1bff32b36ad3bb4fed063ff7a7c97c4b78c7c1c12f2"
            .to_string(),
        time: 1652951041,
        bits: Compact::new(436401055),
        nonce: 0,
    }
}

/// Initialize the new module by storing migration
pub fn apply<T: Config>() -> Weight {
    info!(
        target: "runtime::gateway::dogecoin",
        "✅ Running migration for gateway dogecoin pallet..."
    );
    dogecoin_genesis::<T>()
}

pub fn dogecoin_genesis<T: Config>() -> Weight {
    // TODO：Use dogecoin mainnet genesis info.
    let dogecoin_block_params = doge_mainnet_genesis_params();
    let dogecoin_params = DogeParams::new(
        // for dogecoin
        545259519,            // max_bits
        2 * 60 * 60,          // block_max_future
        2 * 7 * 24 * 60 * 60, // target_timespan_seconds
        10 * 60,              // target_spacing_seconds
        4,                    // retargeting_factor
    );
    // genesis network id
    let genesis_hash = dogecoin_block_params.hash();
    let (genesis_header, genesis_height) =
        (dogecoin_block_params.header(), dogecoin_block_params.height);
    let genesis_index = DogeHeaderIndex {
        hash: genesis_hash,
        height: genesis_height,
    };
    let header_info = DogeHeaderInfo {
        header: genesis_header,
        height: genesis_height,
    };

    Headers::<T>::insert(&genesis_hash, header_info);
    BlockHashFor::<T>::insert(&genesis_index.height, vec![genesis_hash]);
    MainChain::<T>::insert(&genesis_hash, true);
    BestIndex::<T>::put(genesis_index);
    GenesisInfo::<T>::put((genesis_header, genesis_height));
    ParamsInfo::<T>::put(dogecoin_params);
    NetworkId::<T>::put(dogecoin_block_params.network);
    ConfirmationNumber::<T>::put(dogecoin_block_params.confirmation_number);
    DogeWithdrawalFee::<T>::put(2_000_000_000);
    MaxWithdrawalCount::<T>::put(100);
    DogeMinDeposit::<T>::put(1_000_000_000);

    info!(
        target: "runtime::gateway::dogecoin",
        "✅ Migration for dogecoin genesis done"
    );
    <T as frame_system::Config>::DbWeight::get().writes(11)
}
