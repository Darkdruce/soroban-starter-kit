#![no_std]
#![deny(missing_docs)]
//! Crowdfunding campaign contract template.
//!
//! Contributors pledge tokens toward a funding goal before a deadline. If the
//! goal is met the creator claims the funds; otherwise contributors refund
//! their pledges.

use soroban_sdk::{Address, Env, contract, contractimpl, token};

mod errors;
mod events;
mod storage;

pub use errors::CrowdfundError;
pub use storage::{CrowdfundInfo, DataKey};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn get_instance<T: soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>(
    env: &Env,
    key: &DataKey,
) -> Result<T, CrowdfundError> {
    env.storage()
        .instance()
        .get(key)
        .ok_or(CrowdfundError::NotInitialized)
}

/// All-or-nothing crowdfunding contract.
///
/// Lifecycle:
/// - Creator calls `initialize` to set a token, funding goal, and deadline ledger.
/// - Contributors call `pledge` to deposit tokens before the deadline.
/// - If the goal is met, the creator calls `claim` to collect all funds after the deadline.
/// - If the goal is not met after the deadline, each contributor calls `refund` to recover their pledge.
/// - A contributor can call `withdraw` to pull back their pledge before the deadline.
pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module and re-export
// the public contract API above, keeping the rest of the crate enforced.
mod contract {
    #![allow(missing_docs)]
    use super::*;

    #[contract]
    pub struct CrowdfundContract;

    #[contractimpl]
    impl CrowdfundContract {
        /// Initialize the campaign. Can only be called once.
        ///
        /// # Errors
        ///
        /// - [`CrowdfundError::AlreadyInitialized`] if already set up.
        /// - [`CrowdfundError::InvalidGoal`] if `goal` <= 0.
        /// - [`CrowdfundError::InvalidDeadline`] if `deadline` <= current ledger.
        pub fn initialize(
            env: Env,
            creator: Address,
            token: Address,
            goal: i128,
            deadline: u32,
        ) -> Result<(), CrowdfundError> {
            if env.storage().instance().has(&DataKey::Creator) {
                return Err(CrowdfundError::AlreadyInitialized);
            }
            if goal <= 0 {
                return Err(CrowdfundError::InvalidGoal);
            }
            if deadline <= env.ledger().sequence() {
                return Err(CrowdfundError::InvalidDeadline);
            }

            creator.require_auth();

            env.storage().instance().set(&DataKey::Creator, &creator);
            env.storage().instance().set(&DataKey::Token, &token);
            env.storage().instance().set(&DataKey::Goal, &goal);
            env.storage().instance().set(&DataKey::Deadline, &deadline);
            env.storage()
                .instance()
                .set(&DataKey::TotalPledged, &0_i128);
            env.storage().instance().set(&DataKey::Claimed, &false);

            bump_instance(&env);
            events::initialized(&env, &creator, goal, deadline);
            Ok(())
        }

        /// Pledge `amount` tokens to the campaign. Must be called before the deadline.
        ///
        /// # Errors
        ///
        /// - [`CrowdfundError::NotInitialized`] if not set up.
        /// - [`CrowdfundError::DeadlinePassed`] if the deadline has passed.
        /// - [`CrowdfundError::InvalidAmount`] if `amount` <= 0.
        pub fn pledge(env: Env, pledger: Address, amount: i128) -> Result<(), CrowdfundError> {
            if amount <= 0 {
                return Err(CrowdfundError::InvalidAmount);
            }

            let deadline: u32 = get_instance(&env, &DataKey::Deadline)?;
            if env.ledger().sequence() > deadline {
                return Err(CrowdfundError::DeadlinePassed);
            }

            pledger.require_auth();

            let token: Address = get_instance(&env, &DataKey::Token)?;
            token::Client::new(&env, &token).transfer(
                &pledger,
                &env.current_contract_address(),
                &amount,
            );

            let existing: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Pledge(pledger.clone()))
                .unwrap_or(0);
            let new_pledge = existing + amount;
            env.storage()
                .persistent()
                .set(&DataKey::Pledge(pledger.clone()), &new_pledge);
            env.storage().persistent().extend_ttl(
                &DataKey::Pledge(pledger.clone()),
                LEDGER_LIFETIME_THRESHOLD,
                LEDGER_BUMP_AMOUNT,
            );

            let total: i128 = get_instance(&env, &DataKey::TotalPledged)?;
            let new_total = total + amount;
            env.storage()
                .instance()
                .set(&DataKey::TotalPledged, &new_total);

            bump_instance(&env);
            events::pledged(&env, &pledger, amount, new_total);
            Ok(())
        }

        /// Withdraw the caller's pledge before the deadline. Goal must not have been reached.
        ///
        /// # Errors
        ///
        /// - [`CrowdfundError::NotInitialized`] if not set up.
        /// - [`CrowdfundError::DeadlinePassed`] if the deadline has already passed.
        /// - [`CrowdfundError::NothingToWithdraw`] if the caller has no active pledge.
        pub fn withdraw(env: Env, pledger: Address) -> Result<(), CrowdfundError> {
            get_instance::<Address>(&env, &DataKey::Creator)?; // ensure initialized

            let deadline: u32 = get_instance(&env, &DataKey::Deadline)?;
            if env.ledger().sequence() > deadline {
                return Err(CrowdfundError::DeadlinePassed);
            }

            pledger.require_auth();

            let pledge: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Pledge(pledger.clone()))
                .unwrap_or(0);
            if pledge <= 0 {
                return Err(CrowdfundError::NothingToWithdraw);
            }

