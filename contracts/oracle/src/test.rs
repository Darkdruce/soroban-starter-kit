#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing
)]
#![cfg(test)]

use super::*;
use soroban_sdk::{
    Address, Env,
    testutils::{Address as _, Ledger as _},
};

fn setup(env: &Env) -> (OracleContractClient, Address) {
    let admin = Address::generate(env);
    let addr = env.register_contract(None, OracleContract);
    let client = OracleContractClient::new(env, &addr);
    client.initialize(&admin, &100);
    (client, admin)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    // Push a price so get_price_data works.
    client.update_price(&1_000_000);
    let data = client.get_price_data();
    assert_eq!(data.admin, admin);
    assert_eq!(data.staleness_threshold, 100);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    client.initialize(&admin, &100);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_initialize_zero_threshold_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let addr = env.register_contract(None, OracleContract);
    let client = OracleContractClient::new(&env, &addr);
    client.initialize(&admin, &0);
}

#[test]
fn test_update_and_get_price() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    client.update_price(&5_000_000);
    assert_eq!(client.get_price(), 5_000_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_get_price_before_any_update_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    client.get_price();
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_stale_price_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    client.update_price(&1_000_000);
    // Advance ledger past threshold (100).
    env.ledger().with_mut(|l| l.sequence_number += 101);
    client.get_price();
}

#[test]
fn test_price_at_threshold_boundary_is_valid() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    client.update_price(&2_000_000);
    // Advance exactly to threshold — still valid.
    env.ledger().with_mut(|l| l.sequence_number += 100);
    assert_eq!(client.get_price(), 2_000_000);
}

#[test]
fn test_price_update_overwrites_previous() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    client.update_price(&1_000);
    client.update_price(&9_999);
    assert_eq!(client.get_price(), 9_999);
}
