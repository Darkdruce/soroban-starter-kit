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
    /// The admin-configured set of authorized publishers (persistent).
    Publishers,
    /// The latest submission from a given publisher (persistent).
    Submission(Address),
    /// Ring buffer of the last `N` price observations, oldest first (instance).
    History,
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

/// A single publisher's latest price submission.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublisherSubmission {
    /// The submitted price.
    pub price: i128,
    /// The unix timestamp at which the price was submitted.
    pub timestamp: u64,
}

/// A single historical price observation, used to compute a TWAP.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriceObservation {
    /// The recorded price.
    pub price: i128,
    /// The unix timestamp at which the price was recorded.
    pub timestamp: u64,
}
