// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BondingCurveError {
    /// `initialize` was called on an already-initialized contract.
    AlreadyInitialized = 1,
    /// An operation was attempted before the contract was initialized.
    NotInitialized = 2,
    /// Caller is not the admin.
    Unauthorized = 3,
    /// Amount is zero or negative.
    InvalidAmount = 4,
    /// Insufficient reserve to sell.
    InsufficientReserve = 5,
    /// Arithmetic overflow.
    Overflow = 6,
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::BondingCurveError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)]
    fn render_error_code_snapshot() -> String {
        format!(
            "\
BondingCurveError::AlreadyInitialized = {}\n\
BondingCurveError::NotInitialized = {}\n\
BondingCurveError::Unauthorized = {}\n\
BondingCurveError::InvalidAmount = {}\n\
BondingCurveError::InsufficientReserve = {}\n\
BondingCurveError::Overflow = {}\n",
            BondingCurveError::AlreadyInitialized as u32,
            BondingCurveError::NotInitialized as u32,
            BondingCurveError::Unauthorized as u32,
            BondingCurveError::InvalidAmount as u32,
            BondingCurveError::InsufficientReserve as u32,
            BondingCurveError::Overflow as u32,
        )
    }

    #[test]
    fn bonding_curve_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
