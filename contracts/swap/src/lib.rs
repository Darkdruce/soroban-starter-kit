#![no_std]
#![deny(missing_docs)]
//! Atomic two-party token swap contract template.
//!
//! Party A proposes a swap of their tokens for party B's tokens; on acceptance
//! both transfers execute atomically, or party A may cancel beforehand.

use soroban_sdk::{Address, Env, contract, contractimpl, token};

mod errors;
mod events;
mod storage;

pub use errors::SwapError;
pub use storage::{DataKey, SwapInfo, SwapKey, SwapState};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump_persistent<K>(env: &Env, key: &K)
where
    K: soroban_sdk::TryIntoVal<Env, soroban_sdk::Val> + soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Atomic token swap contract. Party A proposes a swap, party B accepts, or party A cancels.
///
/// Party A specifies what they offer (`token_a`, `amount_a`) and what they want
/// (`token_b`, `amount_b`). On acceptance, both transfers execute atomically in one transaction.
pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module and re-export
// the public contract API above, keeping the rest of the crate enforced.
mod contract {
    #![allow(missing_docs)]
    use super::*;

    #[contract]
    pub struct SwapContract;

    #[contractimpl]
    impl SwapContract {
        /// Initialize the swap contract with admin, treasury, and fee configuration.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::AlreadyInitialized`] if the contract is already initialized.
        /// Returns [`SwapError::InvalidFee`] if `fee_bps` > 10000 (100%).
        pub fn initialize(
            env: Env,
            admin: Address,
            treasury: Address,
            fee_bps: u32,
        ) -> Result<(), SwapError> {
            if env.storage().instance().has(&DataKey::Initialized) {
                return Err(SwapError::AlreadyInitialized);
            }
            if fee_bps > 10_000 {
                return Err(SwapError::InvalidFee);
            }

            admin.require_auth();

            env.storage().instance().set(&DataKey::Admin, &admin);
            env.storage().instance().set(&DataKey::Treasury, &treasury);
            env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
            env.storage().instance().set(&DataKey::Initialized, &true);

            bump_instance(&env);
            Ok(())
        }

        /// Update the treasury address. Only admin can call this.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        /// Returns [`SwapError::NotAuthorized`] if caller is not the admin.
        pub fn set_treasury(env: Env, new_treasury: Address) -> Result<(), SwapError> {
            if !env.storage().instance().has(&DataKey::Initialized) {
                return Err(SwapError::NotInitialized);
            }

            let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            admin.require_auth();

            env.storage()
                .instance()
                .set(&DataKey::Treasury, &new_treasury);
            bump_instance(&env);

            Ok(())
        }

        /// Update the fee basis points. Only admin can call this.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        /// Returns [`SwapError::NotAuthorized`] if caller is not the admin.
        /// Returns [`SwapError::InvalidFee`] if `new_fee_bps` > 10000 (100%).
        pub fn set_fee_bps(env: Env, new_fee_bps: u32) -> Result<(), SwapError> {
            if !env.storage().instance().has(&DataKey::Initialized) {
                return Err(SwapError::NotInitialized);
            }
            if new_fee_bps > 10_000 {
                return Err(SwapError::InvalidFee);
            }

            let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            admin.require_auth();

            env.storage().instance().set(&DataKey::FeeBps, &new_fee_bps);
            bump_instance(&env);

            Ok(())
        }

        /// Update the admin address. Only current admin can call this.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        /// Returns [`SwapError::NotAuthorized`] if caller is not the current admin.
        pub fn set_admin(env: Env, new_admin: Address) -> Result<(), SwapError> {
            if !env.storage().instance().has(&DataKey::Initialized) {
                return Err(SwapError::NotInitialized);
            }

            let current_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            current_admin.require_auth();

            env.storage().instance().set(&DataKey::Admin, &new_admin);
            bump_instance(&env);

            Ok(())
        }

