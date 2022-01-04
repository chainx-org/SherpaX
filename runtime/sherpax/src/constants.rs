pub use currency::*;
pub use fee::*;

pub mod currency {
    use crate::Balance;

    /// The existential deposit = 10_000_000_000.
    pub const EXISTENTIAL_DEPOSIT: Balance = MILLICENTS / 100;

    /// 1 KSX = 1 UNITS
    pub const UNITS: Balance = 1_000_000_000_000_000_000;
    pub const CENTS: Balance = UNITS / 1_000;
    pub const MILLICENTS: Balance = CENTS / 1_000;

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        (items as Balance) * 100 * CENTS + (bytes as Balance) * CENTS
    }
}

/// Fee-related.
pub mod fee {
    use crate::Balance;
    use frame_support::weights::{
        constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    };
    use smallvec::smallvec;
    pub use sp_runtime::Perbill;

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
