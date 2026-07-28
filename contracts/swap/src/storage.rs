// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, contracttype};

/// Instance-storage keys.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    SwapCount,
    Initialized,
    Admin,
    Treasury,
    FeeBps,
}

/// Persistent-storage key for individual swaps.
#[contracttype]
#[derive(Clone)]
pub enum SwapKey {
    Swap(u32),
}

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SwapState {
    Open = 0,
    Completed = 1,
    Cancelled = 2,
}

impl core::fmt::Display for SwapState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            SwapState::Open => "open",
            SwapState::Completed => "completed",
            SwapState::Cancelled => "cancelled",
        })
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapInfo {
    pub id: u32,
    pub party_a: Address,
    pub token_a: Address,
    pub amount_a: i128,
    pub token_b: Address,
    pub amount_b: i128,
    pub expires_at: u32,
    pub state: SwapState,
}