            env.storage()
                .persistent()
                .remove(&DataKey::Pledge(pledger.clone()));

            let total: i128 = get_instance(&env, &DataKey::TotalPledged)?;
            env.storage()
                .instance()
                .set(&DataKey::TotalPledged, &(total - pledge));

            let token: Address = get_instance(&env, &DataKey::Token)?;
            token::Client::new(&env, &token).transfer(
                &env.current_contract_address(),
                &pledger,
                &pledge,
            );

            bump_instance(&env);
            events::withdrawn(&env, &pledger, pledge);
            Ok(())
        }

        /// Creator claims all pledged funds after the deadline when the goal is met.
        ///
        /// # Errors
        ///
        /// - [`CrowdfundError::NotInitialized`] if not set up.
        /// - [`CrowdfundError::NotAuthorized`] if caller is not the creator.
        /// - [`CrowdfundError::DeadlineNotReached`] if the deadline has not passed.
        /// - [`CrowdfundError::GoalNotMet`] if total pledged < goal.
        /// - [`CrowdfundError::AlreadyClaimed`] if funds were already claimed.
        pub fn claim(env: Env) -> Result<(), CrowdfundError> {
            let creator: Address = get_instance(&env, &DataKey::Creator)?;
            creator.require_auth();

            let deadline: u32 = get_instance(&env, &DataKey::Deadline)?;
            if env.ledger().sequence() <= deadline {
                return Err(CrowdfundError::DeadlineNotReached);
            }

            let claimed: bool = get_instance(&env, &DataKey::Claimed)?;
            if claimed {
                return Err(CrowdfundError::AlreadyClaimed);
            }

            let goal: i128 = get_instance(&env, &DataKey::Goal)?;
            let total: i128 = get_instance(&env, &DataKey::TotalPledged)?;
            if total < goal {
                return Err(CrowdfundError::GoalNotMet);
            }

            env.storage().instance().set(&DataKey::Claimed, &true);

            let token: Address = get_instance(&env, &DataKey::Token)?;
            token::Client::new(&env, &token).transfer(
                &env.current_contract_address(),
                &creator,
                &total,
            );

            bump_instance(&env);
            events::claimed(&env, &creator, total);
            Ok(())
        }

        /// Contributor reclaims their pledge after the deadline when the goal was not met.
        ///
        /// # Errors
        ///
        /// - [`CrowdfundError::NotInitialized`] if not set up.
        /// - [`CrowdfundError::DeadlineNotReached`] if the deadline has not passed.
        /// - [`CrowdfundError::GoalAlreadyMet`] if the goal was met (creator should claim instead).
        /// - [`CrowdfundError::NothingToWithdraw`] if the caller has no pledge to refund.
        pub fn refund(env: Env, pledger: Address) -> Result<(), CrowdfundError> {
            get_instance::<Address>(&env, &DataKey::Creator)?; // ensure initialized

            let deadline: u32 = get_instance(&env, &DataKey::Deadline)?;
            if env.ledger().sequence() <= deadline {
                return Err(CrowdfundError::DeadlineNotReached);
            }

            let goal: i128 = get_instance(&env, &DataKey::Goal)?;
            let total: i128 = get_instance(&env, &DataKey::TotalPledged)?;
            if total >= goal {
                return Err(CrowdfundError::GoalAlreadyMet);
            }

            pledger.require_auth();

            let pledge: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Pledge(pledger.clone()))
                .unwrap_or(0);
            if pledge <= 0 {
                return Err(CrowdfundError::NothingToWithdraw);
            }

            env.storage()
                .persistent()
                .remove(&DataKey::Pledge(pledger.clone()));

            let token: Address = get_instance(&env, &DataKey::Token)?;
            token::Client::new(&env, &token).transfer(
                &env.current_contract_address(),
                &pledger,
                &pledge,
            );

            bump_instance(&env);
            events::refunded(&env, &pledger, pledge);
            Ok(())
        }

        /// Return campaign details.
        #[must_use]
        pub fn get_info(env: Env) -> Result<CrowdfundInfo, CrowdfundError> {
            Ok(CrowdfundInfo {
                creator: get_instance(&env, &DataKey::Creator)?,
                token: get_instance(&env, &DataKey::Token)?,
                goal: get_instance(&env, &DataKey::Goal)?,
                deadline: get_instance(&env, &DataKey::Deadline)?,
                total_pledged: get_instance(&env, &DataKey::TotalPledged)?,
                claimed: get_instance(&env, &DataKey::Claimed)?,
            })
        }

        /// Return a contributor's current pledge amount.
        #[must_use]
        pub fn get_pledge(env: Env, pledger: Address) -> i128 {
            env.storage()
                .persistent()
                .get(&DataKey::Pledge(pledger))
                .unwrap_or(0)
        }
    }
}

mod test;
