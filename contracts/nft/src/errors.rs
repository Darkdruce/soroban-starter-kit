// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug)]
pub enum NftError {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    TokenNotFound = 4,
    TokenAlreadyMinted = 5,
    NotOwner = 6,
    NotApproved = 7,
    SupplyCapReached = 8,
    InvalidTokenId = 9,
}

impl core::fmt::Display for NftError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NftError::NotAuthorized => write!(f, "not authorized"),
            NftError::AlreadyInitialized => write!(f, "already initialized"),
            NftError::NotInitialized => write!(f, "not initialized"),
            NftError::TokenNotFound => write!(f, "token not found"),
            NftError::TokenAlreadyMinted => write!(f, "token already minted"),
            NftError::NotOwner => write!(f, "not the token owner"),
            NftError::NotApproved => write!(f, "not approved for this token"),
            NftError::SupplyCapReached => write!(f, "supply cap reached"),
            NftError::InvalidTokenId => write!(f, "invalid token id"),
        }
    }
}
