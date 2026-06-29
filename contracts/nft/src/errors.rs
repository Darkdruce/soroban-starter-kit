use soroban_common::impl_display_error;
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

impl_display_error!(
    NftError,
    NotAuthorized      => "not authorized",
    AlreadyInitialized => "already initialized",
    NotInitialized     => "not initialized",
    TokenNotFound      => "token not found",
    TokenAlreadyMinted => "token already minted",
    NotOwner           => "not the token owner",
    NotApproved        => "not approved for this token",
    SupplyCapReached   => "supply cap reached",
    InvalidTokenId     => "invalid token id",
);
