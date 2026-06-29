// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug)]
pub enum TimelockError {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    NotYetReleasable = 4,
    AlreadyReleased = 5,
    AlreadyCancelled = 6,
    InvalidAmount = 7,
    InvalidReleaseLedger = 8,
}

impl core::fmt::Display for TimelockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TimelockError::NotAuthorized => write!(f, "not authorized"),
            TimelockError::AlreadyInitialized => write!(f, "already initialized"),
            TimelockError::NotInitialized => write!(f, "not initialized"),
            TimelockError::NotYetReleasable => write!(f, "not yet releasable"),
            TimelockError::AlreadyReleased => write!(f, "already released"),
            TimelockError::AlreadyCancelled => write!(f, "already cancelled"),
            TimelockError::InvalidAmount => write!(f, "invalid amount"),
            TimelockError::InvalidReleaseLedger => write!(f, "invalid release ledger"),
        }
    }
}
