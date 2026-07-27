// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, contracttype};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    VotingActive,
    RegisteredVoter(Address),
    Voter(Address),
    YesVotes,
    NoVotes,
    /// First ledger sequence at which voting is open (inclusive).
    VotingStart,
    /// Last ledger sequence at which voting is open (inclusive).
    VotingEnd,
    /// Running count of total votes cast; used to gate `deregister_voter`.
    TotalVotes,
}
