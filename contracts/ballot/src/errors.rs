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
    /// Voting has not started or is closed.
    VotingClosed = 7,
}
