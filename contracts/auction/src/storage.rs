// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, contracttype};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Seller,
    Token,
    StartPrice,
    MinIncrement,
    Deadline,
    HighestBidder,
    HighestBid,
    Settled,
    /// Optional reserve price; auction settles only if highest_bid >= reserve_price.
    ReservePrice,
    /// Pending refund for outbid bidders.
    Pending(Address),
    /// Anti-sniping: number of ledgers to extend the deadline when a bid
    /// arrives within this window of the current deadline.
    ExtensionWindow,
    /// True once the seller has cancelled the auction (before any bids).
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AuctionInfo {
    pub seller: Address,
    pub token: Address,
    pub start_price: i128,
    pub min_increment: i128,
    pub deadline: u32,
    pub highest_bid: i128,
    pub highest_bidder: Option<Address>,
    pub settled: bool,
    /// Optional reserve price set at start.
    pub reserve_price: Option<i128>,
    /// Anti-sniping extension window in ledgers (0 = disabled).
    pub extension_window: u32,
}
