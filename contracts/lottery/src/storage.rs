// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, BytesN, contracttype};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    TicketPrice,
    /// Legacy key — no longer written; kept so existing deployed state can be
    /// read during any future migration if needed.
    Participants,
    State,
    Commit,
    Winner,
    Winners,
    RevealDeadline,
    WinnerCount,
    PrizeSplits,
    MaxTicketsPerAddress,
    /// Per-address ticket counter (persistent storage).
    TicketCount(Address),
    /// Indexed participant entry in persistent storage.
    /// `DataKey::Participant(idx)` where `idx` is 0-based.
    Participant(u32),
    /// Total number of tickets sold across all buyers (instance storage).
    TotalTickets,
    /// Optional minimum participants required before `draw` may succeed.
    MinParticipants,
}

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LotteryState {
    Open = 0,
    Committed = 1,
    Drawn = 2,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct LotteryInfo {
    pub admin: Address,
    pub token: Address,
    pub ticket_price: i128,
    pub state: LotteryState,
    /// Total number of tickets sold.
    pub total_tickets: u32,
}

/// Commit stored on-chain: hash(secret || salt) where both are 32-byte values.
#[contracttype]
#[derive(Clone)]
pub struct Commit {
    pub hash: BytesN<32>,
}
