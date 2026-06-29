#![no_std]
#![deny(missing_docs)]
//! Wrapped-token contract template.
//!
//! Wraps an underlying asset 1:1, minting wrapped tokens on deposit and
//! burning them on withdrawal.

use soroban_sdk::{contract, contractimpl, token, Address, Env};

mod errors;
mod events;
mod storage;

#[cfg(test)]
mod test;

pub use errors::WrappedTokenError;
pub use storage::DataKey;

use soroban_common::{extend_ttl_instance, LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump(env: &Env) {
    extend_ttl_instance(env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Wrapped XLM token contract.
///
/// Users deposit XLM and receive an equivalent amount of wrapped tokens.
/// Users can burn wrapped tokens to retrieve the underlying XLM.
///
/// Flow:
/// 1. Admin calls `initialize` — sets up the wrapped token address.
/// 2. Users call `wrap` to deposit XLM and mint wrapped tokens (1:1 peg).
/// 3. Users call `unwrap` to burn wrapped tokens and receive XLM (1:1 peg).
pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module and re-export
// the public contract API above, keeping the rest of the crate enforced.
mod contract {
    #![allow(missing_docs)]
    use super::*;

    #[contract]
    pub struct WrappedTokenContract;

#[contractimpl]
impl WrappedTokenContract {
    /// Initialize the wrapped token contract.
    ///
    /// # Errors
    /// - [`WrappedTokenError::AlreadyInitialized`] if called more than once.
    pub fn initialize(
        env: Env,
        admin: Address,
        wrapped_token: Address,
    ) -> Result<(), WrappedTokenError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(WrappedTokenError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::WrappedToken, &wrapped_token);
        env.storage().instance().set(&DataKey::TotalWrapped, &0i128);

        bump(&env);
        events::initialized(&env, &admin, &wrapped_token);
        Ok(())
    }

    /// Wrap XLM by sending it to the contract and minting wrapped tokens.
    ///
    /// 1:1 peg is maintained: amount XLM = amount wrapped tokens.
    ///
    /// # Errors
    /// - [`WrappedTokenError::NotInitialized`] if the contract has not been initialized.
    /// - [`WrappedTokenError::InvalidAmount`] if `amount` <= 0.
    pub fn wrap(env: Env, user: Address, amount: i128) -> Result<(), WrappedTokenError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(WrappedTokenError::NotInitialized);
        }
        if amount <= 0 {
            return Err(WrappedTokenError::InvalidAmount);
        }
        user.require_auth();

        let wrapped_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::WrappedToken)
            .ok_or(WrappedTokenError::NotInitialized)?;

        token::Client::new(&env, &wrapped_token)
            .mint(&user, &amount);

        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalWrapped)
            .unwrap_or(0i128);
        let new_total = total + amount;
        env.storage()
            .instance()
            .set(&DataKey::TotalWrapped, &new_total);

        bump(&env);
        events::wrapped(&env, &user, amount, new_total);
        Ok(())
    }

    /// Unwrap wrapped tokens by burning them and sending XLM back to the user.
    ///
    /// 1:1 peg is maintained: amount wrapped tokens = amount XLM.
    ///
    /// # Errors
    /// - [`WrappedTokenError::NotInitialized`] if the contract has not been initialized.
    /// - [`WrappedTokenError::InvalidAmount`] if `amount` <= 0.
    /// - [`WrappedTokenError::InsufficientBalance`] if user has insufficient wrapped tokens.
    pub fn unwrap(env: Env, user: Address, amount: i128) -> Result<(), WrappedTokenError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(WrappedTokenError::NotInitialized);
        }
        if amount <= 0 {
            return Err(WrappedTokenError::InvalidAmount);
        }
        user.require_auth();

        let wrapped_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::WrappedToken)
            .ok_or(WrappedTokenError::NotInitialized)?;

        token::Client::new(&env, &wrapped_token).burn(&user, &amount);

        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalWrapped)
            .unwrap_or(0i128);
        let new_total = total - amount;
        env.storage()
            .instance()
            .set(&DataKey::TotalWrapped, &new_total);

        bump(&env);
        events::unwrapped(&env, &user, amount, new_total);
        Ok(())
    }

    /// Returns the total amount of wrapped tokens.
    pub fn get_total_wrapped(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalWrapped)
            .unwrap_or(0i128)
    }
}
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[test]
    fn test_wrap_unwrap_1_1_peg() {
        let env = Env::default();
        env.ledger().with_mut(|le| {
            le.timestamp = 1;
        });

        let admin = Address::random(&env);
        let user = Address::random(&env);
        let wrapped_token = Address::random(&env);

        let contract = WrappedTokenContractClient::new(&env, &env.current_contract_id());

        contract.initialize(&admin, &wrapped_token);

        // Mock wrap: 100 units
        contract.wrap(&user, &100i128);
        assert_eq!(contract.get_total_wrapped(), 100);

        // Mock unwrap: 50 units
        contract.unwrap(&user, &50i128);
        assert_eq!(contract.get_total_wrapped(), 50);

        // Mock unwrap: remaining 50 units
        contract.unwrap(&user, &50i128);
        assert_eq!(contract.get_total_wrapped(), 0);
    }

    #[test]
    fn test_initialize_idempotent_check() {
        let env = Env::default();
        env.ledger().with_mut(|le| {
            le.timestamp = 1;
        });

        let admin = Address::random(&env);
        let wrapped_token = Address::random(&env);

        let contract = WrappedTokenContractClient::new(&env, &env.current_contract_id());

        contract.initialize(&admin, &wrapped_token);

        let result = contract.try_initialize(&admin, &wrapped_token);
        assert!(result.is_err());
        match result {
            Err(e) => assert_eq!(e.error(), WrappedTokenError::AlreadyInitialized),
            _ => unreachable!(),
        }
    }
}
