// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AirdropError {
    /// `initialize` called on an already-initialized contract.
    AlreadyInitialized = 1,
    /// Operation attempted before the contract was initialized.
    NotInitialized = 2,
    /// Caller is not the admin.
    Unauthorized = 3,
    /// Merkle root has not been set yet.
    RootNotSet = 4,
    /// The provided merkle proof is invalid.
    InvalidProof = 5,
    /// This address has already claimed their airdrop.
    AlreadyClaimed = 6,
    /// Claim amount is zero.
    InvalidAmount = 7,
    /// The claim deadline has passed; no further claims are accepted.
    ClaimWindowClosed = 8,
}
