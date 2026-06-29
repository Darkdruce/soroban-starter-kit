// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug)]
pub enum AuctionError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AuctionEnded = 3,
    AuctionNotEnded = 4,
    BidTooLow = 5,
    AlreadyEnded = 6,
    NoBids = 7,
    NotAuthorized = 8,
    InvalidAmount = 9,
    InvalidDeadline = 10,
    NothingToWithdraw = 11,
}

impl core::fmt::Display for AuctionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AuctionError::AlreadyInitialized => write!(f, "already initialized"),
            AuctionError::NotInitialized => write!(f, "not initialized"),
            AuctionError::AuctionEnded => write!(f, "auction has ended"),
            AuctionError::AuctionNotEnded => write!(f, "auction has not ended"),
            AuctionError::BidTooLow => write!(f, "bid too low"),
            AuctionError::AlreadyEnded => write!(f, "auction already settled"),
            AuctionError::NoBids => write!(f, "no bids placed"),
            AuctionError::NotAuthorized => write!(f, "not authorized"),
            AuctionError::InvalidAmount => write!(f, "invalid amount"),
            AuctionError::InvalidDeadline => write!(f, "invalid deadline"),
            AuctionError::NothingToWithdraw => write!(f, "nothing to withdraw"),
        }
    }
}
