// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, contracttype};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Creator,
    Token,
    Goal,
    Deadline,
    TotalPledged,
    Claimed,
    Pledge(Address),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CrowdfundInfo {
    pub creator: Address,
    pub token: Address,
    pub goal: i128,
    pub deadline: u32,
    pub total_pledged: i128,
    pub claimed: bool,
}
