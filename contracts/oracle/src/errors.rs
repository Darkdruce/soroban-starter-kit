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
}
