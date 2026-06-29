// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::Address;

#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    VotingActive,
    RegisteredVoter(Address),
    Voter(Address),
    YesVotes,
    NoVotes,
}
