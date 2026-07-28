use soroban_common::impl_display_error;
// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

/// Error codes returned by [`MultisigContract`](crate::MultisigContract).
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MultisigError {
    /// The contract has already been initialized.
    AlreadyInitialized = 1,
    /// The contract has not been initialized.
    NotInitialized = 2,
    /// The threshold must be greater than zero and no greater than signer count.
    InvalidThreshold = 3,
    /// Signer lists cannot be empty or contain duplicates.
    InvalidSigners = 4,
    /// The caller or approver is not a signer.
    NotSigner = 5,
    /// The transaction does not exist.
    TransactionNotFound = 6,
    /// The transaction has already been executed.
    AlreadyExecuted = 7,
    /// The signer has already signed the transaction.
    AlreadySigned = 8,
    /// The transaction has too few signatures to execute.
    ThresholdNotMet = 9,
    /// Signer-management approval list does not satisfy the threshold.
    InsufficientApprovals = 10,
    /// The proposal has expired and can no longer be signed or executed.
    ProposalExpired = 11,
}

impl_display_error!(
    MultisigError,
    AlreadyInitialized  => "already initialized",
    NotInitialized      => "not initialized",
    InvalidThreshold    => "invalid threshold",
    InvalidSigners      => "invalid signers",
    NotSigner           => "not signer",
    TransactionNotFound => "transaction not found",
    AlreadyExecuted     => "already executed",
    AlreadySigned       => "already signed",
    ThresholdNotMet     => "threshold not met",
    InsufficientApprovals => "insufficient approvals",
    ProposalExpired     => "proposal expired",
);

#[cfg(test)]
mod tests {
    extern crate std;

    use super::MultisigError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)]
    fn render_error_code_snapshot() -> String {
        format!(
            "\
MultisigError::AlreadyInitialized = {}\n\
MultisigError::NotInitialized = {}\n\
MultisigError::InvalidThreshold = {}\n\
MultisigError::InvalidSigners = {}\n\
MultisigError::NotSigner = {}\n\
MultisigError::TransactionNotFound = {}\n\
MultisigError::AlreadyExecuted = {}\n\
MultisigError::AlreadySigned = {}\n\
MultisigError::ThresholdNotMet = {}\n\
MultisigError::InsufficientApprovals = {}\n\
MultisigError::ProposalExpired = {}\n",
            MultisigError::AlreadyInitialized as u32,
            MultisigError::NotInitialized as u32,
            MultisigError::InvalidThreshold as u32,
            MultisigError::InvalidSigners as u32,
            MultisigError::NotSigner as u32,
            MultisigError::TransactionNotFound as u32,
            MultisigError::AlreadyExecuted as u32,
            MultisigError::AlreadySigned as u32,
            MultisigError::ThresholdNotMet as u32,
            MultisigError::InsufficientApprovals as u32,
            MultisigError::ProposalExpired as u32,
        )
    }

    #[test]
    fn multisig_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
