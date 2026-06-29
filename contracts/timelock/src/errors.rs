use soroban_common::impl_display_error;
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

impl_display_error!(
    TimelockError,
    NotAuthorized       => "not authorized",
    AlreadyInitialized  => "already initialized",
    NotInitialized      => "not initialized",
    NotYetReleasable    => "not yet releasable",
    AlreadyReleased     => "already released",
    AlreadyCancelled    => "already cancelled",
    InvalidAmount       => "invalid amount",
    InvalidReleaseLedger => "invalid release ledger",
);
