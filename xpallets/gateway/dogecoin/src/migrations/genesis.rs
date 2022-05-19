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

pub fn doge_testnet_genesis_params() -> DogeGenesisParams {
    DogeGenesisParams {
        network: Network::DogeCoinTestnet,
        confirmation_number: 1,
        height: 3836100,
        hash: "78450863d7503a8b8441510c5cecbad087aa03e5ce118f33e54aed542491aad1".to_string(),
        version: 6422788,
        previous_header_hash: "fdbc6c89882d0bcb003c96a849bbe8f739eec6e62c1e8a04ecaa2c6fc7f4c385"
            .to_string(),
        merkle_root_hash: "baf30c61780f84f4daa3653474680d96ba4ef318ad8a6f8b9537d994e1c00a2c"
            .to_string(),
        time: 1651731133,
        bits: Compact::new(471186995),
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
    let dogecoin_block_params = doge_testnet_genesis_params();
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
