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

// Helper: ledger sequence when tests start.
const START_LEDGER: u32 = 10;
const VOTING_START: u32 = 20;
const VOTING_END: u32 = 100;

fn make_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|le| {
        le.sequence_number = START_LEDGER;
    });
    env
}

fn setup(env: &Env) -> (BallotContractClient, Address) {
    let admin = Address::generate(env);
    let addr = env.register_contract(None, BallotContract);
    let client = BallotContractClient::new(env, &addr);
    client.initialize(&admin, &VOTING_START, &VOTING_END);
    (client, admin)
}

// ---------------------------------------------------------------------------
// Basic lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_ballot_lifecycle() {
    let env = make_env();
    let (client, _admin) = setup(&env);

    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    client.register_voter(&voter1);
    client.register_voter(&voter2);

    // Advance into the voting window
    env.ledger().with_mut(|le| le.sequence_number = VOTING_START);

    client.vote(&voter1, &1u32);
    client.vote(&voter2, &0u32);

    assert_eq!(client.get_yes_votes(), 1);
    assert_eq!(client.get_no_votes(), 1);

    let (yes, no) = client.tally();
    assert_eq!(yes, 1);
    assert_eq!(no, 1);
}

// ---------------------------------------------------------------------------
// Double-vote prevention
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_double_vote_prevention() {
    let env = make_env();
    let (client, _admin) = setup(&env);

    let voter = Address::generate(&env);
    client.register_voter(&voter);
    env.ledger().with_mut(|le| le.sequence_number = VOTING_START);

    client.vote(&voter, &1u32);
    client.vote(&voter, &1u32); // should panic AlreadyVoted (#5)
}

// ---------------------------------------------------------------------------
// Unregistered voter rejected
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_unregistered_voter_rejected() {
    let env = make_env();
    let (client, _admin) = setup(&env);

    let unregistered = Address::generate(&env);
    env.ledger().with_mut(|le| le.sequence_number = VOTING_START);
    client.vote(&unregistered, &1u32); // should panic NotRegistered (#4)
}

