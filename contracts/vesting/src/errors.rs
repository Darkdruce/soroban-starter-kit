// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VestingError {
    /// `initialize` was called on an already-initialized contract.
    AlreadyInitialized = 1,
    /// An operation was attempted before the contract was initialized.
    NotInitialized = 2,
    /// Caller is not the admin.
    Unauthorized = 3,
    /// Amount is zero or negative.
    InvalidAmount = 4,
    /// `cliff_ledger` >= `end_ledger`, or `end_ledger` <= current ledger.
    InvalidSchedule = 5,
    /// No tokens are currently vested and unclaimed.
    NothingToClaim = 6,
    /// The vesting schedule has already been revoked.
    AlreadyRevoked = 7,
    /// admin_release was called after the cliff has already passed.
    CliffAlreadyPassed = 8,
}
