// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StakingError {
    /// `initialize` was called on an already-initialized contract.
    AlreadyInitialized = 1,
    /// An operation was attempted before the contract was initialized.
    NotInitialized = 2,
    /// Caller is not the admin.
    Unauthorized = 3,
    /// Amount is zero or negative.
    InvalidAmount = 4,
    /// Staker has no stake to unstake or claim from.
    NoStake = 5,
    /// Requested unstake amount exceeds the staker's current stake.
    InsufficientStake = 6,
    /// No rewards are available to claim.
    NoRewards = 7,
    /// Stake and reward token must be the same to compound.
    CompoundTokenMismatch = 8,
}
