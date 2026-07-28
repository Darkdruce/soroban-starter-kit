// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WrappedTokenError {
    /// `initialize` was called on an already-initialized contract.
    AlreadyInitialized = 1,
    /// An operation was attempted before the contract was initialized.
    NotInitialized = 2,
    /// Caller is not the admin.
    Unauthorized = 3,
    /// Amount is zero or negative.
    InvalidAmount = 4,
    /// Insufficient wrapped token balance to burn.
    InsufficientBalance = 5,
    /// Insufficient XLM in reserve to unwrap.
    InsufficientReserve = 6,
    /// Wrapping this amount would exceed the per-address cap set at `initialize`.
    MaxWrapExceeded = 7,
}
