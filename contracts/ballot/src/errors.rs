// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BallotError {
    /// `initialize` was called on an already-initialized contract.
    AlreadyInitialized = 1,
    /// An operation was attempted before the contract was initialized.
    NotInitialized = 2,
    /// Caller is not the admin.
    Unauthorized = 3,
    /// Voter is not registered.
    NotRegistered = 4,
    /// Voter has already voted.
    AlreadyVoted = 5,
    /// Invalid vote choice.
    InvalidChoice = 6,
    /// Voting has not started, is closed, or has not reached its start ledger yet.
    VotingClosed = 7,
    /// `deregister_voter` was called after at least one vote has been cast.
    VotingAlreadyStarted = 8,
    /// Voting window is invalid: start_ledger >= end_ledger or window is in the past.
    InvalidWindow = 9,
    /// Current ledger is before voting_start.
    VotingNotStarted = 10,
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::BallotError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)]
    fn render_error_code_snapshot() -> String {
        format!(
            "\
BallotError::AlreadyInitialized = {}\n\
BallotError::NotInitialized = {}\n\
BallotError::Unauthorized = {}\n\
BallotError::NotRegistered = {}\n\
BallotError::AlreadyVoted = {}\n\
BallotError::InvalidChoice = {}\n\
BallotError::VotingClosed = {}\n\
BallotError::VotingAlreadyStarted = {}\n\
BallotError::InvalidWindow = {}\n\
BallotError::VotingNotStarted = {}\n",
            BallotError::AlreadyInitialized as u32,
            BallotError::NotInitialized as u32,
            BallotError::Unauthorized as u32,
            BallotError::NotRegistered as u32,
            BallotError::AlreadyVoted as u32,
            BallotError::InvalidChoice as u32,
            BallotError::VotingClosed as u32,
            BallotError::VotingAlreadyStarted as u32,
            BallotError::InvalidWindow as u32,
            BallotError::VotingNotStarted as u32,
        )
    }

    #[test]
    fn ballot_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
