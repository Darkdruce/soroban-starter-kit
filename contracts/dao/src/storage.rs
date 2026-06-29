// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, String, contracttype};

/// Instance-storage keys (contract-level state).
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    VotingPeriod,
    Quorum,
    ProposalCount,
    Initialized,
}

/// Persistent-storage keys (per-proposal and per-vote data).
#[contracttype]
#[derive(Clone)]
pub enum ProposalKey {
    Proposal(u32),
}

/// Composite key for vote deduplication.
#[contracttype]
#[derive(Clone)]
pub struct VoteKey {
    pub proposal_id: u32,
    pub voter: Address,
}

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProposalState {
    Active = 0,
    Executed = 1,
    Cancelled = 2,
}

impl core::fmt::Display for ProposalState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            ProposalState::Active => "active",
            ProposalState::Executed => "executed",
            ProposalState::Cancelled => "cancelled",
        })
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub deadline: u32,
    pub yes_votes: i128,
    pub no_votes: i128,
    pub state: ProposalState,
}
