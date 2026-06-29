// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, Symbol, Val, Vec, contracttype};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Signers,
    Threshold,
    NextTransactionId,
    Transaction(u64),
    Paused,
    Version,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    pub id: u64,
    pub proposer: Address,
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
    pub signatures: Vec<Address>,
    pub executed: bool,
}
