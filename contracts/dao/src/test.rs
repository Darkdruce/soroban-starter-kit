#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects, clippy::indexing_slicing)]
#![cfg(test)]

use super::*;
use soroban_sdk::{
    Address, Env, String,
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup(env: &Env) -> (DaoContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token = sac.address();

    let addr = env.register_contract(None, DaoContract);
    let client = DaoContractClient::new(env, &addr);
    client.initialize(&admin, &token, &100, &500);

    (client, admin, token, addr)
}

fn mint_tokens(env: &Env, token: &Address, admin: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
    let _ = admin;
}

// ---------------------------------------------------------------------------
// Tests
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
    client.initialize(&admin, &token, &100, &500);
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
