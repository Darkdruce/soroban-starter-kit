// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarketplaceError {
    /// `initialize` called on an already-initialized contract.
    AlreadyInitialized = 1,
    /// Operation attempted before the contract was initialized.
    NotInitialized = 2,
    /// Caller is not authorized for this operation.
    NotAuthorized = 3,
    /// Price is zero or negative.
    InvalidPrice = 4,
    /// Listing ID does not exist.
    ListingNotFound = 5,
    /// Listing is no longer active (already bought or cancelled).
    ListingInactive = 6,
    /// Royalty basis points exceed 10 000 (100 %).
    InvalidRoyalty = 7,
    /// The provided expiry ledger sequence is not in the future.
    InvalidExpiry = 8,
    /// The listing's expiry ledger sequence has already passed.
    ListingExpired = 9,
    /// `sweep_expired` called on a listing that has no expiry, or whose expiry hasn't passed.
    ListingNotExpired = 10,
    /// Offer amount is zero, negative, or not below the listing price.
    InvalidOfferAmount = 11,
    /// No offer exists for this (listing, buyer) pair.
    OfferNotFound = 12,
}
