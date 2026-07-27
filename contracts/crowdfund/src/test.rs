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
    token::StellarAssetClient,
};

fn setup(env: &Env) -> (CrowdfundContractClient, Address, Address, Address, Address) {
    let creator = Address::generate(env);
    let contributor1 = Address::generate(env);
    let contributor2 = Address::generate(env);

    let sac = env.register_stellar_asset_contract_v2(creator.clone());
    let token = sac.address();
    StellarAssetClient::new(env, &token).mint(&contributor1, &10_000);
    StellarAssetClient::new(env, &token).mint(&contributor2, &10_000);

    let addr = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(env, &addr);

    (client, creator, contributor1, contributor2, token)
}

// ---------------------------------------------------------------------------
// Goal-met path
// ---------------------------------------------------------------------------

#[test]
fn test_goal_met_creator_claims() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, creator, c1, c2, token) = setup(&env);

    let deadline = env.ledger().sequence() + 100;
    let goal = 5_000_i128;
    client.initialize(&creator, &token, &goal, &deadline, &Vec::new(&env), &None);

    client.pledge(&c1, &3_000);
    client.pledge(&c2, &2_500);

    assert_eq!(client.get_pledge(&c1), 3_000);
    assert_eq!(client.get_info().total_pledged, 5_500);

    // Advance past deadline
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.claim();

    // Claimed flag set
    assert!(client.get_info().claimed);
}

#[test]
fn test_pledge_increments_total() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, creator, c1, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 50;
    client.initialize(&creator, &token, &1_000, &deadline, &Vec::new(&env), &None);

    client.pledge(&c1, &400);
    assert_eq!(client.get_info().total_pledged, 400);
    client.pledge(&c1, &200);
    assert_eq!(client.get_info().total_pledged, 600);
    assert_eq!(client.get_pledge(&c1), 600);
}

#[test]
fn test_withdraw_before_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, creator, c1, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 50;
    client.initialize(&creator, &token, &10_000, &deadline, &Vec::new(&env), &None);

    client.pledge(&c1, &3_000);
    client.withdraw(&c1);

    assert_eq!(client.get_pledge(&c1), 0);
    assert_eq!(client.get_info().total_pledged, 0);
}

// ---------------------------------------------------------------------------
// Goal-not-met path
// ---------------------------------------------------------------------------

#[test]
fn test_goal_not_met_contributors_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, creator, c1, c2, token) = setup(&env);

    let deadline = env.ledger().sequence() + 100;
    client.initialize(&creator, &token, &10_000, &deadline, &Vec::new(&env), &None);

    client.pledge(&c1, &1_000);
    client.pledge(&c2, &500);

    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);

    client.refund(&c1);
    client.refund(&c2);

    assert_eq!(client.get_pledge(&c1), 0);
    assert_eq!(client.get_pledge(&c2), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_pledge_after_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, creator, c1, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 10;
    client.initialize(&creator, &token, &1_000, &deadline, &Vec::new(&env), &None);

    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.pledge(&c1, &500);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_claim_before_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, creator, c1, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 100;
    client.initialize(&creator, &token, &500, &deadline, &Vec::new(&env), &None);
    client.pledge(&c1, &1_000);
    // deadline not reached
    client.claim();
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_claim_goal_not_met_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, creator, c1, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 10;
    client.initialize(&creator, &token, &10_000, &deadline, &Vec::new(&env), &None);
    client.pledge(&c1, &100);
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.claim();
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_refund_when_goal_met_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, creator, c1, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 10;
    client.initialize(&creator, &token, &500, &deadline, &Vec::new(&env), &None);
    client.pledge(&c1, &1_000);
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.refund(&c1);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_initialize_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, creator, _, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 100;
    client.initialize(&creator, &token, &1_000, &deadline, &Vec::new(&env), &None);
    client.initialize(&creator, &token, &1_000, &deadline, &Vec::new(&env), &None);
}
