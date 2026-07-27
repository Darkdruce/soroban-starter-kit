use soroban_sdk::{Address, Env};

use crate::errors::EscrowError;
use crate::events;
use crate::lifecycle::{extend_ttl, get_required, refund_to_buyer, release_to_seller};
use crate::storage::{DataKey, EscrowState};

use DataKey::*;

pub fn raise_dispute(env: Env, caller: Address) -> Result<(), EscrowError> {
    #[cfg(feature = "pausable")]
    crate::EscrowContract::require_not_paused(&env)?;

    let buyer: Address = get_required(&env, &Buyer)?;
    let seller: Address = get_required(&env, &Seller)?;

    if caller != buyer && caller != seller {
        return Err(EscrowError::NotAuthorized);
    }
    caller.require_auth();

    let state: EscrowState = get_required(&env, &State)?;
    if !matches!(state, EscrowState::Funded | EscrowState::Delivered) {
        return Err(EscrowError::InvalidState);
    }

    env.storage().instance().set(&State, &EscrowState::Disputed);
    extend_ttl(&env);

    events::dispute_raised(&env, &caller);

    Ok(())
}

pub fn resolve_dispute(
    env: Env,
    caller: Address,
    release_to_seller_flag: bool,
) -> Result<(), EscrowError> {
    let state: EscrowState = get_required(&env, &State)?;
    if state != EscrowState::Disputed {
        return Err(EscrowError::InvalidState);
    }

    let arbiters_opt: Option<soroban_sdk::Vec<Address>> =
        env.storage().instance().get(&DataKey::Arbiters);

    if let Some(arbiters) = arbiters_opt {
        let required_sigs: u32 = env
            .storage()
            .instance()
            .get(&DataKey::RequiredSignatures)
            .unwrap_or(1);
        resolve_multisig(env, caller, arbiters, required_sigs, release_to_seller_flag)
    } else {
        resolve_single(env, release_to_seller_flag)
    }
}

fn resolve_single(env: Env, release_to_seller_flag: bool) -> Result<(), EscrowError> {
    let arbiter: Address = env
        .storage()
        .instance()
        .get(&Arbiter)
        .ok_or(EscrowError::NotInitialized)?;
    arbiter.require_auth();

    if release_to_seller_flag {
        env.storage()
            .instance()
            .set(&State, &EscrowState::Delivered);
        release_to_seller(env)
    } else {
        env.storage().instance().set(&State, &EscrowState::Funded);
        refund_to_buyer(env)
    }
}

fn resolve_multisig(
    env: Env,
    caller: Address,
    arbiters: soroban_sdk::Vec<Address>,
    required_sigs: u32,
    release_to_seller_flag: bool,
) -> Result<(), EscrowError> {
    let mut is_valid_arbiter = false;
    for arbiter in arbiters.iter() {
        if arbiter == caller {
            is_valid_arbiter = true;
            break;
        }
    }

    if !is_valid_arbiter {
        return Err(EscrowError::NotAuthorized);
    }

    caller.require_auth();

    let mut votes: soroban_sdk::Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::ArbiterVotes)
        .unwrap_or_else(|| soroban_sdk::Vec::new(&env));

    if !votes.iter().any(|v| v == caller) {
        votes.push_back(caller.clone());
    }

    env.storage().instance().set(&DataKey::ArbiterVotes, &votes);

    #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
    if votes.len() as u32 >= required_sigs {
        env.storage().instance().remove(&DataKey::ArbiterVotes);
        extend_ttl(&env);
        if release_to_seller_flag {
            env.storage()
                .instance()
                .set(&State, &EscrowState::Delivered);
            release_to_seller(env)
        } else {
            env.storage().instance().set(&State, &EscrowState::Funded);
            refund_to_buyer(env)
        }
    } else {
        extend_ttl(&env);
        Ok(())
    }
}
