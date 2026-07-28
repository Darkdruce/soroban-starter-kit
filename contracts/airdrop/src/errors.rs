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

#[cfg(test)]
mod tests {
    extern crate std;

    use super::AirdropError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)]
    fn render_error_code_snapshot() -> String {
        format!(
            "\
AirdropError::AlreadyInitialized = {}\n\
AirdropError::NotInitialized = {}\n\
AirdropError::Unauthorized = {}\n\
AirdropError::RootNotSet = {}\n\
AirdropError::InvalidProof = {}\n\
AirdropError::AlreadyClaimed = {}\n\
AirdropError::InvalidAmount = {}\n\
AirdropError::ClaimWindowClosed = {}\n",
            AirdropError::AlreadyInitialized as u32,
            AirdropError::NotInitialized as u32,
            AirdropError::Unauthorized as u32,
            AirdropError::RootNotSet as u32,
            AirdropError::InvalidProof as u32,
            AirdropError::AlreadyClaimed as u32,
            AirdropError::InvalidAmount as u32,
            AirdropError::ClaimWindowClosed as u32,
        )
    }

    #[test]
    fn airdrop_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
