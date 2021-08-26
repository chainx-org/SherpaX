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

//! Runtime API definition for pallet-coming-id module.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::too_many_arguments)]

use codec::Codec;
use sp_core::Bytes;
use sp_std::prelude::Vec;

pub use pallet_coming_id::{BondData, BondType, Cid, CidDetails};

sp_api::decl_runtime_apis! {
    pub trait ComingIdApi<AccountId> where
        AccountId: Codec
    {
        fn get_account_id(cid: Cid) -> Option<AccountId>;
        fn get_cids(account: AccountId) -> Vec<Cid>;
        fn get_bond_data(cid: Cid) -> Option<CidDetails<AccountId>>;
        fn get_card(cid: Cid) -> Option<Bytes>;
    }
}
