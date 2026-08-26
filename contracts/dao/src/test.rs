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
    Address, Env, String,
    testutils::{Address as _, Ledger as _},
};
use soroban_token_template::{TokenContract, TokenContractClient};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Default setup: voting_period=100, quorum=500 (absolute), quorum_bps=0 (disabled).
fn setup(env: &Env) -> (DaoContractClient, Address, Address, Address) {
    setup_with_quorum_bps(env, 500, 0)
}

/// Setup with explicit quorum (absolute) and quorum_bps (percentage).
///
/// The governance token is `soroban-token-template`, not a bare Stellar
/// Asset Contract: the quorum_bps check needs `total_supply`, which is not
/// part of the SEP-41 `TokenInterface` a SAC exposes.
fn setup_with_quorum_bps(
    env: &Env,
    quorum: i128,
    quorum_bps: u32,
) -> (DaoContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_addr = env.register_contract(None, TokenContract);
    TokenContractClient::new(env, &token_addr).initialize(
        &admin,
        &String::from_str(env, "Governance Token"),
        &String::from_str(env, "GOV"),
        &7u32,
        &None,
    );

    let addr = env.register_contract(None, DaoContract);
    let client = DaoContractClient::new(env, &addr);
    client.initialize(&admin, &token_addr, &100, &quorum, &quorum_bps);

    (client, admin, token_addr, addr)
}

fn mint_tokens(env: &Env, token: &Address, admin: &Address, to: &Address, amount: i128) {
    let _ = admin;
    TokenContractClient::new(env, token).mint(to, &amount);
}

// ---------------------------------------------------------------------------
// Existing tests (updated for new initialize signature)
// ---------------------------------------------------------------------------

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    assert_eq!(client.proposal_count(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);
    client.initialize(&admin, &token, &100, &500, &0);
}

#[test]
fn test_create_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);

    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "Upgrade Protocol"),
        &String::from_str(&env, "Upgrade to v2"),
    );
    assert_eq!(id, 0);
    assert_eq!(client.proposal_count(), 1);

    let proposal = client.get_proposal(&0);
    assert_eq!(proposal.state, ProposalState::Active);
    assert_eq!(proposal.yes_votes, 0);
    assert_eq!(proposal.no_votes, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_create_proposal_no_tokens_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);

    let proposer = Address::generate(&env);
    // proposer has no tokens
    client.create_proposal(
        &proposer,
        &String::from_str(&env, "Bad Proposal"),
        &String::from_str(&env, "no tokens"),
    );
}

#[test]
fn test_vote_yes() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P1"),
        &String::from_str(&env, "Desc"),
    );

    let voter = Address::generate(&env);
    mint_tokens(&env, &token, &admin, &voter, 600);
    client.vote(&voter, &id, &true);

    let proposal = client.get_proposal(&id);
    assert_eq!(proposal.yes_votes, 600);
    assert_eq!(proposal.no_votes, 0);
}

#[test]
fn test_vote_no() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P1"),
        &String::from_str(&env, "Desc"),
    );

    let voter = Address::generate(&env);
    mint_tokens(&env, &token, &admin, &voter, 300);
    client.vote(&voter, &id, &false);

    let proposal = client.get_proposal(&id);
    assert_eq!(proposal.yes_votes, 0);
    assert_eq!(proposal.no_votes, 300);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_vote_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    let voter = Address::generate(&env);
    mint_tokens(&env, &token, &admin, &voter, 100);
    client.vote(&voter, &id, &true);
    client.vote(&voter, &id, &true);
}

#[test]
fn test_execute_proposal_passes() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    let voter = Address::generate(&env);
    mint_tokens(&env, &token, &admin, &voter, 600);
    client.vote(&voter, &id, &true);

    // Advance past voting deadline (voting_period = 100)
    let deadline = client.get_proposal(&id).deadline;
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);

    client.execute_proposal(&id);
    assert_eq!(client.get_proposal(&id).state, ProposalState::Executed);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_execute_before_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    let voter = Address::generate(&env);
    mint_tokens(&env, &token, &admin, &voter, 600);
    client.vote(&voter, &id, &true);
    // Do NOT advance past deadline
    client.execute_proposal(&id);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_execute_quorum_not_met_fails() {
    let env = Env::default();
    env.mock_all_auths();
    // quorum = 500
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    let voter = Address::generate(&env);
    mint_tokens(&env, &token, &admin, &voter, 100); // only 100 < 500 quorum
    client.vote(&voter, &id, &true);

    let deadline = client.get_proposal(&id).deadline;
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.execute_proposal(&id);
}

#[test]
fn test_cancel_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    client.cancel_proposal(&id);
    assert_eq!(client.get_proposal(&id).state, ProposalState::Cancelled);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_cancel_already_executed_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    let voter = Address::generate(&env);
    mint_tokens(&env, &token, &admin, &voter, 600);
    client.vote(&voter, &id, &true);

    let deadline = client.get_proposal(&id).deadline;
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.execute_proposal(&id);

    client.cancel_proposal(&id);
}

// ---------------------------------------------------------------------------
// #830 — Proposer self-cancellation
// ---------------------------------------------------------------------------

