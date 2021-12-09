// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod currency {
    use runtime_common::Balance;

    /// The existential deposit = 10_000_000_000.
    pub const EXISTENTIAL_DEPOSIT: Balance = MILLICENTS / 100;

    pub const UNITS: Balance = 1_000_000_000_000_000_000;
    pub const CENTS: Balance = UNITS / 1_000;
    pub const MILLICENTS: Balance = CENTS / 1_000;

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        (items as Balance) * 20 * UNITS + (bytes as Balance) * CENTS
    }
}

/// Fee-related.
pub mod fee {
    use frame_support::weights::{
        constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    };
    use runtime_common::Balance;
    use smallvec::smallvec;
    pub use sp_runtime::Perbill;

    /// The block saturation level. Fees will be updates based on this value.
    pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

    /// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
    /// node's balance type.
    ///
    /// This should typically create a mapping between the following ranges:
    ///   - [0, MAXIMUM_BLOCK_WEIGHT]
    ///   - [Balance::min, Balance::max]
    ///
    /// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
    ///   - Setting it to `0` will essentially disable the weight fee.
    ///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
    pub struct WeightToFee;
    impl WeightToFeePolynomial for WeightToFee {
        type Balance = Balance;
        fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
            // In SherpaX, extrinsic base weight (smallest non-zero weight) is mapped to 1/25 KSX:
            let p = super::currency::UNITS;
            let q = 25 * Balance::from(ExtrinsicBaseWeight::get());
            smallvec![WeightToFeeCoefficient {
                degree: 1,
                negative: false,
                coeff_frac: Perbill::from_rational(p % q, q),
                coeff_integer: p / q,
            }]
        }
    }
}

pub mod time {
    use chainx_primitives::{BlockNumber, Moment};

    pub const MILLISECS_PER_BLOCK: Moment = 6000;
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 7 * DAYS;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;

    // 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}
