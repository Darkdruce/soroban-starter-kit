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

#[cfg(test)]
mod tests {
    extern crate std;

    use super::MarketplaceError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)]
    fn render_error_code_snapshot() -> String {
        format!(
            "\
MarketplaceError::AlreadyInitialized = {}\n\
MarketplaceError::NotInitialized = {}\n\
MarketplaceError::NotAuthorized = {}\n\
MarketplaceError::InvalidPrice = {}\n\
MarketplaceError::ListingNotFound = {}\n\
MarketplaceError::ListingInactive = {}\n\
MarketplaceError::InvalidRoyalty = {}\n\
MarketplaceError::InvalidExpiry = {}\n\
MarketplaceError::ListingExpired = {}\n\
MarketplaceError::ListingNotExpired = {}\n\
MarketplaceError::InvalidOfferAmount = {}\n\
MarketplaceError::OfferNotFound = {}\n",
            MarketplaceError::AlreadyInitialized as u32,
            MarketplaceError::NotInitialized as u32,
            MarketplaceError::NotAuthorized as u32,
            MarketplaceError::InvalidPrice as u32,
            MarketplaceError::ListingNotFound as u32,
            MarketplaceError::ListingInactive as u32,
            MarketplaceError::InvalidRoyalty as u32,
            MarketplaceError::InvalidExpiry as u32,
            MarketplaceError::ListingExpired as u32,
            MarketplaceError::ListingNotExpired as u32,
            MarketplaceError::InvalidOfferAmount as u32,
            MarketplaceError::OfferNotFound as u32,
        )
    }

    #[test]
    fn marketplace_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
