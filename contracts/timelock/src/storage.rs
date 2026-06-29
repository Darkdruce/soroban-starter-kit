// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, contracttype};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    Beneficiary,
    ReleaseLedger,
    Amount,
    State,
}

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimelockState {
    Active = 0,
    Released = 1,
    Cancelled = 2,
}

impl core::fmt::Display for TimelockState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            TimelockState::Active => "active",
            TimelockState::Released => "released",
            TimelockState::Cancelled => "cancelled",
        })
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TimelockInfo {
    pub admin: Address,
    pub token: Address,
    pub beneficiary: Address,
    pub release_ledger: u32,
    pub amount: i128,
    pub state: TimelockState,
}