        /// Get the current admin address.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        pub fn get_admin(env: Env) -> Result<Address, SwapError> {
            if !env.storage().instance().has(&DataKey::Initialized) {
                return Err(SwapError::NotInitialized);
            }

            Ok(env.storage().instance().get(&DataKey::Admin).unwrap())
        }

        /// Get the current treasury address.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        pub fn get_treasury(env: Env) -> Result<Address, SwapError> {
            if !env.storage().instance().has(&DataKey::Initialized) {
                return Err(SwapError::NotInitialized);
            }

            Ok(env.storage().instance().get(&DataKey::Treasury).unwrap())
        }

        /// Get the current fee basis points.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        pub fn get_fee_bps(env: Env) -> Result<u32, SwapError> {
            if !env.storage().instance().has(&DataKey::Initialized) {
                return Err(SwapError::NotInitialized);
            }

            Ok(env.storage().instance().get(&DataKey::FeeBps).unwrap())
        }

        /// Propose a new swap. Party A deposits `amount_a` of `token_a` into the contract.
        ///
        /// Returns the `swap_id` for use in `accept_swap` or `cancel_swap`.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        /// Returns [`SwapError::InvalidAmount`] if either amount is <= 0.
        /// Returns [`SwapError::InvalidDeadline`] if `expires_at` <= current ledger.
        pub fn propose_swap(
            env: Env,
            party_a: Address,
            token_a: Address,
            amount_a: i128,
            token_b: Address,
            amount_b: i128,
            expires_at: u32,
        ) -> Result<u32, SwapError> {
            if !env.storage().instance().has(&DataKey::Initialized) {
                return Err(SwapError::NotInitialized);
            }
            if amount_a <= 0 || amount_b <= 0 {
                return Err(SwapError::InvalidAmount);
            }
            if expires_at <= env.ledger().sequence() {
                return Err(SwapError::InvalidDeadline);
            }

            party_a.require_auth();

            // Transfer token_a from party_a to this contract.
            token::Client::new(&env, &token_a).transfer(
                &party_a,
                &env.current_contract_address(),
                &amount_a,
            );

            let swap_id: u32 = env
                .storage()
                .instance()
                .get(&DataKey::SwapCount)
                .unwrap_or(0);

            let swap = SwapInfo {
                id: swap_id,
                party_a: party_a.clone(),
                token_a: token_a.clone(),
                amount_a,
                token_b: token_b.clone(),
                amount_b,
                expires_at,
                state: SwapState::Open,
            };

            env.storage()
                .persistent()
                .set(&SwapKey::Swap(swap_id), &swap);
            env.storage()
                .instance()
                .set(&DataKey::SwapCount, &(swap_id + 1));

            bump_persistent(&env, &SwapKey::Swap(swap_id));
            events::swap_proposed(
                &env, &party_a, swap_id, &token_a, amount_a, &token_b, amount_b, expires_at,
            );

            Ok(swap_id)
        }

        /// Accept a swap as party B. Party B deposits `amount_b` of `token_b` and both parties
        /// receive their requested tokens in the same transaction.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::SwapNotFound`] if the swap does not exist.
        /// Returns [`SwapError::AlreadyCompleted`] or [`SwapError::AlreadyCancelled`] for finished swaps.
        /// Returns [`SwapError::DeadlineExpired`] if the swap deadline has passed.
        pub fn accept_swap(env: Env, swap_id: u32, party_b: Address) -> Result<(), SwapError> {
            party_b.require_auth();

            if !env.storage().instance().has(&DataKey::Initialized) {
                return Err(SwapError::NotInitialized);
            }

            let treasury: Address = env.storage().instance().get(&DataKey::Treasury).unwrap();
            let fee_bps: u32 = env.storage().instance().get(&DataKey::FeeBps).unwrap();

            let mut swap: SwapInfo = env
                .storage()
                .persistent()
                .get(&SwapKey::Swap(swap_id))
                .ok_or(SwapError::SwapNotFound)?;

            match swap.state {
                SwapState::Completed => return Err(SwapError::AlreadyCompleted),
                SwapState::Cancelled => return Err(SwapError::AlreadyCancelled),
                SwapState::Open => {}
            }

            if env.ledger().sequence() > swap.expires_at {
                return Err(SwapError::DeadlineExpired);
            }

            swap.state = SwapState::Completed;
            env.storage()
                .persistent()
                .set(&SwapKey::Swap(swap_id), &swap);
            bump_persistent(&env, &SwapKey::Swap(swap_id));

            // Calculate fee - deducted from party A's token_b amount
            #[allow(clippy::arithmetic_side_effects)]
            let fee = (swap.amount_b * fee_bps as i128) / 10_000;
            #[allow(clippy::arithmetic_side_effects)]
            let party_a_amount = swap.amount_b - fee;

            // Party B sends token_b to this contract, then contract forwards all tokens.
            token::Client::new(&env, &swap.token_b).transfer(
                &party_b,
                &env.current_contract_address(),
                &swap.amount_b,
            );

            // Party B receives token_a.
            token::Client::new(&env, &swap.token_a).transfer(
                &env.current_contract_address(),
                &party_b,
                &swap.amount_a,
            );

            // Party A receives token_b minus fee.
            token::Client::new(&env, &swap.token_b).transfer(
                &env.current_contract_address(),
                &swap.party_a,
                &party_a_amount,
            );

            // Treasury receives the fee.
            if fee > 0 {
                token::Client::new(&env, &swap.token_b).transfer(
                    &env.current_contract_address(),
                    &treasury,
                    &fee,
                );
            }

            events::swap_accepted(&env, &party_b, swap_id);

            Ok(())
        }

        /// Cancel a swap. Party A can cancel any time before acceptance. After the deadline,
        /// anyone may cancel to return party A's tokens.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::SwapNotFound`] if the swap does not exist.
        /// Returns [`SwapError::AlreadyCompleted`] or [`SwapError::AlreadyCancelled`] if already done.
        /// Returns [`SwapError::NotAuthorized`] if the caller is not party A and the deadline has not passed.
        pub fn cancel_swap(env: Env, swap_id: u32) -> Result<(), SwapError> {
            if !env.storage().instance().has(&DataKey::Initialized) {
                return Err(SwapError::NotInitialized);
            }

            let mut swap: SwapInfo = env
                .storage()
                .persistent()
                .get(&SwapKey::Swap(swap_id))
                .ok_or(SwapError::SwapNotFound)?;

            match swap.state {
                SwapState::Completed => return Err(SwapError::AlreadyCompleted),
                SwapState::Cancelled => return Err(SwapError::AlreadyCancelled),
                SwapState::Open => {}
            }

            let expires_at_passed = env.ledger().sequence() > swap.expires_at;
            if !expires_at_passed {
                // Before expiry: only party A may cancel.
                swap.party_a.require_auth();
            }
            // After deadline: no auth required; anyone can trigger to return party A's funds.

            swap.state = SwapState::Cancelled;
            env.storage()
                .persistent()
                .set(&SwapKey::Swap(swap_id), &swap);
            bump_persistent(&env, &SwapKey::Swap(swap_id));

            // Return token_a to party_a.
            token::Client::new(&env, &swap.token_a).transfer(
                &env.current_contract_address(),
                &swap.party_a,
                &swap.amount_a,
            );

            events::swap_cancelled(&env, swap_id);

            Ok(())
        }

        /// Return swap details by ID.
        #[must_use]
        pub fn get_swap(env: Env, swap_id: u32) -> Result<SwapInfo, SwapError> {
            env.storage()
                .persistent()
                .get(&SwapKey::Swap(swap_id))
                .ok_or(SwapError::SwapNotFound)
        }

        /// Return total number of swaps proposed.
        #[must_use]
        pub fn swap_count(env: Env) -> u32 {
            env.storage()
                .instance()
                .get(&DataKey::SwapCount)
                .unwrap_or(0)
        }
    }
}

mod test;

#[cfg(test)]
mod prop_test;
