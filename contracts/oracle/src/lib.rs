#![no_std]
#![deny(missing_docs)]
//! Price oracle consumer contract template.
//!
//! An admin pushes signed price updates and consumers read them through
//! `get_price`, which rejects prices older than a configurable staleness
//! threshold.

use soroban_sdk::{Address, Env, contract, contractimpl};

mod errors;
mod events;
mod storage;

pub use errors::OracleError;
pub use storage::{DataKey, PriceData};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn get_required<T: soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>(
    env: &Env,
    key: &DataKey,
) -> Result<T, OracleError> {
    env.storage()
        .instance()
        .get(key)
        .ok_or(OracleError::NotInitialized)
}

pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module (which also
// hosts the unit tests so the generated client's fields stay reachable) and
// re-export the public contract API above.
mod contract {
    #![allow(missing_docs)]
    use super::*;
    use storage::DataKey::*;

    /// Price oracle consumer contract.
    ///
    /// The admin pushes price updates; consumers call `get_price` which
    /// validates that the price is not stale before returning it.
    #[contract]
    pub struct OracleContract;

    #[contractimpl]
    impl OracleContract {
        /// Initialize with an admin and a staleness threshold (in ledgers).
        ///
        /// # Errors
        /// - [`OracleError::AlreadyInitialized`] if already set up.
        /// - [`OracleError::InvalidStalenessThreshold`] if threshold is zero.
        pub fn initialize(
            env: Env,
            admin: Address,
            staleness_threshold: u32,
        ) -> Result<(), OracleError> {
            if env.storage().instance().has(&Admin) {
                return Err(OracleError::AlreadyInitialized);
            }
            if staleness_threshold == 0 {
                return Err(OracleError::InvalidStalenessThreshold);
            }
            admin.require_auth();

            env.storage().instance().set(&Admin, &admin);
            env.storage()
                .instance()
                .set(&StalenessThreshold, &staleness_threshold);

            bump_instance(&env);
            events::initialized(&env, &admin, staleness_threshold);
            Ok(())
        }

        /// Push a new price. Admin only.
        ///
        /// # Errors
        /// - [`OracleError::NotInitialized`] if not yet set up.
        /// - [`OracleError::Unauthorized`] if caller is not the admin.
        pub fn update_price(env: Env, price: i128) -> Result<(), OracleError> {
            let admin: Address = get_required(&env, &Admin)?;
            admin.require_auth();

            let ledger = env.ledger().sequence();
            let timestamp = env.ledger().timestamp();
            env.storage().instance().set(&Price, &price);
            env.storage().instance().set(&UpdatedAt, &ledger);
            env.storage()
                .instance()
                .set(&UpdatedAtTimestamp, &timestamp);

            bump_instance(&env);
            events::price_updated(&env, &admin, price, ledger);
            Ok(())
        }

        /// Return the current price, rejecting it if it is stale.
        ///
        /// A price is stale when `current_ledger - updated_at > staleness_threshold`.
        ///
        /// # Errors
        /// - [`OracleError::NotInitialized`] if no price has been pushed yet.
        /// - [`OracleError::StalePrice`] if the price is older than the threshold.
        pub fn get_price(env: Env) -> Result<i128, OracleError> {
            let price: i128 = get_required(&env, &Price)?;
            let updated_at: u32 = get_required(&env, &UpdatedAt)?;
            let threshold: u32 = get_required(&env, &StalenessThreshold)?;

            let age = env.ledger().sequence().saturating_sub(updated_at);
            if age > threshold {
                return Err(OracleError::StalePrice);
            }

            bump_instance(&env);
            Ok(price)
        }

        /// Return the current price, rejecting it if it is older than `max_age` seconds.
        ///
        /// Unlike `get_price` (which enforces the fixed `staleness_threshold` configured
        /// at `initialize` time, measured in ledgers), this lets a caller specify its own
        /// tolerance in seconds, checked against `env.ledger().timestamp()`.
        ///
        /// # Errors
        /// - [`OracleError::NotInitialized`] if no price has been pushed yet.
        /// - [`OracleError::StalePrice`] if `env.ledger().timestamp() - last_updated > max_age`.
        pub fn get_price_checked(env: Env, max_age: u64) -> Result<i128, OracleError> {
            let price: i128 = get_required(&env, &Price)?;
            let updated_at_timestamp: u64 = get_required(&env, &UpdatedAtTimestamp)?;

            let age = env.ledger().timestamp().saturating_sub(updated_at_timestamp);
            if age > max_age {
                return Err(OracleError::StalePrice);
            }

            bump_instance(&env);
            Ok(price)
        }

        /// Return the raw price data (price, updated_at, admin, staleness_threshold).
        ///
        /// # Errors
        /// - [`OracleError::NotInitialized`] if not yet set up.
        pub fn get_price_data(env: Env) -> Result<PriceData, OracleError> {
            Ok(PriceData {
                price: get_required(&env, &Price)?,
                updated_at: get_required(&env, &UpdatedAt)?,
                admin: get_required(&env, &Admin)?,
                staleness_threshold: get_required(&env, &StalenessThreshold)?,
            })
        }
    }
}

mod test;
