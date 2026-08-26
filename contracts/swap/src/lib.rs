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

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD, apply_bps_fee};

fn extend_ttl_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn extend_ttl_persistent<K>(env: &Env, key: &K)
where
    K: soroban_sdk::TryIntoVal<Env, soroban_sdk::Val> + soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn bump_persistent<K>(env: &Env, key: &K)
where
    K: soroban_sdk::TryIntoVal<Env, soroban_sdk::Val> + soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn get_required<V: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>>(
    env: &Env,
    key: &impl soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
) -> Result<V, SwapError> {
    env.storage()
        .instance()
        .get(key)
        .ok_or(SwapError::NotInitialized)
}

/// Atomic two-party token swap contract.
pub use contract::*;

mod contract {
    #![allow(missing_docs)]
    use super::*;
    use storage::DataKey::*;
    use storage::SwapState;

    #[contract]
    pub struct SwapContract;

    #[contractimpl]
    impl SwapContract {
        /// Initialise the swap contract.
        ///
        /// # Errors
        /// - [`SwapError::AlreadyInitialized`]
        /// - [`SwapError::InvalidFee`] if `fee_bps` > 10000 (100%).
        pub fn initialize(
            env: Env,
            admin: Address,
            fee_bps: u32,
        ) -> Result<(), SwapError> {
            if env.storage().instance().has(&State) {
                return Err(SwapError::AlreadyInitialized);
            }
            if fee_bps > 10_000 {
                return Err(SwapError::InvalidFee);
            }
            admin.require_auth();

            env.storage().instance().set(&Admin, &admin);
            env.storage().instance().set(&FeeBps, &fee_bps);
            env.storage().instance().set(&State, &true);

            extend_ttl_instance(&env);
            bump_instance(&env);
            events::initialized(&env, &admin, fee_bps);
            Ok(())
        }

        /// Update the fee basis points. Only admin can call.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        /// Returns [`SwapError::NotAuthorized`] if caller is not the admin.
        pub fn set_treasury(env: Env, new_treasury: Address) -> Result<(), SwapError> {
            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(SwapError::NotInitialized)?;
            admin.require_auth();

            env.storage()
                .instance()
                .set(&DataKey::Treasury, &new_treasury);
            bump_instance(&env);
            env.storage().instance().set(&DataKey::Treasury, &new_treasury);
            extend_ttl_instance(&env);

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
            let admin: Address = get_required(&env, &Admin)?;
            admin.require_auth();

            if new_fee_bps > 10_000 {
                return Err(SwapError::InvalidFee);
            }
            env.storage().instance().set(&FeeBps, &new_fee_bps);
            bump_instance(&env);
            events::fee_updated(&env, &admin, new_fee_bps);
            Ok(())
        }

        /// Update the admin address. Only current admin can call this.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        /// Returns [`SwapError::NotAuthorized`] if caller is not the current admin.
        pub fn set_admin(env: Env, new_admin: Address) -> Result<(), SwapError> {
            let current_admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(SwapError::NotInitialized)?;
            current_admin.require_auth();

            env.storage().instance().set(&DataKey::Admin, &new_admin);
            extend_ttl_instance(&env);

            Ok(())
        }

        /// Get the current admin address.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        pub fn get_admin(env: Env) -> Result<Address, SwapError> {
            env.storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(SwapError::NotInitialized)
        }

        /// Get the current treasury address.
        ///
        /// # Errors
        ///
        /// Returns [`SwapError::NotInitialized`] if the contract is not initialized.
        pub fn get_treasury(env: Env) -> Result<Address, SwapError> {
            env.storage()
                .instance()
                .get(&DataKey::Treasury)
                .ok_or(SwapError::NotInitialized)
        }

