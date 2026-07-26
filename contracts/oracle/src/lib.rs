#![no_std]
#![deny(missing_docs)]
//! Price oracle consumer contract template.
//!
//! An admin pushes signed price updates and consumers read them through
//! `get_price`, which rejects prices older than a configurable staleness
//! threshold.

use soroban_sdk::{Address, Env, Vec, contract, contractimpl};

mod errors;
mod events;
mod storage;

pub use errors::OracleError;
pub use storage::{DataKey, PriceData, PublisherSubmission};

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

/// Compute the median of `values`, sorting them in place.
///
/// Uses insertion sort: authorized publisher sets are expected to stay small,
/// so this stays cheap while avoiding a dependency on the standard library.
#[allow(clippy::arithmetic_side_effects, clippy::integer_division)]
fn median(values: &mut Vec<i128>) -> i128 {
    let len = values.len();
    for i in 1..len {
        let key = values.get_unchecked(i);
        let mut j = i;
        while j > 0 && values.get_unchecked(j - 1) > key {
            let prev = values.get_unchecked(j - 1);
            values.set(j, prev);
            j -= 1;
        }
        values.set(j, key);
    }

    let mid = len / 2;
    #[allow(clippy::manual_is_multiple_of)] // is_multiple_of postdates the pinned 1.85.0 toolchain
    if len % 2 == 0 {
        values
            .get_unchecked(mid - 1)
            .saturating_add(values.get_unchecked(mid))
            / 2
    } else {
        values.get_unchecked(mid)
    }
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

        /// Configure the set of addresses authorized to submit prices via `submit_price`.
        /// Admin only. Replaces any previously configured set.
        ///
        /// # Errors
        /// - [`OracleError::NotInitialized`] if not yet set up.
        /// - [`OracleError::Unauthorized`] if `admin` is not the configured admin.
        pub fn set_publishers(
            env: Env,
            admin: Address,
            publishers: Vec<Address>,
        ) -> Result<(), OracleError> {
            let stored_admin: Address = get_required(&env, &Admin)?;
            admin.require_auth();
            if admin != stored_admin {
                return Err(OracleError::Unauthorized);
            }

            env.storage().instance().set(&Publishers, &publishers);
            bump_instance(&env);
            Ok(())
        }

        /// Submit a price as an authorized publisher.
        ///
        /// # Errors
        /// - [`OracleError::Unauthorized`] if `publisher` is not in the set configured
        ///   via `set_publishers`.
        pub fn submit_price(env: Env, publisher: Address, price: i128) -> Result<(), OracleError> {
            publisher.require_auth();

            let publishers: Vec<Address> = env
                .storage()
                .instance()
                .get(&Publishers)
                .unwrap_or(Vec::new(&env));
            if !publishers.contains(&publisher) {
                return Err(OracleError::Unauthorized);
            }

            let timestamp = env.ledger().timestamp();
            let key = Submission(publisher.clone());
            env.storage()
                .persistent()
                .set(&key, &PublisherSubmission { price, timestamp });
            env.storage().persistent().extend_ttl(
                &key,
                LEDGER_LIFETIME_THRESHOLD,
                LEDGER_BUMP_AMOUNT,
            );
            bump_instance(&env);

            events::price_submitted(&env, &publisher, price, timestamp);
            Ok(())
        }

        /// Return the median of the latest submission from each authorized publisher,
        /// rather than trusting a single caller as `get_price` does.
        ///
        /// # Errors
        /// - [`OracleError::NotInitialized`] if `set_publishers` has never been called.
        /// - [`OracleError::NoPublisherData`] if no publisher has submitted a price yet.
        pub fn get_median_price(env: Env) -> Result<i128, OracleError> {
            let publishers: Vec<Address> = get_required(&env, &Publishers)?;

            let mut prices: Vec<i128> = Vec::new(&env);
            for publisher in publishers.iter() {
                if let Some(submission) = env
                    .storage()
                    .persistent()
                    .get::<_, PublisherSubmission>(&Submission(publisher))
                {
                    prices.push_back(submission.price);
                }
            }

            if prices.is_empty() {
                return Err(OracleError::NoPublisherData);
            }

            Ok(median(&mut prices))
        }
    }
}

mod test;
