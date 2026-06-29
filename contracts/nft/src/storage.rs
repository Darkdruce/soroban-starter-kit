// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{String, contracttype};

/// Instance-storage keys (shared TTL for contract-level data).
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Name,
    Symbol,
    TotalSupply,
    MaxSupply,
    Initialized,
}

/// Persistent-storage keys (per-key TTL for per-token data).
#[contracttype]
#[derive(Clone)]
pub enum TokenKey {
    Owner(u32),
    Approval(u32),
    Uri(u32),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub token_uri: String,
}
