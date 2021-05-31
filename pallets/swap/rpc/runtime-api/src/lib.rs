// Copyright 2019-2021 ChainX Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use pallet_swap::{rpc::TokenInfo, AssetId};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
     pub trait SwapApi<AccountId>
     where
        AccountId: Codec,
     {
         fn get_amount_in_price(amount_out: u128, path: Vec<AssetId>) -> u128;

         fn get_amount_out_price(amount_in: u128, path: Vec<AssetId>) -> u128;

         fn get_token_list() -> Vec<TokenInfo>;

         fn get_balance(asset_id: AssetId, account: AccountId) -> u128;

         fn get_all_pairs() -> Vec<(AssetId, AssetId)>;
     }
}
