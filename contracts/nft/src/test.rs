#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing
)]
#![cfg(test)]

use super::*;
use soroban_sdk::{Address, Env, String, testutils::Address as _};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup(env: &Env) -> (NftContractClient, Address) {
    let admin = Address::generate(env);
    let addr = env.register_contract(None, NftContract);
    let client = NftContractClient::new(env, &addr);
    client.initialize(
        &admin,
        &String::from_str(env, "My Collection"),
        &String::from_str(env, "MYC"),
        &0,
    );
    (client, admin)
}

fn setup_with_cap(env: &Env, max_supply: u32) -> (NftContractClient, Address) {
    let admin = Address::generate(env);
    let addr = env.register_contract(None, NftContract);
    let client = NftContractClient::new(env, &addr);
    client.initialize(
        &admin,
        &String::from_str(env, "Capped"),
        &String::from_str(env, "CAP"),
        &max_supply,
    );
    (client, admin)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    assert_eq!(client.name(), String::from_str(&env, "My Collection"));
    assert_eq!(client.symbol(), String::from_str(&env, "MYC"));
    assert_eq!(client.total_supply(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let addr = env.register_contract(None, NftContract);
    let client = NftContractClient::new(&env, &addr);
    client.initialize(
        &admin,
        &String::from_str(&env, "A"),
        &String::from_str(&env, "A"),
        &0,
    );
    client.initialize(
        &admin,
        &String::from_str(&env, "B"),
        &String::from_str(&env, "B"),
        &0,
    );
}

#[test]
fn test_mint() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let owner = Address::generate(&env);
    client.mint(&owner, &1, &String::from_str(&env, "ipfs://token/1"));

    assert_eq!(client.owner_of(&1), owner);
    assert_eq!(client.total_supply(), 1);
    assert_eq!(
        client.token_uri(&1),
        String::from_str(&env, "ipfs://token/1")
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_mint_duplicate_token_id_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let owner = Address::generate(&env);
    client.mint(&owner, &1, &String::from_str(&env, "ipfs://1"));
    client.mint(&owner, &1, &String::from_str(&env, "ipfs://1b"));
}

#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.mint(&alice, &1, &String::from_str(&env, "ipfs://1"));
    client.transfer(&alice, &bob, &1);

    assert_eq!(client.owner_of(&1), bob);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_transfer_not_owner_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    client.mint(&alice, &1, &String::from_str(&env, "ipfs://1"));
    // bob tries to transfer alice's token
    client.transfer(&bob, &carol, &1);
}

#[test]
fn test_burn() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let owner = Address::generate(&env);
    client.mint(&owner, &1, &String::from_str(&env, "ipfs://1"));
    assert_eq!(client.total_supply(), 1);

    client.burn(&owner, &1);
    assert_eq!(client.total_supply(), 0);
}

#[test]
fn test_approve_and_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    client.mint(&alice, &1, &String::from_str(&env, "ipfs://1"));
    client.approve(&1, &bob);
    assert_eq!(client.get_approved(&1), Some(bob.clone()));

    client.transfer_from(&bob, &alice, &carol, &1);
    assert_eq!(client.owner_of(&1), carol);
    // Approval should be cleared after transfer.
    assert_eq!(client.get_approved(&1), None);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_transfer_from_without_approval_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    client.mint(&alice, &1, &String::from_str(&env, "ipfs://1"));
    // Bob was never approved.
    client.transfer_from(&bob, &alice, &carol, &1);
}

#[test]
fn test_transfer_clears_approval() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    client.mint(&alice, &1, &String::from_str(&env, "ipfs://1"));
    client.approve(&1, &bob);

    // Direct transfer should clear the approval.
    client.transfer(&alice, &carol, &1);
    assert_eq!(client.get_approved(&1), None);
}

#[test]
fn test_supply_cap_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_with_cap(&env, 2);

    let owner = Address::generate(&env);
    client.mint(&owner, &1, &String::from_str(&env, "ipfs://1"));
    client.mint(&owner, &2, &String::from_str(&env, "ipfs://2"));
    assert_eq!(client.total_supply(), 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_supply_cap_exceeded_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_with_cap(&env, 1);

    let owner = Address::generate(&env);
    client.mint(&owner, &1, &String::from_str(&env, "ipfs://1"));
    client.mint(&owner, &2, &String::from_str(&env, "ipfs://2"));
}

#[test]
fn test_metadata() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let meta = client.metadata();
    assert_eq!(meta.name, String::from_str(&env, "My Collection"));
    assert_eq!(meta.symbol, String::from_str(&env, "MYC"));
}
