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
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

fn setup(env: &Env) -> (BallotContractClient, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let addr = env.register_contract(None, BallotContract);
    let client = BallotContractClient::new(env, &addr);
    client.initialize(&admin);
    (client, admin)
}

// ---------------------------------------------------------------------------
// Basic lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_ballot_lifecycle() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    client.register_voter(&voter1);
    client.register_voter(&voter2);

    client.vote(&voter1, &1u32);
    client.vote(&voter2, &0u32);

    assert_eq!(client.get_yes_votes(), 1);
    assert_eq!(client.get_no_votes(), 1);

    let (yes, no) = client.tally();
    assert_eq!(yes, 1);
    assert_eq!(no, 1);
}

// ---------------------------------------------------------------------------
// Double-vote prevention (#777 core behaviour)
// ---------------------------------------------------------------------------

#[test]
fn test_double_vote_prevention() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let voter = Address::generate(&env);
    client.register_voter(&voter);
    client.vote(&voter, &1u32);

    let result = client.try_vote(&voter, &1u32);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Unregistered voter rejected
// ---------------------------------------------------------------------------

#[test]
fn test_unregistered_voter_rejected() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let unregistered = Address::generate(&env);
    let result = client.try_vote(&unregistered, &1u32);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Invalid choice rejected
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_choice_rejected() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let voter = Address::generate(&env);
    client.register_voter(&voter);

    let result = client.try_vote(&voter, &2u32);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Double-initialize rejected
// ---------------------------------------------------------------------------

#[test]
fn test_double_initialize_rejected() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// TTL extension — #777
// Verify register_voter and vote extend persistent TTL (no archival regression).
// We test indirectly: after bumping the ledger far past the minimum threshold,
// the persistent keys should still be readable (they are extended on write).
// ---------------------------------------------------------------------------

#[test]
fn test_register_voter_extends_persistent_ttl() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let voter = Address::generate(&env);
    client.register_voter(&voter);

    // Advance the ledger significantly — if TTL were NOT extended the entry
    // would expire; with the fix the entry's TTL is bumped to LEDGER_BUMP_AMOUNT.
    env.ledger().with_mut(|l| l.sequence_number += 10_000);

    // The voter should still be registered (TTL was extended at register time).
    // If the entry had expired it would return false, causing vote() to fail.
    let result = client.try_vote(&voter, &1u32);
    // Should succeed (not fail with NotRegistered) — confirms TTL was extended.
    assert!(result.is_ok());
}

#[test]
fn test_vote_extends_persistent_ttl() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let voter = Address::generate(&env);
    client.register_voter(&voter);
    client.vote(&voter, &1u32);

    // Advance ledger; without the TTL fix the Voter key would expire and the
    // double-vote guard would fail to detect the previous vote.
    env.ledger().with_mut(|l| l.sequence_number += 10_000);

    // Attempt second vote — must still be rejected (AlreadyVoted).
    let result = client.try_vote(&voter, &1u32);
    assert!(result.is_err(), "second vote should be rejected even after ledger advance");
}

// ---------------------------------------------------------------------------
// Tally closes voting
// ---------------------------------------------------------------------------

#[test]
fn test_tally_closes_voting() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let voter = Address::generate(&env);
    client.register_voter(&voter);
    client.vote(&voter, &1u32);

    client.tally();

    // After tally, voting is closed — new votes should fail
    let voter2 = Address::generate(&env);
    client.register_voter(&voter2);
    let result = client.try_vote(&voter2, &0u32);
    assert!(result.is_err());
}
