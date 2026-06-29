#![no_std]
#![deny(missing_docs)]
//! Token vesting contract template.
//!
//! An admin deposits tokens under a linear schedule with a cliff; the
//! beneficiary claims vested tokens over time and the admin may revoke the
//! unvested remainder.

use soroban_sdk::{Address, Env, contract, contractimpl, token};

mod errors;
mod events;
mod storage;

#[cfg(test)]
mod prop_test;
#[cfg(test)]
mod test;

pub use errors::VestingError;
pub use storage::{DataKey, VestingInfo};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD, extend_ttl_instance};

fn bump(env: &Env) {
    extend_ttl_instance(env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Returns the number of tokens vested as of `ledger`, ignoring already-claimed tokens.
pub(crate) fn vested_amount(amount: i128, cliff_ledger: u32, end_ledger: u32, ledger: u32) -> i128 {
    if ledger < cliff_ledger {
        return 0;
    }
    if ledger >= end_ledger {
        return amount;
    }
    // Linear interpolation between cliff and end.
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // u32 ledger difference fits in i128
    let elapsed = (ledger - cliff_ledger) as i128;
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // u32 ledger difference fits in i128
    let total = (end_ledger - cliff_ledger) as i128;
    amount * elapsed / total
}

fn validate_schedule(cliff_ledger: u32, end_ledger: u32, now: u32) -> Result<(), VestingError> {
    if cliff_ledger >= end_ledger || end_ledger <= now {
        return Err(VestingError::InvalidSchedule);
    }
    Ok(())
}

/// Token vesting contract with cliff + linear release schedule.
///
/// Flow:
/// 1. Admin calls `initialize` — deposits `amount` tokens and records the schedule.
/// 2. Beneficiary calls `claim` any time after the cliff to receive vested tokens.
/// 3. Admin may call `revoke` to cancel unvested tokens (returned to admin).
pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module and re-export
// the public contract API above, keeping the rest of the crate enforced.
mod contract {
    #![allow(missing_docs)]
    use super::*;

    #[contract]
    pub struct VestingContract;

    #[contractimpl]
    impl VestingContract {
        /// Set up the vesting schedule and transfer `amount` tokens from the caller into the contract.
        ///
        /// # Errors
        /// - [`VestingError::AlreadyInitialized`] if called more than once.
        /// - [`VestingError::InvalidAmount`] if `amount` <= 0.
        /// - [`VestingError::InvalidSchedule`] if `cliff_ledger` >= `end_ledger` or
        ///   `end_ledger` <= current ledger.
        pub fn initialize(
            env: Env,
            admin: Address,
            beneficiary: Address,
            token: Address,
            cliff_ledger: u32,
            end_ledger: u32,
            amount: i128,
        ) -> Result<(), VestingError> {
            if env.storage().instance().has(&DataKey::Admin) {
                return Err(VestingError::AlreadyInitialized);
            }
            if amount <= 0 {
                return Err(VestingError::InvalidAmount);
            }
            validate_schedule(cliff_ledger, end_ledger, env.ledger().sequence())?;

            admin.require_auth();

            // Pull tokens from admin into the contract.
            token::Client::new(&env, &token).transfer(
                &admin,
                &env.current_contract_address(),
                &amount,
            );

            env.storage().instance().set(&DataKey::Admin, &admin);
            env.storage()
                .instance()
                .set(&DataKey::Beneficiary, &beneficiary);
            env.storage().instance().set(&DataKey::Token, &token);
            env.storage()
                .instance()
                .set(&DataKey::CliffLedger, &cliff_ledger);
            env.storage()
                .instance()
                .set(&DataKey::EndLedger, &end_ledger);
            env.storage().instance().set(&DataKey::Amount, &amount);
            env.storage().instance().set(&DataKey::Claimed, &0i128);
            env.storage().instance().set(&DataKey::Revoked, &false);

            bump(&env);
            events::initialized(&env, &beneficiary, amount, cliff_ledger, end_ledger);
            Ok(())
        }

        /// Release all currently vested, unclaimed tokens to the beneficiary.
        ///
        /// After `revoke`, the beneficiary may still claim tokens that were vested
        /// at the time of revocation (the schedule amount is capped at that point).
        ///
        /// # Errors
        /// - [`VestingError::NotInitialized`] if the contract has not been initialized.
        /// - [`VestingError::NothingToClaim`] if no new tokens have vested since the last claim.
        pub fn claim(env: Env) -> Result<i128, VestingError> {
            if !env.storage().instance().has(&DataKey::Admin) {
                return Err(VestingError::NotInitialized);
            }

            let beneficiary: Address = env
                .storage()
                .instance()
                .get(&DataKey::Beneficiary)
                .ok_or(VestingError::NotInitialized)?;
            beneficiary.require_auth();

            let amount: i128 = env
                .storage()
                .instance()
                .get(&DataKey::Amount)
                .ok_or(VestingError::NotInitialized)?;
            let cliff_ledger: u32 = env
                .storage()
                .instance()
                .get(&DataKey::CliffLedger)
                .ok_or(VestingError::NotInitialized)?;
            let end_ledger: u32 = env
                .storage()
                .instance()
                .get(&DataKey::EndLedger)
                .ok_or(VestingError::NotInitialized)?;
            let claimed: i128 = env
                .storage()
                .instance()
                .get(&DataKey::Claimed)
                .ok_or(VestingError::NotInitialized)?;
            let revoked: bool = env
                .storage()
                .instance()
                .get(&DataKey::Revoked)
                .unwrap_or(false);

            // After revoke, `amount` is already capped to what was vested at revoke time.
            // We still allow claiming that remainder; once claimed == amount there's nothing left.
            let vested = if revoked {
                amount // amount was capped at revoke time
            } else {
                vested_amount(amount, cliff_ledger, end_ledger, env.ledger().sequence())
            };
            let claimable = vested - claimed;

            if claimable <= 0 {
                return Err(VestingError::NothingToClaim);
            }

            env.storage()
                .instance()
                .set(&DataKey::Claimed, &(claimed + claimable));

            let token: Address = env
                .storage()
                .instance()
                .get(&DataKey::Token)
                .ok_or(VestingError::NotInitialized)?;
            token::Client::new(&env, &token).transfer(
                &env.current_contract_address(),
                &beneficiary,
                &claimable,
            );

            bump(&env);
            events::claimed(&env, &beneficiary, claimable);
            Ok(claimable)
        }

        /// Admin cancels the vesting schedule. Unvested tokens are returned to admin;
        /// already-vested tokens remain claimable by the beneficiary (but no further
        /// vesting accrues after this ledger).
        ///
        /// # Errors
        /// - [`VestingError::NotInitialized`] if the contract has not been initialized.
        /// - [`VestingError::Unauthorized`] if the caller is not the admin.
        /// - [`VestingError::AlreadyRevoked`] if already revoked.
        pub fn revoke(env: Env) -> Result<i128, VestingError> {
            if !env.storage().instance().has(&DataKey::Admin) {
                return Err(VestingError::NotInitialized);
            }

            let revoked: bool = env
                .storage()
                .instance()
                .get(&DataKey::Revoked)
                .unwrap_or(false);
            if revoked {
                return Err(VestingError::AlreadyRevoked);
            }

            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(VestingError::NotInitialized)?;
            admin.require_auth();

            let amount: i128 = env
                .storage()
                .instance()
                .get(&DataKey::Amount)
                .ok_or(VestingError::NotInitialized)?;
            let cliff_ledger: u32 = env
                .storage()
                .instance()
                .get(&DataKey::CliffLedger)
                .ok_or(VestingError::NotInitialized)?;
            let end_ledger: u32 = env
                .storage()
                .instance()
                .get(&DataKey::EndLedger)
                .ok_or(VestingError::NotInitialized)?;
            let claimed: i128 = env
                .storage()
                .instance()
                .get(&DataKey::Claimed)
                .ok_or(VestingError::NotInitialized)?;

            let vested = vested_amount(amount, cliff_ledger, end_ledger, env.ledger().sequence());
            // Tokens vested but not yet claimed stay in the contract for the beneficiary.
            // Tokens not yet vested are returned to admin.
            let returnable = amount - vested;

            env.storage().instance().set(&DataKey::Revoked, &true);
            // Cap the schedule amount to what has vested so beneficiary can still claim the rest.
            env.storage().instance().set(&DataKey::Amount, &vested);
            // Claimed stays the same; beneficiary can still claim (vested - claimed).
            let _ = claimed; // already stored, no change needed

            let token: Address = env
                .storage()
                .instance()
                .get(&DataKey::Token)
                .ok_or(VestingError::NotInitialized)?;
            if returnable > 0 {
                token::Client::new(&env, &token).transfer(
                    &env.current_contract_address(),
                    &admin,
                    &returnable,
                );
            }

            bump(&env);
            events::revoked(&env, &admin, returnable);
            Ok(returnable)
        }

        /// Returns a snapshot of the vesting schedule, or `None` if uninitialized.
        pub fn get_info(env: Env) -> Option<VestingInfo> {
            if !env.storage().instance().has(&DataKey::Admin) {
                return None;
            }
            bump(&env);
            Some(VestingInfo {
                beneficiary: env
                    .storage()
                    .instance()
                    .get(&DataKey::Beneficiary)
                    .ok_or(VestingError::NotInitialized)
                    .ok()?,
                token: env
                    .storage()
                    .instance()
                    .get(&DataKey::Token)
                    .ok_or(VestingError::NotInitialized)
                    .ok()?,
                cliff_ledger: env
                    .storage()
                    .instance()
                    .get(&DataKey::CliffLedger)
                    .ok_or(VestingError::NotInitialized)
                    .ok()?,
                end_ledger: env
                    .storage()
                    .instance()
                    .get(&DataKey::EndLedger)
                    .ok_or(VestingError::NotInitialized)
                    .ok()?,
                amount: env
                    .storage()
                    .instance()
                    .get(&DataKey::Amount)
                    .ok_or(VestingError::NotInitialized)
                    .ok()?,
                claimed: env
                    .storage()
                    .instance()
                    .get(&DataKey::Claimed)
                    .ok_or(VestingError::NotInitialized)
                    .ok()?,
                revoked: env
                    .storage()
                    .instance()
                    .get(&DataKey::Revoked)
                    .unwrap_or(false),
            })
        }

        /// Returns the amount claimable right now (vested minus already claimed).
        pub fn claimable(env: Env) -> i128 {
            if !env.storage().instance().has(&DataKey::Admin) {
                return 0;
            }
            let amount: i128 = env
                .storage()
                .instance()
                .get(&DataKey::Amount)
                .ok_or(VestingError::NotInitialized)
                .unwrap_or(0);
            let cliff_ledger: u32 = env
                .storage()
                .instance()
                .get(&DataKey::CliffLedger)
                .ok_or(VestingError::NotInitialized)
                .unwrap_or(0);
            let end_ledger: u32 = env
                .storage()
                .instance()
                .get(&DataKey::EndLedger)
                .ok_or(VestingError::NotInitialized)
                .unwrap_or(0);
            let claimed: i128 = env
                .storage()
                .instance()
                .get(&DataKey::Claimed)
                .ok_or(VestingError::NotInitialized)
                .unwrap_or(0);
            let revoked: bool = env
                .storage()
                .instance()
                .get(&DataKey::Revoked)
                .unwrap_or(false);
            // After revoke, amount is already capped to what was vested at revoke time.
            let vested = if revoked {
                amount
            } else {
                vested_amount(amount, cliff_ledger, end_ledger, env.ledger().sequence())
            };
            (vested - claimed).max(0)
        }
    }
}
