// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_common::impl_display_error;
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
    /// The auction ended but the highest bid did not meet the reserve price.
    ReserveNotMet = 12,
    /// `cancel` was called after at least one bid has been placed.
    BidAlreadyPlaced = 13,
}

impl_display_error!(
    AuctionError,
    AlreadyInitialized => "already initialized",
    NotInitialized     => "not initialized",
    AuctionEnded       => "auction has ended",
    AuctionNotEnded    => "auction has not ended",
    BidTooLow          => "bid too low",
    AlreadyEnded       => "auction already settled",
    NoBids             => "no bids placed",
    NotAuthorized      => "not authorized",
    InvalidAmount      => "invalid amount",
    InvalidDeadline    => "invalid deadline",
    NothingToWithdraw  => "nothing to withdraw",
    ReserveNotMet      => "reserve price not met",
    BidAlreadyPlaced   => "cannot cancel after a bid has been placed",
);
