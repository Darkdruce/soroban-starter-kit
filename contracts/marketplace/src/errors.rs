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
}
