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

#[cfg(test)]
mod tests {
    extern crate std;

    use super::StakingError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)]
    fn render_error_code_snapshot() -> String {
        format!(
            "\
StakingError::AlreadyInitialized = {}\n\
StakingError::NotInitialized = {}\n\
StakingError::Unauthorized = {}\n\
StakingError::InvalidAmount = {}\n\
StakingError::NoStake = {}\n\
StakingError::InsufficientStake = {}\n\
StakingError::NoRewards = {}\n\
StakingError::CompoundTokenMismatch = {}\n",
            StakingError::AlreadyInitialized as u32,
            StakingError::NotInitialized as u32,
            StakingError::Unauthorized as u32,
            StakingError::InvalidAmount as u32,
            StakingError::NoStake as u32,
            StakingError::InsufficientStake as u32,
            StakingError::NoRewards as u32,
            StakingError::CompoundTokenMismatch as u32,
        )
    }

    #[test]
    fn staking_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
