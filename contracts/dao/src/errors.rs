// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

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

impl core::fmt::Display for DaoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DaoError::NotAuthorized => write!(f, "not authorized"),
            DaoError::AlreadyInitialized => write!(f, "already initialized"),
            DaoError::NotInitialized => write!(f, "not initialized"),
            DaoError::ProposalNotFound => write!(f, "proposal not found"),
            DaoError::InvalidState => write!(f, "invalid proposal state"),
            DaoError::DeadlineNotReached => write!(f, "voting deadline not yet reached"),
            DaoError::AlreadyVoted => write!(f, "already voted on this proposal"),
            DaoError::QuorumNotMet => write!(f, "quorum not met"),
            DaoError::ProposalRejected => write!(f, "proposal rejected by majority"),
            DaoError::InsufficientVotingPower => write!(f, "insufficient voting power"),
        }
    }
}
