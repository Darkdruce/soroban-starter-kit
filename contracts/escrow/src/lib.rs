#![no_std]

use soroban_sdk::{contract, contractimpl, token, Address, Env};

mod admin;
mod dispute;
mod errors;
mod events;
mod lifecycle;
mod queries;
mod storage;

pub use errors::EscrowError;
pub use storage::{DataKey, EscrowInfo, EscrowState};

#[cfg(feature = "pausable")]
use admin::require_admin;

/// Escrow contract for secure two-party transactions.
///
/// Lifecycle: `Created → Funded → Delivered → Completed`
/// with side exits to `Refunded` (deadline-based) or `Cancelled` (pre-fund).
#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn initialize(
        env: Env,
        buyer: Address,
        seller: Address,
        arbiter: Address,
        token_contract: Address,
        amount: i128,
        deadline_ledger: u32,
    ) -> Result<(), EscrowError> {
        lifecycle::initialize(env, buyer, seller, arbiter, token_contract, amount, deadline_ledger)
    }

    pub fn initialize_with_arbiters(
        env: Env,
        buyer: Address,
        seller: Address,
        arbiters: soroban_sdk::Vec<Address>,
        token_contract: Address,
        amount: i128,
        deadline_ledger: u32,
        required_signatures: u32,
    ) -> Result<(), EscrowError> {
        lifecycle::initialize_with_arbiters(
            env, buyer, seller, arbiters, token_contract, amount, deadline_ledger, required_signatures,
        )
    }

    pub fn update_amount(env: Env, new_amount: i128) -> Result<(), EscrowError> {
        lifecycle::update_amount(env, new_amount)
    }

    pub fn fund(env: Env) -> Result<(), EscrowError> {
        lifecycle::fund(env)
    }

    pub fn mark_delivered(env: Env) -> Result<(), EscrowError> {
        lifecycle::mark_delivered(env)
    }

    pub fn approve_delivery(env: Env) -> Result<(), EscrowError> {
        lifecycle::approve_delivery(env)
    }

    pub fn release_partial(env: Env, amount: i128) -> Result<(), EscrowError> {
        lifecycle::release_partial(env, amount)
    }

    pub fn request_refund(env: Env) -> Result<(), EscrowError> {
        lifecycle::request_refund(env)
    }

    pub fn raise_dispute(env: Env, caller: Address) -> Result<(), EscrowError> {
        dispute::raise_dispute(env, caller)
    }

    pub fn resolve_dispute(env: Env, release_to_seller: bool) -> Result<(), EscrowError> {
        dispute::resolve_dispute(env, release_to_seller)
    }

    pub fn cancel(env: Env) -> Result<(), EscrowError> {
        lifecycle::cancel(env)
    }

    pub fn extend_deadline(env: Env, new_deadline: u32) -> Result<(), EscrowError> {
        lifecycle::extend_deadline(env, new_deadline)
    }

    pub fn bump(env: Env) -> Result<(), EscrowError> {
        queries::bump(env)
    }

    #[must_use]
    pub fn get_escrow_info(env: Env) -> Result<EscrowInfo, EscrowError> {
        queries::get_escrow_info(env)
    }

    #[must_use]
    pub fn get_state(env: Env) -> Option<EscrowState> {
        queries::get_state(env)
    }

    #[must_use]
    pub fn is_deadline_passed(env: Env) -> bool {
        queries::is_deadline_passed(env)
    }

    pub fn get_remaining_ledgers(env: Env) -> i64 {
        queries::get_remaining_ledgers(env)
    }
}

/// Pause / unpause — only compiled when the `pausable` feature is enabled.
#[cfg(feature = "pausable")]
#[contractimpl]
impl EscrowContract {
    const UPGRADE_DELAY_LEDGERS: u32 = 17_280;

    pub fn pause(env: Env) -> Result<(), EscrowError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &true);
        lifecycle::extend_ttl(&env);
        events::paused(&env, &admin);
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), EscrowError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &false);
        lifecycle::extend_ttl(&env);
        events::unpaused(&env, &admin);
        Ok(())
    }

    #[must_use]
    pub fn version(env: Env) -> soroban_sdk::String {
        soroban_sdk::String::from_str(&env, env!("GIT_HASH"))
    }

    pub fn propose_upgrade(env: Env, wasm_hash: soroban_sdk::BytesN<32>) -> Result<(), EscrowError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        let ready_after = env.ledger().sequence() + Self::UPGRADE_DELAY_LEDGERS;
        env.storage()
            .instance()
            .set(&DataKey::PendingUpgrade, &(wasm_hash.clone(), ready_after));
        lifecycle::extend_ttl(&env);
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "upgrade_proposed"), admin),
            (wasm_hash, ready_after),
        );
        Ok(())
    }

    pub fn execute_upgrade(env: Env) -> Result<(), EscrowError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        let (wasm_hash, ready_after): (soroban_sdk::BytesN<32>, u32) = env
            .storage()
            .instance()
            .get(&DataKey::PendingUpgrade)
            .ok_or(EscrowError::NotAuthorized)?;
        if env.ledger().sequence() < ready_after {
            return Err(EscrowError::NotAuthorized);
        }
        env.storage().instance().remove(&DataKey::PendingUpgrade);
        events::upgraded(&env, &admin, &wasm_hash);
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "upgrade_executed"), admin),
            wasm_hash.clone(),
        );
        env.deployer().update_current_contract_wasm(wasm_hash);
        Ok(())
    }
}

impl EscrowContract {
    #[cfg(feature = "pausable")]
    pub(crate) fn require_not_paused(env: &Env) -> Result<(), EscrowError> {
        if env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(EscrowError::NotAuthorized);
        }
        Ok(())
    }
}

mod test;
