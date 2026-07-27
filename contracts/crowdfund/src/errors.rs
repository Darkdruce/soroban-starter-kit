// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_common::impl_display_error;
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug)]
pub enum CrowdfundError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    DeadlinePassed = 3,
    DeadlineNotReached = 4,
    GoalAlreadyMet = 5,
    GoalNotMet = 6,
    AlreadyClaimed = 7,
    NothingToPledge = 8,
    NothingToWithdraw = 9,
    InvalidAmount = 10,
    InvalidDeadline = 11,
    InvalidGoal = 12,
    NotAuthorized = 13,
    InvalidTier = 14,
    PledgeCapExceeded = 15,
    DeadlineAlreadyExtended = 16,
}

impl_display_error!(
    CrowdfundError,
    AlreadyInitialized      => "already initialized",
    NotInitialized          => "not initialized",
    DeadlinePassed          => "deadline has passed",
    DeadlineNotReached      => "deadline not reached",
    GoalAlreadyMet          => "goal already met",
    GoalNotMet              => "goal not met",
    AlreadyClaimed          => "funds already claimed",
    NothingToPledge         => "nothing to pledge",
    NothingToWithdraw       => "nothing to withdraw",
    InvalidAmount           => "invalid amount",
    InvalidDeadline         => "invalid deadline",
    InvalidGoal             => "invalid goal",
    NotAuthorized           => "not authorized",
    InvalidTier             => "invalid funding tier",
    PledgeCapExceeded       => "pledge would exceed the per-address cap",
    DeadlineAlreadyExtended => "deadline has already been extended once",
);
