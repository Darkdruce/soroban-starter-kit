// `#[contracttype]` generates undocumented public associated items; allow
// missing_docs for this module. The types and fields are still documented.
#![allow(missing_docs)]

use soroban_sdk::{Address, contracttype};

/// Instance-storage keys for the oracle contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The admin address authorized to push price updates.
    Admin,
    /// The most recently published price.
    Price,
    /// The ledger sequence at which the price was last updated.
    UpdatedAt,
    /// The maximum age, in ledgers, before a price is considered stale.
    StalenessThreshold,
    /// The unix timestamp at which the price was last updated.
    UpdatedAtTimestamp,
}

/// Snapshot of the oracle state returned by read entry points.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriceData {
    /// The published price.
    pub price: i128,
    /// The ledger sequence at which `price` was set.
    pub updated_at: u32,
    /// The admin address that published the price.
    pub admin: Address,
    /// The configured staleness threshold, in ledgers.
    pub staleness_threshold: u32,
}
