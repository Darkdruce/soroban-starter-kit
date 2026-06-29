#![cfg(test)]

use super::*;
use soroban_sdk::{
    Address, Env,
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn register_token(env: &Env) -> (Address, Address) {
    let admin = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    (sac.address(), admin)
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    let token_admin = Address::generate(env);
    // Use the SAC admin from the token's own admin key via mock_all_auths
    let _ = token_admin;
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup(env: &Env) -> (SwapContractClient, Address, Address, Address, Address) {
    let (token_a, _) = register_token(env);
    let (token_b, _) = register_token(env);
    let party_a = Address::generate(env);
    let party_b = Address::generate(env);

    mint(env, &token_a, &party_a, 10_000);
    mint(env, &token_b, &party_b, 10_000);

    let addr = env.register_contract(None, SwapContract);
    let client = SwapContractClient::new(env, &addr);

    (client, party_a, party_b, token_a, token_b)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_propose_swap() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, party_a, _, token_a, token_b) = setup(&env);
    let deadline = env.ledger().sequence() + 100;

    let swap_id = client.propose_swap(&party_a, &token_a, &1_000, &token_b, &500, &deadline);
    assert_eq!(swap_id, 0);
    assert_eq!(client.swap_count(), 1);

    let swap = client.get_swap(&0);
    assert_eq!(swap.party_a, party_a);
    assert_eq!(swap.state, SwapState::Open);
    assert_eq!(swap.amount_a, 1_000);
    assert_eq!(swap.amount_b, 500);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_propose_swap_zero_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, party_a, _, token_a, token_b) = setup(&env);
    let deadline = env.ledger().sequence() + 100;

    client.propose_swap(&party_a, &token_a, &0, &token_b, &500, &deadline);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_propose_swap_past_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.sequence_number = 200);

    let (client, party_a, _, token_a, token_b) = setup(&env);
    client.propose_swap(&party_a, &token_a, &1_000, &token_b, &500, &100);
}

#[test]
fn test_accept_swap() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, party_a, party_b, token_a, token_b) = setup(&env);
    let deadline = env.ledger().sequence() + 100;

    let swap_id = client.propose_swap(&party_a, &token_a, &1_000, &token_b, &500, &deadline);
    client.accept_swap(&swap_id, &party_b);

    let swap = client.get_swap(&swap_id);
    assert_eq!(swap.state, SwapState::Completed);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_accept_swap_after_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, party_a, party_b, token_a, token_b) = setup(&env);
    let deadline = env.ledger().sequence() + 10;

    let swap_id = client.propose_swap(&party_a, &token_a, &1_000, &token_b, &500, &deadline);
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.accept_swap(&swap_id, &party_b);
}

#[test]
fn test_cancel_swap_by_party_a() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, party_a, _, token_a, token_b) = setup(&env);
    let deadline = env.ledger().sequence() + 100;

    let swap_id = client.propose_swap(&party_a, &token_a, &1_000, &token_b, &500, &deadline);
    client.cancel_swap(&swap_id);

    assert_eq!(client.get_swap(&swap_id).state, SwapState::Cancelled);
}

#[test]
fn test_cancel_swap_after_deadline_by_anyone() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, party_a, _, token_a, token_b) = setup(&env);
    let deadline = env.ledger().sequence() + 10;

    let swap_id = client.propose_swap(&party_a, &token_a, &1_000, &token_b, &500, &deadline);
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    // anyone can cancel after deadline
    client.cancel_swap(&swap_id);

    assert_eq!(client.get_swap(&swap_id).state, SwapState::Cancelled);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_accept_completed_swap_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, party_a, party_b, token_a, token_b) = setup(&env);
    let deadline = env.ledger().sequence() + 100;

    let swap_id = client.propose_swap(&party_a, &token_a, &1_000, &token_b, &500, &deadline);
    client.accept_swap(&swap_id, &party_b);
    client.accept_swap(&swap_id, &party_b);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_cancel_already_cancelled_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, party_a, _, token_a, token_b) = setup(&env);
    let deadline = env.ledger().sequence() + 100;

    let swap_id = client.propose_swap(&party_a, &token_a, &1_000, &token_b, &500, &deadline);
    client.cancel_swap(&swap_id);
    client.cancel_swap(&swap_id);
}

#[test]
fn test_multiple_swaps_increment_id() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, party_a, _, token_a, token_b) = setup(&env);
    let deadline = env.ledger().sequence() + 100;

    let id0 = client.propose_swap(&party_a, &token_a, &100, &token_b, &50, &deadline);
    let id1 = client.propose_swap(&party_a, &token_a, &200, &token_b, &100, &deadline);
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(client.swap_count(), 2);
}
