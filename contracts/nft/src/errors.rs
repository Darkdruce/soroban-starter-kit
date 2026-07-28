use soroban_common::impl_display_error;
use soroban_sdk::contracterror;

// `#[contracterror]` generates undocumented public associated items.
#[allow(missing_docs)]
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

#[cfg(test)]
mod tests {
    extern crate std;

    use super::NftError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)]
    fn render_error_code_snapshot() -> String {
        format!(
            "\
NftError::NotAuthorized = {}\n\
NftError::AlreadyInitialized = {}\n\
NftError::NotInitialized = {}\n\
NftError::TokenNotFound = {}\n\
NftError::TokenAlreadyMinted = {}\n\
NftError::NotOwner = {}\n\
NftError::NotApproved = {}\n\
NftError::SupplyCapReached = {}\n\
NftError::InvalidTokenId = {}\n",
            NftError::NotAuthorized as u32,
            NftError::AlreadyInitialized as u32,
            NftError::NotInitialized as u32,
            NftError::TokenNotFound as u32,
            NftError::TokenAlreadyMinted as u32,
            NftError::NotOwner as u32,
            NftError::NotApproved as u32,
            NftError::SupplyCapReached as u32,
            NftError::InvalidTokenId as u32,
        )
    }

    #[test]
    fn nft_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
