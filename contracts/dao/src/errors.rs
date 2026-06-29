use soroban_common::impl_display_error;
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug)]
pub enum DaoError {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    ProposalNotFound = 4,
    InvalidState = 5,
    DeadlineNotReached = 6,
    AlreadyVoted = 7,
    QuorumNotMet = 8,
    ProposalRejected = 9,
    InsufficientVotingPower = 10,
}

impl_display_error!(
    DaoError,
    NotAuthorized           => "not authorized",
    AlreadyInitialized      => "already initialized",
    NotInitialized          => "not initialized",
    ProposalNotFound        => "proposal not found",
    InvalidState            => "invalid proposal state",
    DeadlineNotReached      => "voting deadline not yet reached",
    AlreadyVoted            => "already voted on this proposal",
    QuorumNotMet            => "quorum not met",
    ProposalRejected        => "proposal rejected by majority",
    InsufficientVotingPower => "insufficient voting power",
);
