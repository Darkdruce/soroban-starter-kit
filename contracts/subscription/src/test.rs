#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing
)]
#![cfg(test)]

use soroban_sdk::{
    Address, Env,
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
};

use crate::{SubscriptionContract, SubscriptionContractClient, SubscriptionError};

// ── helpers ──────────────────────────────────────────────────────────────────

fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

fn make_token(env: &Env, mint_to: &Address, amount: i128) -> Address {
    let sac = env.register_stellar_asset_contract_v2(Address::generate(env));
    let addr = sac.address();
    StellarAssetClient::new(env, &addr).mint(mint_to, &amount);
    addr
}

/// Returns (client, contract_addr, provider, token).
fn setup(env: &Env) -> (SubscriptionContractClient, Address, Address, Address) {
    let provider = Address::generate(env);
    let token = make_token(env, &Address::generate(env), 0);
    let addr = env.register_contract(None, SubscriptionContract);
    let client = SubscriptionContractClient::new(env, &addr);
    client.initialize(&provider, &token);
    (client, addr, provider, token)
}

fn approve_and_subscribe(
    env: &Env,
    client: &SubscriptionContractClient,
    contract_addr: &Address,
    token_addr: &Address,
    subscriber: &Address,
    amount: i128,
    interval: u32,
    allowance: i128,
) {
    StellarAssetClient::new(env, token_addr).mint(subscriber, &allowance);
    let token_client = soroban_sdk::token::Client::new(env, token_addr);
    token_client.approve(subscriber, contract_addr, &allowance, &1_000_000);
    client.subscribe(subscriber, &amount, &interval);
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_provider_and_token() {
    let env = setup_env();
    let (client, _addr, provider, token) = setup(&env);
    assert_eq!(client.get_provider(), Some(provider));
    assert_eq!(client.get_token(), Some(token));
}

#[test]
fn test_initialize_twice_fails() {
    let env = setup_env();
    let (client, _addr, provider, token) = setup(&env);
    let result = client.try_initialize(&provider, &token);
    assert_eq!(result, Err(Ok(SubscriptionError::AlreadyInitialized)));
}

#[test]
fn test_subscribe_stores_subscription() {
    let env = setup_env();
    let (client, addr, _provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    approve_and_subscribe(&env, &client, &addr, &token, &subscriber, 100, 50, 500);

    let info = client.get_subscription(&subscriber).unwrap();
    assert_eq!(info.amount, 100);
    assert_eq!(info.interval_ledgers, 50);
    assert!(info.active);
}

#[test]
fn test_subscribe_zero_amount_fails() {
    let env = setup_env();
    let (client, _addr, _provider, _token) = setup(&env);
    let subscriber = Address::generate(&env);
    let result = client.try_subscribe(&subscriber, &0, &10);
    assert_eq!(result, Err(Ok(SubscriptionError::InvalidAmount)));
}

#[test]
fn test_subscribe_zero_interval_fails() {
    let env = setup_env();
    let (client, _addr, _provider, _token) = setup(&env);
    let subscriber = Address::generate(&env);
    let result = client.try_subscribe(&subscriber, &100, &0);
    assert_eq!(result, Err(Ok(SubscriptionError::InvalidInterval)));
}

#[test]
fn test_subscribe_twice_fails() {
    let env = setup_env();
    let (client, addr, _provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    approve_and_subscribe(&env, &client, &addr, &token, &subscriber, 100, 50, 1_000);
    let result = client.try_subscribe(&subscriber, &100, &50);
    assert_eq!(result, Err(Ok(SubscriptionError::AlreadySubscribed)));
}

#[test]
fn test_charge_before_interval_fails() {
    let env = setup_env();
    let (client, addr, provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    approve_and_subscribe(&env, &client, &addr, &token, &subscriber, 100, 50, 500);

    // Advance only 10 ledgers (interval is 50)
    env.ledger().with_mut(|l| l.sequence_number += 10);

    let result = client.try_charge(&subscriber);
    assert_eq!(result, Err(Ok(SubscriptionError::IntervalNotElapsed)));
    let _ = provider;
}

#[test]
fn test_charge_after_interval_transfers_tokens() {
    let env = setup_env();
    let (client, addr, provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    approve_and_subscribe(&env, &client, &addr, &token, &subscriber, 100, 50, 500);

    env.ledger().with_mut(|l| l.sequence_number += 50);
    client.charge(&subscriber);

    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&provider), 100);
    assert_eq!(token_client.balance(&subscriber), 400);
}

#[test]
fn test_charge_updates_last_charged_ledger() {
    let env = setup_env();
    let (client, addr, _provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    approve_and_subscribe(&env, &client, &addr, &token, &subscriber, 100, 50, 500);
    let start = env.ledger().sequence();

    env.ledger().with_mut(|l| l.sequence_number += 50);
    client.charge(&subscriber);

    let info = client.get_subscription(&subscriber).unwrap();
    assert_eq!(info.last_charged_ledger, start + 50);
}

#[test]
fn test_charge_multiple_times() {
    let env = setup_env();
    let (client, addr, provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    approve_and_subscribe(&env, &client, &addr, &token, &subscriber, 100, 50, 500);

    env.ledger().with_mut(|l| l.sequence_number += 50);
    client.charge(&subscriber);

    env.ledger().with_mut(|l| l.sequence_number += 50);
    client.charge(&subscriber);

    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&provider), 200);
}

#[test]
fn test_charge_insufficient_allowance_fails() {
    let env = setup_env();
    let (client, addr, _provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    // Mint tokens but approve less than the charge amount
    StellarAssetClient::new(&env, &token).mint(&subscriber, &500);
    let token_client = soroban_sdk::token::Client::new(&env, &token);
    token_client.approve(&subscriber, &addr, &50, &1_000_000); // only 50 approved, need 100
    client.subscribe(&subscriber, &100, &50);

    env.ledger().with_mut(|l| l.sequence_number += 50);
    let result = client.try_charge(&subscriber);
    assert_eq!(result, Err(Ok(SubscriptionError::InsufficientAllowance)));
}

#[test]
fn test_cancel_deactivates_subscription() {
    let env = setup_env();
    let (client, addr, _provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    approve_and_subscribe(&env, &client, &addr, &token, &subscriber, 100, 50, 500);
    client.cancel(&subscriber);

    let info = client.get_subscription(&subscriber).unwrap();
    assert!(!info.active);
}

#[test]
fn test_cancel_prevents_further_charges() {
    let env = setup_env();
    let (client, addr, _provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    approve_and_subscribe(&env, &client, &addr, &token, &subscriber, 100, 50, 500);
    client.cancel(&subscriber);

    env.ledger().with_mut(|l| l.sequence_number += 50);
    let result = client.try_charge(&subscriber);
    assert_eq!(result, Err(Ok(SubscriptionError::SubscriptionInactive)));
}

#[test]
fn test_cancel_twice_fails() {
    let env = setup_env();
    let (client, addr, _provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    approve_and_subscribe(&env, &client, &addr, &token, &subscriber, 100, 50, 500);
    client.cancel(&subscriber);

    let result = client.try_cancel(&subscriber);
    assert_eq!(result, Err(Ok(SubscriptionError::SubscriptionInactive)));
}

#[test]
fn test_cancel_nonexistent_fails() {
    let env = setup_env();
    let (client, _addr, _provider, _token) = setup(&env);
    let subscriber = Address::generate(&env);
    let result = client.try_cancel(&subscriber);
    assert_eq!(result, Err(Ok(SubscriptionError::NotSubscribed)));
}

#[test]
fn test_resubscribe_after_cancel() {
    let env = setup_env();
    let (client, addr, _provider, token) = setup(&env);
    let subscriber = Address::generate(&env);

    approve_and_subscribe(&env, &client, &addr, &token, &subscriber, 100, 50, 1_000);
    client.cancel(&subscriber);

    // Re-subscribing after cancel should succeed.
    let result = client.try_subscribe(&subscriber, &200, &30);
    assert!(result.is_ok());

    let info = client.get_subscription(&subscriber).unwrap();
    assert_eq!(info.amount, 200);
    assert_eq!(info.interval_ledgers, 30);
    assert!(info.active);
}

#[test]
fn test_charge_nonexistent_subscriber_fails() {
    let env = setup_env();
    let (client, _addr, _provider, _token) = setup(&env);
    let stranger = Address::generate(&env);
    let result = client.try_charge(&stranger);
    assert_eq!(result, Err(Ok(SubscriptionError::NotSubscribed)));
}

#[test]
fn test_get_subscription_returns_none_for_unknown() {
    let env = setup_env();
    let (client, _addr, _provider, _token) = setup(&env);
    let stranger = Address::generate(&env);
    assert_eq!(client.get_subscription(&stranger), None);
}
