// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, contracttype};

/// Top-level storage keys used by [`SubscriptionContract`](crate::SubscriptionContract).
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The service provider's [`Address`] (instance storage).
    Provider,
    /// The payment token contract [`Address`] (instance storage).
    Token,
    /// Per-subscriber [`SubscriptionInfo`] (persistent storage).
    Subscription(Address),
}

/// Subscription configuration and state for a single subscriber.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct SubscriptionInfo {
    /// Amount of tokens charged per interval.
    pub amount: i128,
    /// Number of ledgers between each charge.
    pub interval_ledgers: u32,
    /// Ledger sequence number of the last successful charge (or subscription start).
    pub last_charged_ledger: u32,
    /// Whether the subscription is currently active.
    pub active: bool,
}