            bump_instance(&env);
            events::fee_updated(&env, &admin, new_fee_bps);
            Ok(())
        }

        /// Get the current fee basis points.
        ///
        /// # Errors
        /// - [`SwapError::NotInitialized`]
        pub fn get_fee_bps(env: Env) -> Result<u32, SwapError> {
            get_required(&env, &FeeBps)
        }

        /// Propose a new swap. Party A proposes a swap of `amount_a` of `token_a`
        /// for `amount_b` of `token_b`. Returns the swap ID.
        ///
        /// The swap is valid until `expires_at` ledger.
        ///
        /// # Errors
        /// - [`SwapError::NotInitialized`]
        /// - [`SwapError::InvalidDeadline`] if `expires_at` <= current ledger.
        pub fn propose_swap(
            env: Env,
            party_a: Address,
            token_a: Address,
            amount_a: i128,
            token_b: Address,
            amount_b: i128,
            expires_at: u32,
        ) -> Result<u32, SwapError> {
            party_a.require_auth();

            if expires_at <= env.ledger().sequence() {
                return Err(SwapError::InvalidDeadline);
            }

            let swap_id: u32 = env
                .storage()
                .instance()
                .get(&SwapCount)
                .unwrap_or(0);
            let next_id = swap_id.checked_add(1).ok_or(SwapError::StorageError)?;
            env.storage().instance().set(&SwapCount, &next_id);

            let swap = SwapInfo {
                party_a: party_a.clone(),
                token_a: token_a.clone(),
                amount_a,
                token_b: token_b.clone(),
                amount_b,
                expires_at,
                state: SwapState::Pending,
            };
            env.storage()
                .persistent()
                .set(&SwapKey::Swap(swap_id), &swap);
            env.storage()
                .instance()
                .set(&DataKey::SwapCount, &(swap_id + 1));

            extend_ttl_persistent(&env, &SwapKey::Swap(swap_id));
            events::swap_proposed(
                &env, &party_a, swap_id, &token_a, amount_a, &token_b, amount_b, expires_at,
            );
            bump_persistent(&env, &SwapKey::Swap(swap_id));
            bump_instance(&env);

            events::swap_proposed(&env, &party_a, swap_id, token_a, amount_a, token_b, amount_b);
            Ok(swap_id)
        }

        /// Accept a proposed swap. Party B accepts and the swap executes atomically.
        ///
        /// # Errors
        /// - [`SwapError::NotInitialized`]
        /// - [`SwapError::SwapNotFound`]
        /// - [`SwapError::SwapNotPending`]
        /// - [`SwapError::SwapExpired`]
        pub fn accept_swap(
            env: Env,
            swap_id: u32,
            party_b: Address,
        ) -> Result<u32, SwapError> {
            party_b.require_auth();

            let treasury: Address = env
                .storage()
                .instance()
                .get(&DataKey::Treasury)
                .ok_or(SwapError::NotInitialized)?;
            let fee_bps: u32 = get_required(&env, &DataKey::FeeBps)?;

            let mut swap: SwapInfo = env
                .storage()
                .persistent()
                .get(&SwapKey::Swap(swap_id))
                .ok_or(SwapError::SwapNotFound)?;
            if swap.state != SwapState::Pending {
                return Err(SwapError::SwapNotPending);
            }
            if env.ledger().sequence() > swap.expires_at {
                return Err(SwapError::SwapExpired);
            }

            swap.state = SwapState::Accepted;
            env.storage()
                .persistent()
                .set(&SwapKey::Swap(swap_id), &swap);
            extend_ttl_persistent(&env, &SwapKey::Swap(swap_id));
            bump_persistent(&env, &SwapKey::Swap(swap_id));
            bump_instance(&env);

            // Calculate fee - deducted from party A's token_b amount
            let fee = apply_bps_fee(swap.amount_b, fee_bps).unwrap_or(0);
            #[allow(clippy::arithmetic_side_effects)]
            let party_a_amount = swap.amount_b - fee;

            // Party B sends token_b to this contract, then contract forwards all tokens.
            token::Client::new(&env, &swap.token_b).transfer(
                &party_b,
                &env.current_contract_address(),
                &swap.amount_b,
            );
            token::Client::new(&env, &swap.token_b).transfer(
                &env.current_contract_address(),
                &swap.party_a,
                &party_a_amount,
            );
            if fee > 0 {
                let admin: Address = get_required(&env, &Admin)?;
                token::Client::new(&env, &swap.token_b).transfer(
                    &env.current_contract_address(),
                    &admin,
                    &fee,
                );
            }

            // Party A sends token_a to party B.
            token::Client::new(&env, &swap.token_a).transfer(
                &swap.party_a,
                &party_b,
                &swap.amount_a,
            );

            events::swap_accepted(&env, &party_b, swap_id);
            Ok(swap_id)
        }

        /// Cancel a pending swap. Only party A can cancel.
        ///
        /// # Errors
        /// - [`SwapError::NotInitialized`]
        /// - [`SwapError::SwapNotFound`]
        /// - [`SwapError::SwapNotPending`]
        /// - [`SwapError::Unauthorized`]
        pub fn cancel_swap(env: Env, swap_id: u32) -> Result<(), SwapError> {
            let mut swap: SwapInfo = env
                .storage()
                .persistent()
                .get(&SwapKey::Swap(swap_id))
                .ok_or(SwapError::SwapNotFound)?;
            if swap.state != SwapState::Pending {
                return Err(SwapError::SwapNotPending);
            }

            swap.party_a.require_auth();
            if swap.party_a != env.current_contract_address() {
                return Err(SwapError::Unauthorized);
            }

            swap.state = SwapState::Cancelled;
            env.storage()
                .persistent()
                .set(&SwapKey::Swap(swap_id), &swap);
            extend_ttl_persistent(&env, &SwapKey::Swap(swap_id));
            bump_persistent(&env, &SwapKey::Swap(swap_id));
            bump_instance(&env);

            events::swap_cancelled(&env, &swap.party_a, swap_id);
            Ok(())
        }

        /// Get swap details.
        ///
        /// # Errors
        /// - [`SwapError::NotInitialized`]
        /// - [`SwapError::SwapNotFound`]
        pub fn get_swap(env: Env, swap_id: u32) -> Result<SwapInfo, SwapError> {
            let swap: SwapInfo = env
                .storage()
                .persistent()
                .get(&SwapKey::Swap(swap_id))
                .ok_or(SwapError::SwapNotFound)?;
            Ok(swap)
        }
    }
}

mod test;

#[cfg(test)]
mod prop_test;
