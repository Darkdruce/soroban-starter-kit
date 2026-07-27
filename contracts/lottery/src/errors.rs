// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug)]
pub enum LotteryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    LotteryClosed = 4,
    DrawAlreadyDone = 5,
    DrawNotDone = 6,
    InvalidTicketPrice = 7,
    CommitAlreadySubmitted = 8,
    RevealMismatch = 9,
    NoTickets = 10,
    InvalidRevealDeadline = 11,
    RefundNotAvailable = 12,
    NothingToRefund = 13,
    InvalidWinnerConfig = 14,
    InsufficientParticipants = 15,
    InvalidTicketCap = 11,
    TicketCapExceeded = 12,
}
