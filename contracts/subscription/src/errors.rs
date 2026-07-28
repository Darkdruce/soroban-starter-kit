// `#[contracterror]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SubscriptionError {
    /// `initialize` was called on an already-initialized contract.
    AlreadyInitialized = 1,
    /// An operation was attempted before the contract was initialized.
    NotInitialized = 2,
    /// Caller is not authorized (e.g. non-provider calling `charge`).
    NotAuthorized = 3,
    /// Amount is zero or negative.
    InvalidAmount = 4,
    /// Interval is zero.
    InvalidInterval = 5,
    /// Subscriber already has an active subscription.
    AlreadySubscribed = 6,
    /// No subscription found for this subscriber.
    NotSubscribed = 7,
    /// Subscription has been cancelled.
    SubscriptionInactive = 8,
    /// The charge interval has not elapsed since the last payment.
    IntervalNotElapsed = 9,
    /// Subscriber has not granted sufficient token allowance to this contract.
    InsufficientAllowance = 10,
    /// Plan with this ID already exists.
    PlanAlreadyExists = 11,
    /// Plan does not exist.
    PlanNotFound = 12,
    /// Plan is not active and cannot be subscribed to.
    PlanInactive = 13,
}