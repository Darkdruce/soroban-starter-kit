// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug)]
pub enum SwapError {
    NotAuthorized = 1,
    SwapNotFound = 2,
    InvalidState = 3,
    DeadlineExpired = 4,
    InvalidAmount = 5,
    InvalidDeadline = 6,
    AlreadyCompleted = 7,
    AlreadyCancelled = 8,
}

impl core::fmt::Display for SwapError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SwapError::NotAuthorized => write!(f, "not authorized"),
            SwapError::SwapNotFound => write!(f, "swap not found"),
            SwapError::InvalidState => write!(f, "invalid swap state"),
            SwapError::DeadlineExpired => write!(f, "swap deadline has expired"),
            SwapError::InvalidAmount => write!(f, "invalid amount"),
            SwapError::InvalidDeadline => write!(f, "invalid deadline"),
            SwapError::AlreadyCompleted => write!(f, "swap already completed"),
            SwapError::AlreadyCancelled => write!(f, "swap already cancelled"),
        }
    }
}
