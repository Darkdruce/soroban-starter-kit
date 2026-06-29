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
