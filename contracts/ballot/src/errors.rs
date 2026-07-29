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
    /// Invalid vote choice index (out of range).
    InvalidChoice = 6,
    /// Voting has not started, is closed, or has not reached its start ledger yet.
    VotingClosed = 7,
    /// `deregister_voter` was called after at least one vote has been cast.
    VotingAlreadyStarted = 8,
    /// Voting window is invalid: start_ledger >= end_ledger or window is in the past.
    InvalidWindow = 9,
    /// Current ledger is before voting_start.
    VotingNotStarted = 10,
    /// `initialize` was called with an empty choices list.
    NoChoices = 11,
}
