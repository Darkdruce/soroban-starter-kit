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

#[cfg(test)]
mod tests {
    extern crate std;

    use super::VestingError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)]
    fn render_error_code_snapshot() -> String {
        format!(
            "\
VestingError::AlreadyInitialized = {}\n\
VestingError::NotInitialized = {}\n\
VestingError::Unauthorized = {}\n\
VestingError::InvalidAmount = {}\n\
VestingError::InvalidSchedule = {}\n\
VestingError::NothingToClaim = {}\n\
VestingError::AlreadyRevoked = {}\n\
VestingError::CliffAlreadyPassed = {}\n",
            VestingError::AlreadyInitialized as u32,
            VestingError::NotInitialized as u32,
            VestingError::Unauthorized as u32,
            VestingError::InvalidAmount as u32,
            VestingError::InvalidSchedule as u32,
            VestingError::NothingToClaim as u32,
            VestingError::AlreadyRevoked as u32,
            VestingError::CliffAlreadyPassed as u32,
        )
    }

    #[test]
    fn vesting_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
