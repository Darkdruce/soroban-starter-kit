// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

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
}

impl core::fmt::Display for CrowdfundError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CrowdfundError::AlreadyInitialized => write!(f, "already initialized"),
            CrowdfundError::NotInitialized => write!(f, "not initialized"),
            CrowdfundError::DeadlinePassed => write!(f, "deadline has passed"),
            CrowdfundError::DeadlineNotReached => write!(f, "deadline not reached"),
            CrowdfundError::GoalAlreadyMet => write!(f, "goal already met"),
            CrowdfundError::GoalNotMet => write!(f, "goal not met"),
            CrowdfundError::AlreadyClaimed => write!(f, "funds already claimed"),
            CrowdfundError::NothingToPledge => write!(f, "nothing to pledge"),
            CrowdfundError::NothingToWithdraw => write!(f, "nothing to withdraw"),
            CrowdfundError::InvalidAmount => write!(f, "invalid amount"),
            CrowdfundError::InvalidDeadline => write!(f, "invalid deadline"),
            CrowdfundError::InvalidGoal => write!(f, "invalid goal"),
            CrowdfundError::NotAuthorized => write!(f, "not authorized"),
        }
    }
}
