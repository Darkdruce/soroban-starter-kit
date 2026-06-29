use soroban_common::impl_display_error;
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

impl_display_error!(
    SwapError,
    NotAuthorized    => "not authorized",
    SwapNotFound     => "swap not found",
    InvalidState     => "invalid swap state",
    DeadlineExpired  => "swap deadline has expired",
    InvalidAmount    => "invalid amount",
    InvalidDeadline  => "invalid deadline",
    AlreadyCompleted => "swap already completed",
    AlreadyCancelled => "swap already cancelled",
);
