// `#[contracterror]` generates undocumented public associated items; allow
// missing_docs for this module. The variants themselves are still documented.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

/// Errors returned by the oracle contract entry points.
#[contracterror]
#[derive(Clone, Copy, Debug)]
pub enum OracleError {
    /// `initialize` was called on an already-initialized contract.
    AlreadyInitialized = 1,
    /// An entry point was called before `initialize`.
    NotInitialized = 2,
    /// The caller is not the configured admin.
    Unauthorized = 3,
    /// The stored price is older than the staleness threshold.
    StalePrice = 4,
    /// The provided staleness threshold is zero.
    InvalidStalenessThreshold = 5,
    /// No authorized publisher has submitted a price yet.
    NoPublisherData = 6,
    /// No price observations fall within the requested TWAP window.
    InsufficientHistory = 7,
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::OracleError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)]
    fn render_error_code_snapshot() -> String {
        format!(
            "\
OracleError::AlreadyInitialized = {}\n\
OracleError::NotInitialized = {}\n\
OracleError::Unauthorized = {}\n\
OracleError::StalePrice = {}\n\
OracleError::InvalidStalenessThreshold = {}\n\
OracleError::NoPublisherData = {}\n\
OracleError::InsufficientHistory = {}\n",
            OracleError::AlreadyInitialized as u32,
            OracleError::NotInitialized as u32,
            OracleError::Unauthorized as u32,
            OracleError::StalePrice as u32,
            OracleError::InvalidStalenessThreshold as u32,
            OracleError::NoPublisherData as u32,
            OracleError::InsufficientHistory as u32,
        )
    }

    #[test]
    fn oracle_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
