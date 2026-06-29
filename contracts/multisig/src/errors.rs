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
}

impl core::fmt::Display for MultisigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MultisigError::AlreadyInitialized => write!(f, "already initialized"),
            MultisigError::NotInitialized => write!(f, "not initialized"),
            MultisigError::InvalidThreshold => write!(f, "invalid threshold"),
            MultisigError::InvalidSigners => write!(f, "invalid signers"),
            MultisigError::NotSigner => write!(f, "not signer"),
            MultisigError::TransactionNotFound => write!(f, "transaction not found"),
            MultisigError::AlreadyExecuted => write!(f, "already executed"),
            MultisigError::AlreadySigned => write!(f, "already signed"),
            MultisigError::ThresholdNotMet => write!(f, "threshold not met"),
            MultisigError::InsufficientApprovals => write!(f, "insufficient approvals"),
        }
    }
}