// ---------------------------------------------------------------------------
// Invalid choice rejected
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_choice_rejected() {
    let env = make_env();
    let (client, _admin) = setup(&env);

    let voter = Address::generate(&env);
    client.register_voter(&voter);
    env.ledger().with_mut(|le| le.sequence_number = VOTING_START);

    let result = client.try_vote(&voter, &2u32);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Double-initialize rejected
// ---------------------------------------------------------------------------

#[test]
fn test_double_initialize_rejected() {
    let env = make_env();
    let (client, admin) = setup(&env);
    let result = client.try_initialize(&admin, &VOTING_START, &VOTING_END);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// TTL extension — #777
// Verify register_voter and vote extend persistent TTL (no archival regression).
// ---------------------------------------------------------------------------

#[test]
fn test_register_voter_extends_persistent_ttl() {
    let env = make_env();
    // Use a large voting window so we can advance the ledger significantly
    // while staying inside the window.
    let admin = Address::generate(&env);
    let addr = env.register_contract(None, BallotContract);
    let client = BallotContractClient::new(&env, &addr);
    let big_end: u32 = 100_000;
    client.initialize(&admin, &VOTING_START, &big_end);

    let voter = Address::generate(&env);
    client.register_voter(&voter);

    // Advance the ledger significantly while still inside the voting window.
    // If TTL were NOT extended the persistent entry would expire; with the fix
    // the entry's TTL is bumped to LEDGER_BUMP_AMOUNT on registration.
    env.ledger().with_mut(|l| l.sequence_number = VOTING_START + 10_000);

    // The voter should still be registered (TTL was extended at register time).
    let result = client.try_vote(&voter, &1u32);
    // Should succeed (not fail with NotRegistered) — confirms TTL was extended.
    assert!(result.is_ok());
}

#[test]
fn test_vote_extends_persistent_ttl() {
    let env = make_env();
    let (client, _admin) = setup(&env);

    let voter = Address::generate(&env);
    client.register_voter(&voter);
    env.ledger().with_mut(|l| l.sequence_number = VOTING_START);
    client.vote(&voter, &1u32);

    // Advance ledger; without the TTL fix the Voter key would expire and the
    // double-vote guard would fail to detect the previous vote.
    env.ledger().with_mut(|l| l.sequence_number = VOTING_START + 10_000);

    // Attempt second vote — must still be rejected (AlreadyVoted).
    let result = client.try_vote(&voter, &1u32);
    assert!(result.is_err(), "second vote should be rejected even after ledger advance");
}

// ---------------------------------------------------------------------------
// Tally closes voting
// ---------------------------------------------------------------------------

#[test]
fn test_tally_closes_voting() {
    let env = make_env();
    let (client, _admin) = setup(&env);

    let voter = Address::generate(&env);
    client.register_voter(&voter);
    env.ledger().with_mut(|l| l.sequence_number = VOTING_START);
    client.vote(&voter, &1u32);

    client.tally();

    // After tally, voting is closed — new votes should fail
    let voter2 = Address::generate(&env);
    client.register_voter(&voter2);
    env.ledger().with_mut(|l| l.sequence_number = VOTING_START + 1);
    let result = client.try_vote(&voter2, &0u32);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Issue #787 — Voting window tests
// ---------------------------------------------------------------------------

/// Vote before voting_start is rejected with VotingNotStarted (#10).
#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_vote_before_window_rejected() {
    let env = make_env();
    let (client, _admin) = setup(&env);
    let voter = Address::generate(&env);

    client.register_voter(&voter);
    // Still at START_LEDGER (10), before VOTING_START (20)
    client.vote(&voter, &1u32);
}

/// Vote after voting_end is rejected with VotingClosed (#7).
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_vote_after_window_rejected() {
    let env = make_env();
    let (client, _admin) = setup(&env);
    let voter = Address::generate(&env);

    client.register_voter(&voter);
    env.ledger().with_mut(|le| le.sequence_number = VOTING_END + 1);
    client.vote(&voter, &1u32);
}

/// Vote exactly at voting_start is accepted.
#[test]
fn test_vote_at_window_start_accepted() {
    let env = make_env();
    let (client, _admin) = setup(&env);
    let voter = Address::generate(&env);

    client.register_voter(&voter);
    env.ledger().with_mut(|le| le.sequence_number = VOTING_START);
    client.vote(&voter, &1u32);
    assert_eq!(client.get_yes_votes(), 1);
}

/// Vote exactly at voting_end is accepted.
#[test]
fn test_vote_at_window_end_accepted() {
    let env = make_env();
    let (client, _admin) = setup(&env);
    let voter = Address::generate(&env);

    client.register_voter(&voter);
    env.ledger().with_mut(|le| le.sequence_number = VOTING_END);
    client.vote(&voter, &1u32);
    assert_eq!(client.get_yes_votes(), 1);
}

/// initialize with invalid window (start >= end) is rejected (#9).
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_invalid_window_rejected() {
    let env = make_env();
    let admin = Address::generate(&env);
    let addr = env.register_contract(None, BallotContract);
    let client = BallotContractClient::new(&env, &addr);
    // start == end → InvalidWindow
    client.initialize(&admin, &50u32, &50u32);
}

// ---------------------------------------------------------------------------
// Issue #786 — deregister_voter tests
// ---------------------------------------------------------------------------

/// Admin can deregister a voter before any vote has been cast.
#[test]
fn test_deregister_voter_before_vote() {
    let env = make_env();
    let (client, _admin) = setup(&env);
    let voter = Address::generate(&env);

    client.register_voter(&voter);
    // Deregister should succeed (no votes cast yet)
    client.deregister_voter(&voter);

    // The voter is now unknown; trying to vote should fail (NotRegistered)
    env.ledger().with_mut(|le| le.sequence_number = VOTING_START);
    let result = client.try_vote(&voter, &1u32);
    assert!(result.is_err());
}

/// deregister_voter is rejected once a vote has been cast (#8).
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_deregister_voter_after_vote_rejected() {
    let env = make_env();
    let (client, _admin) = setup(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    client.register_voter(&voter1);
    client.register_voter(&voter2);

    // voter1 votes
    env.ledger().with_mut(|le| le.sequence_number = VOTING_START);
    client.vote(&voter1, &1u32);

    // Now trying to deregister voter2 should fail with VotingAlreadyStarted (#8)
    client.deregister_voter(&voter2);
}

/// deregister_voter on a non-registered voter fails with NotRegistered (#4).
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_deregister_unregistered_voter_rejected() {
    let env = make_env();
    let (client, _admin) = setup(&env);
    let non_voter = Address::generate(&env);

    // Should fail with NotRegistered (#4)
    client.deregister_voter(&non_voter);
}
