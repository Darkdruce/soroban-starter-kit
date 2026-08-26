// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

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
    /// Proposer self-cancel rejected because votes have already been cast.
    VotesAlreadyCast = 11,
    /// `quorum_bps` must be in the range [0, 10_000].
    InvalidQuorumBps = 12,
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
    VotesAlreadyCast        => "votes have already been cast; proposer cannot cancel",
    InvalidQuorumBps        => "quorum_bps must be between 0 and 10_000",
);

#[cfg(test)]
mod tests {
    extern crate std;

    use super::DaoError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)]
    fn render_error_code_snapshot() -> String {
        format!(
            "\
DaoError::NotAuthorized = {}\n\
DaoError::AlreadyInitialized = {}\n\
DaoError::NotInitialized = {}\n\
DaoError::ProposalNotFound = {}\n\
DaoError::InvalidState = {}\n\
DaoError::DeadlineNotReached = {}\n\
DaoError::AlreadyVoted = {}\n\
DaoError::QuorumNotMet = {}\n\
DaoError::ProposalRejected = {}\n\
DaoError::InsufficientVotingPower = {}\n\
DaoError::VotesAlreadyCast = {}\n\
DaoError::InvalidQuorumBps = {}\n",
            DaoError::NotAuthorized as u32,
            DaoError::AlreadyInitialized as u32,
            DaoError::NotInitialized as u32,
            DaoError::ProposalNotFound as u32,
            DaoError::InvalidState as u32,
            DaoError::DeadlineNotReached as u32,
            DaoError::AlreadyVoted as u32,
            DaoError::QuorumNotMet as u32,
            DaoError::ProposalRejected as u32,
            DaoError::InsufficientVotingPower as u32,
            DaoError::VotesAlreadyCast as u32,
            DaoError::InvalidQuorumBps as u32,
        )
    }

    #[test]
    fn dao_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