/// Proposer can cancel their own proposal before any votes are cast.
#[test]
fn test_proposer_cancel_pre_vote_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "Mistake"),
        &String::from_str(&env, "Oops"),
    );

    // No votes cast yet — proposer self-cancel should succeed.
    client.proposer_cancel_proposal(&admin, &id);
    assert_eq!(client.get_proposal(&id).state, ProposalState::Cancelled);
}

/// Proposer self-cancel is rejected once any vote has been cast.
#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_proposer_cancel_after_vote_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    let voter = Address::generate(&env);
    mint_tokens(&env, &token, &admin, &voter, 600);
    client.vote(&voter, &id, &true);

    // At least one vote cast — proposer self-cancel must be rejected.
    client.proposer_cancel_proposal(&admin, &id);
}

/// A non-proposer cannot use `proposer_cancel_proposal`.
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_proposer_cancel_wrong_caller_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    let impostor = Address::generate(&env);
    // impostor is not the original proposer
    client.proposer_cancel_proposal(&impostor, &id);
}

/// Proposer self-cancel fails on an already-cancelled proposal.
#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_proposer_cancel_already_cancelled_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token, _) = setup(&env);

    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    client.proposer_cancel_proposal(&admin, &id);
    // Second call on an already-cancelled proposal must fail.
    client.proposer_cancel_proposal(&admin, &id);
}

// ---------------------------------------------------------------------------
// #829 — quorum_bps (percentage-based quorum)
// ---------------------------------------------------------------------------

/// Proposal passes when participation meets both absolute quorum and quorum_bps.
///
/// Setup: total supply = 1_000, quorum_bps = 5_000 (50%).
/// Voter holds 600 tokens (60% > 50%) and votes yes → should execute.
#[test]
fn test_execute_meets_quorum_bps() {
    let env = Env::default();
    env.mock_all_auths();
    // quorum=0 (disabled), quorum_bps=5_000 (50% of supply required).
    let (client, admin, token, _) = setup_with_quorum_bps(&env, 0, 5_000);

    // `create_proposal` requires the proposer to hold a nonzero balance, so
    // admin needs *some* tokens to propose — but those must not count toward
    // total supply when quorum_bps is checked at execution time, so admin
    // burns them again immediately after creating the proposal. Final total
    // supply = 600 (all held by `voter`).
    mint_tokens(&env, &token, &admin, &admin, 1_000);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );
    soroban_sdk::token::Client::new(&env, &token).burn(&admin, &1_000);

    let voter = Address::generate(&env);
    // Voter gets all 600 tokens of the remaining total supply: 600/600 = 100% ≥ 50%.
    mint_tokens(&env, &token, &admin, &voter, 600);
    client.vote(&voter, &id, &true);

    let deadline = client.get_proposal(&id).deadline;
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);

    // 600 / 600 = 100% ≥ 50% → quorum_bps met, should execute.
    client.execute_proposal(&id);
    assert_eq!(client.get_proposal(&id).state, ProposalState::Executed);
}

/// Proposal is rejected when participation is below quorum_bps.
///
/// Setup: total supply = 2_000, quorum_bps = 5_000 (50%).
/// Voter holds 500 tokens (25% < 50%) → QuorumNotMet.
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_execute_below_quorum_bps_fails() {
    let env = Env::default();
    env.mock_all_auths();
    // quorum=0, quorum_bps=5_000 (50%).
    let (client, admin, token, _) = setup_with_quorum_bps(&env, 0, 5_000);

    // Mint 2_000 total: 1_500 to admin (won't vote), 500 to voter.
    mint_tokens(&env, &token, &admin, &admin, 1_500);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    let voter = Address::generate(&env);
    mint_tokens(&env, &token, &admin, &voter, 500); // total supply = 2_000
    client.vote(&voter, &id, &true); // 500/2_000 = 25% < 50%

    let deadline = client.get_proposal(&id).deadline;
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.execute_proposal(&id); // must panic with QuorumNotMet (#8)
}

/// Both absolute quorum and quorum_bps are enforced simultaneously.
#[test]
fn test_execute_both_quorums_met() {
    let env = Env::default();
    env.mock_all_auths();
    // quorum=300 (absolute), quorum_bps=2_500 (25%).
    let (client, admin, token, _) = setup_with_quorum_bps(&env, 300, 2_500);

    // Total supply = 1_000: voter gets 400 (40% ≥ 25%, and 400 ≥ 300).
    mint_tokens(&env, &token, &admin, &admin, 600);
    let id = client.create_proposal(
        &admin,
        &String::from_str(&env, "P"),
        &String::from_str(&env, "D"),
    );

    let voter = Address::generate(&env);
    mint_tokens(&env, &token, &admin, &voter, 400);
    client.vote(&voter, &id, &true);

    let deadline = client.get_proposal(&id).deadline;
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);

    client.execute_proposal(&id);
    assert_eq!(client.get_proposal(&id).state, ProposalState::Executed);
}

/// `initialize` rejects quorum_bps > 10_000.
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_initialize_invalid_quorum_bps_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token = sac.address();
    let addr = env.register_contract(None, DaoContract);
    let client = DaoContractClient::new(&env, &addr);
    // quorum_bps = 10_001 is out of range
    client.initialize(&admin, &token, &100, &500, &10_001);
}
