// Copyright 2019-2021 ChainX Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_std::vec::Vec;
use pallet_swap::AssetId;

sp_api::decl_runtime_apis! {
     pub trait SwapApi<AccountId>
     where
        AccountId: Codec,
     {
        //buy amount token price
        fn get_amount_in_price(supply: u128, path: Vec<AssetId>) -> u128;

        //sell amount token price
        fn get_amount_out_price(supply: u128, path: Vec<AssetId>) -> u128;

     }
}
