// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, contracttype};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Beneficiary address.
    Beneficiary,
    /// Token contract address.
    Token,
    /// Ledger sequence at which vesting begins (cliff).
    CliffLedger,
    /// Ledger sequence at which all tokens are fully vested.
    EndLedger,
    /// Total tokens to vest.
    Amount,
    /// Tokens already claimed by the beneficiary.
    Claimed,
    /// Whether the schedule has been revoked by admin.
    Revoked,
    /// Admin address.
    Admin,
    /// Total tokens released early by admin (audit log).
    AdminReleased,
    /// Contract version number (`u32`).
    Version,
}

/// Snapshot returned by `get_info`.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VestingInfo {
    pub beneficiary: Address,
    pub token: Address,
    pub cliff_ledger: u32,
    pub end_ledger: u32,
    pub amount: i128,
    pub claimed: i128,
    pub revoked: bool,
}
