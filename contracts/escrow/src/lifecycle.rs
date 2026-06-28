use soroban_sdk::{token, Address, Env, Vec};

use crate::admin;
use crate::errors::EscrowError;
use crate::events;
use crate::storage::{require_state, DataKey, EscrowState};
use soroban_common::{extend_ttl_instance, validate_deadline, LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

use DataKey::*;

pub fn get_required<T: soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>(
    env: &Env,
    key: &DataKey,
) -> Result<T, EscrowError> {
    env.storage()
        .instance()
        .get(key)
        .ok_or(EscrowError::NotInitialized)
}

fn validate_amount(amount: i128) -> Result<(), EscrowError> {
    if amount <= 0 {
        return Err(EscrowError::InvalidAmount);
    }
    Ok(())
}

fn validate_parties(buyer: &Address, seller: &Address, arbiter: &Address) -> Result<(), EscrowError> {
    if buyer == seller || buyer == arbiter || seller == arbiter {
        return Err(EscrowError::InvalidParties);
    }
    Ok(())
}

fn validate_parties_multi(buyer: &Address, seller: &Address, arbiters: &Vec<Address>, required_signatures: u32) -> Result<(), EscrowError> {
    if arbiters.is_empty() || required_signatures == 0 || required_signatures > arbiters.len() as u32 {
        return Err(EscrowError::InvalidParties);
    }
    for arbiter in arbiters.iter() {
        if &arbiter == buyer || &arbiter == seller {
            return Err(EscrowError::InvalidParties);
        }
    }
    Ok(())
}

fn store_escrow_data(
    env: &Env,
    buyer: &Address,
    seller: &Address,
    arbiter: &Address,
    token_contract: &Address,
    amount: i128,
    deadline_ledger: u32,
    required_signatures: u32,
) {
    env.storage().instance().set(&Buyer, buyer);
    env.storage().instance().set(&Seller, seller);
    env.storage().instance().set(&Arbiter, arbiter);
    env.storage().instance().set(&TokenContract, token_contract);
    env.storage().instance().set(&Amount, &amount);
    env.storage().instance().set(&Deadline, &deadline_ledger);
    env.storage().instance().set(&State, &EscrowState::Created);
    env.storage().instance().set(&RequiredSignatures, &required_signatures);
}

fn emit_init_events(env: &Env, buyer: &Address, seller: &Address, arbiter: &Address, amount: i128) {
    events::escrow_created(env, buyer, seller, amount);
    events::initialized(env, buyer, seller, arbiter, amount);
}

pub fn initialize(
    env: Env,
    buyer: Address,
    seller: Address,
    arbiter: Address,
    token_contract: Address,
    amount: i128,
    deadline_ledger: u32,
) -> Result<(), EscrowError> {
    if env.storage().instance().has(&State) {
        return Err(EscrowError::AlreadyInitialized);
    }
    validate_amount(amount)?;
    validate_parties(&buyer, &seller, &arbiter)?;
    validate_deadline::<EscrowError>(&env, deadline_ledger)?;
    token::Client::new(&env, &token_contract).decimals();
    store_escrow_data(&env, &buyer, &seller, &arbiter, &token_contract, amount, deadline_ledger, 1u32);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
    emit_init_events(&env, &buyer, &seller, &arbiter, amount);
    Ok(())
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
    if env.storage().instance().has(&State) {
        return Err(EscrowError::AlreadyInitialized);
    }
    validate_amount(amount)?;
    validate_parties_multi(&buyer, &seller, &arbiters, required_signatures)?;
    validate_deadline::<EscrowError>(&env, deadline_ledger)?;
    token::Client::new(&env, &token_contract).decimals();
    let primary_arbiter = arbiters.get(0).unwrap();
    store_escrow_data(&env, &buyer, &seller, &primary_arbiter, &token_contract, amount, deadline_ledger, required_signatures);
    env.storage().instance().set(&Arbiters, &arbiters);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
    emit_init_events(&env, &buyer, &seller, &primary_arbiter, amount);
    Ok(())
}

pub fn update_amount(env: Env, new_amount: i128) -> Result<(), EscrowError> {
    let buyer: Address = env
        .storage()
        .instance()
        .get(&Buyer)
        .ok_or(EscrowError::NotInitialized)?;
    buyer.require_auth();

    if new_amount <= 0 {
        return Err(EscrowError::InvalidAmount);
    }

    let state: EscrowState = env
        .storage()
        .instance()
        .get(&State)
        .ok_or(EscrowError::NotInitialized)?;
    if state != EscrowState::Created {
        return Err(EscrowError::InvalidState);
    }

    env.storage().instance().set(&Amount, &new_amount);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

    events::amount_updated(&env, &buyer, new_amount);

    Ok(())
}

pub fn fund(env: Env) -> Result<(), EscrowError> {
    #[cfg(feature = "pausable")]
    crate::EscrowContract::require_not_paused(&env)?;

    let buyer: Address = get_required(&env, &Buyer)?;
    buyer.require_auth();

    let state: EscrowState = get_required(&env, &State)?;
    if state != EscrowState::Created {
        return Err(EscrowError::InvalidState);
    }

    let token_contract: Address = get_required(&env, &TokenContract)?;
    let amount: i128 = get_required(&env, &Amount)?;

    let token_client = token::Client::new(&env, &token_contract);
    if token_client.balance(&buyer) < amount {
        return Err(EscrowError::InsufficientFunds);
    }
    token_client.transfer(&buyer, &env.current_contract_address(), &amount);

    env.storage().instance().set(&State, &EscrowState::Funded);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

    events::escrow_funded(&env, &buyer, amount);

    Ok(())
}

pub fn mark_delivered(env: Env) -> Result<(), EscrowError> {
    #[cfg(feature = "pausable")]
    crate::EscrowContract::require_not_paused(&env)?;

    let seller: Address = get_required(&env, &Seller)?;
    seller.require_auth();

    let state: EscrowState = get_required(&env, &State)?;
    if state != EscrowState::Funded {
        return Err(EscrowError::InvalidState);
    }

    env.storage().instance().set(&State, &EscrowState::Delivered);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

    events::delivery_marked(&env, &seller);

    Ok(())
}

pub fn approve_delivery(env: Env) -> Result<(), EscrowError> {
    #[cfg(feature = "pausable")]
    crate::EscrowContract::require_not_paused(&env)?;

    let buyer: Address = get_required(&env, &Buyer)?;
    buyer.require_auth();

    release_to_seller(env)
}

pub fn release_partial(env: Env, amount: i128) -> Result<(), EscrowError> {
    #[cfg(feature = "pausable")]
    crate::EscrowContract::require_not_paused(&env)?;

    let buyer: Address = env
        .storage()
        .instance()
        .get(&Buyer)
        .ok_or(EscrowError::NotInitialized)?;
    buyer.require_auth();

    let state: EscrowState = env
        .storage()
        .instance()
        .get(&State)
        .ok_or(EscrowError::NotInitialized)?;
    if state != EscrowState::Funded {
        return Err(EscrowError::InvalidState);
    }

    if amount <= 0 {
        return Err(EscrowError::InvalidAmount);
    }

    let stored_amount: i128 = env.storage().instance().get(&Amount).unwrap();
    if amount > stored_amount {
        return Err(EscrowError::InsufficientFunds);
    }

    let seller: Address = env.storage().instance().get(&Seller).unwrap();
    let new_amount = stored_amount - amount;
    env.storage().instance().set(&Amount, &new_amount);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

    admin::transfer_token(&env, &env.current_contract_address(), &seller, amount);
    events::partial_release(&env, &seller, amount);

    Ok(())
}

pub fn request_refund(env: Env) -> Result<(), EscrowError> {
    #[cfg(feature = "pausable")]
    crate::EscrowContract::require_not_paused(&env)?;

    let buyer: Address = get_required(&env, &Buyer)?;
    buyer.require_auth();

    let state: EscrowState = get_required(&env, &State)?;
    let deadline: u32 = get_required(&env, &Deadline)?;

    let can_refund = matches!(state, EscrowState::Funded | EscrowState::Delivered)
        && env.ledger().sequence() > deadline;
    if !can_refund {
        return Err(EscrowError::DeadlineNotReached);
    }

    refund_to_buyer(env)
}

pub fn cancel(env: Env) -> Result<(), EscrowError> {
    let buyer: Address = get_required(&env, &Buyer)?;
    buyer.require_auth();

    let state: EscrowState = get_required(&env, &State)?;
    if state != EscrowState::Created {
        return Err(EscrowError::InvalidState);
    }

    env.storage().instance().set(&State, &EscrowState::Cancelled);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

    events::escrow_cancelled(&env, &buyer);

    Ok(())
}

pub fn extend_deadline(env: Env, new_deadline: u32) -> Result<(), EscrowError> {
    let buyer: Address = env
        .storage()
        .instance()
        .get(&Buyer)
        .ok_or(EscrowError::NotInitialized)?;
    let seller: Address = env
        .storage()
        .instance()
        .get(&Seller)
        .ok_or(EscrowError::NotInitialized)?;

    buyer.require_auth();
    seller.require_auth();

    let current_deadline: u32 = env
        .storage()
        .instance()
        .get(&Deadline)
        .ok_or(EscrowError::NotInitialized)?;

    if new_deadline <= current_deadline {
        return Err(EscrowError::DeadlinePassed);
    }

    let state: EscrowState = env
        .storage()
        .instance()
        .get(&State)
        .ok_or(EscrowError::NotInitialized)?;
    if !matches!(state, EscrowState::Funded | EscrowState::Delivered) {
        return Err(EscrowError::InvalidState);
    }

    env.storage().instance().set(&Deadline, &new_deadline);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

    events::deadline_extended(&env, &buyer, new_deadline);

    Ok(())
}

pub fn release_to_seller(env: Env) -> Result<(), EscrowError> {
    require_state(&env, EscrowState::Delivered)?;

    let seller: Address = get_required(&env, &Seller)?;
    let amount: i128 = get_required(&env, &Amount)?;

    env.storage().instance().set(&State, &EscrowState::Completed);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

    admin::transfer_token(&env, &env.current_contract_address(), &seller, amount);

    events::funds_released(&env, &seller, amount);

    Ok(())
}

pub fn refund_to_buyer(env: Env) -> Result<(), EscrowError> {
    require_state(&env, EscrowState::Funded)?;

    let buyer: Address = get_required(&env, &Buyer)?;
    let amount: i128 = get_required(&env, &Amount)?;

    env.storage().instance().set(&State, &EscrowState::Refunded);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

    admin::transfer_token(&env, &env.current_contract_address(), &buyer, amount);

    events::funds_refunded(&env, &buyer, amount);

    Ok(())
}
