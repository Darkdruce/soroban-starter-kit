use soroban_common::impl_display_error;
// #[contracterror] generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

/// Error codes returned by [`EscrowContract`](crate::EscrowContract) methods.
///
/// Each variant maps to a unique `u32` discriminant embedded in the contract ABI.
///
/// # Examples
///
/// ```ignore
/// match escrow_client.try_fund() {
///     Err(Ok(EscrowError::InvalidState)) => { /* wrong lifecycle state */ }
///     _ => {}
/// }
/// ```
#[contracterror]
#[derive(Clone, Copy, Debug)]
pub enum EscrowError {
    /// The caller is not permitted to perform this action.
    NotAuthorized = 1,
    /// The escrow is not in the required lifecycle state for this operation.
    InvalidState = 2,
    /// The operation cannot proceed because the deadline has already passed.
    DeadlinePassed = 3,
    /// A refund was requested but the deadline has not yet been reached.
    DeadlineNotReached = 4,
    /// [`EscrowContract::initialize`](crate::EscrowContract::initialize) was
    /// called on an already-initialized contract.
    AlreadyInitialized = 5,
    /// An operation was attempted before the contract was initialized.
    NotInitialized = 6,
    /// The requested partial-release amount exceeds the escrowed balance.
    InsufficientFunds = 7,
    /// The escrow amount must be greater than zero.
    InvalidAmount = 8,
    InvalidParties = 9,
}

/// Converts the unit error returned by `soroban_common::validate_deadline`
/// into [`EscrowError::DeadlinePassed`].
///
/// `validate_deadline` is generic over `E: From<()>` so that callers do not
/// need to map the error manually. This impl satisfies that bound for
/// `EscrowError`. It cannot be expressed with `#[derive]`.
impl From<()> for EscrowError {
    fn from(_: ()) -> Self {
        Self::DeadlinePassed
    }
}

impl_display_error!(
    EscrowError,
    NotAuthorized      => "not authorized",
    InvalidState       => "invalid state",
    DeadlinePassed     => "deadline passed",
    DeadlineNotReached => "deadline not reached",
    AlreadyInitialized => "already initialized",
    NotInitialized     => "not initialized",
    InsufficientFunds  => "insufficient funds",
    InvalidAmount      => "invalid amount",
    InvalidParties     => "invalid parties",
);

#[cfg(test)]
mod tests {
    extern crate std;

    use super::EscrowError;
    use std::format;
    use std::string::String;

    #[allow(clippy::as_conversions)] // reading enum discriminants for snapshot verification
    fn render_error_code_snapshot() -> String {
        format!(
            "\
EscrowError::NotAuthorized = {}\n\
EscrowError::InvalidState = {}\n\
EscrowError::DeadlinePassed = {}\n\
EscrowError::DeadlineNotReached = {}\n\
EscrowError::AlreadyInitialized = {}\n\
EscrowError::NotInitialized = {}\n\
EscrowError::InsufficientFunds = {}\n\
EscrowError::InvalidAmount = {}\n\
EscrowError::InvalidParties = {}\n",
            EscrowError::NotAuthorized as u32,
            EscrowError::InvalidState as u32,
            EscrowError::DeadlinePassed as u32,
            EscrowError::DeadlineNotReached as u32,
            EscrowError::AlreadyInitialized as u32,
            EscrowError::NotInitialized as u32,
            EscrowError::InsufficientFunds as u32,
            EscrowError::InvalidAmount as u32,
            EscrowError::InvalidParties as u32,
        )
    }

    #[test]
    fn escrow_error_codes_match_snapshot() {
        assert_eq!(
            render_error_code_snapshot(),
            include_str!("../snapshots/error_codes.snap")
        );
    }
}
