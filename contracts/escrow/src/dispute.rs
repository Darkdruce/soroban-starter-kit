use soroban_sdk::{Address, Env};

use crate::errors::EscrowError;
use crate::events;
use crate::lifecycle::{get_required, refund_to_buyer, release_to_seller};
use crate::storage::{DataKey, EscrowState};
use soroban_common::{extend_ttl_instance, LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

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
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

    events::dispute_raised(&env, &caller);

    Ok(())
}

pub fn resolve_dispute(env: Env, release_to_seller_flag: bool) -> Result<(), EscrowError> {
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

        let mut votes: soroban_sdk::Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::ArbiterVotes)
            .unwrap_or_else(|| soroban_sdk::Vec::new(&env));

        let mut caller_found = false;
        for arbiter in arbiters.iter() {
            arbiter.require_auth();

            let mut already_voted = false;
            for vote in votes.iter() {
                if vote == arbiter {
                    already_voted = true;
                    break;
                }
            }

            if !already_voted {
                votes.push_back(arbiter.clone());
            }
            caller_found = true;
            break;
        }

        if !caller_found {
            return Err(EscrowError::NotAuthorized);
        }

        env.storage().instance().set(&DataKey::ArbiterVotes, &votes);

        if votes.len() as u32 >= required_sigs {
            env.storage().instance().remove(&DataKey::ArbiterVotes);
            if release_to_seller_flag {
                env.storage().instance().set(&State, &EscrowState::Delivered);
                release_to_seller(env)
            } else {
                env.storage().instance().set(&State, &EscrowState::Funded);
                refund_to_buyer(env)
            }
        } else {
            Ok(())
        }
    } else {
        let arbiter: Address = env
            .storage()
            .instance()
            .get(&Arbiter)
            .ok_or(EscrowError::NotInitialized)?;
        arbiter.require_auth();

        if release_to_seller_flag {
            env.storage().instance().set(&State, &EscrowState::Delivered);
            release_to_seller(env)
        } else {
            env.storage().instance().set(&State, &EscrowState::Funded);
            refund_to_buyer(env)
        }
    }
}
