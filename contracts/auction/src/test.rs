#![cfg(test)]

use super::*;
use soroban_sdk::{
    Address, Env,
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
};

fn setup(env: &Env) -> (AuctionContractClient, Address, Address, Address, Address) {
    let seller = Address::generate(env);
    let bidder1 = Address::generate(env);
    let bidder2 = Address::generate(env);

    let sac = env.register_stellar_asset_contract_v2(seller.clone());
    let token = sac.address();
    StellarAssetClient::new(env, &token).mint(&bidder1, &100_000);
    StellarAssetClient::new(env, &token).mint(&bidder2, &100_000);

    let addr = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(env, &addr);

    (client, seller, bidder1, bidder2, token)
}

// ---------------------------------------------------------------------------
// Happy-path / overbid scenario
// ---------------------------------------------------------------------------

#[test]
fn test_single_bid_and_settle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, b1, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 100;
    client.start(&seller, &token, &1_000, &100, &deadline);

    client.bid(&b1, &1_500);
    assert_eq!(client.get_info().highest_bid, 1_500);

    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.end();

    assert!(client.get_info().settled);
}

#[test]
fn test_overbid_refunds_previous_bidder() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, b1, b2, token) = setup(&env);

    let deadline = env.ledger().sequence() + 100;
    client.start(&seller, &token, &1_000, &100, &deadline);

    client.bid(&b1, &1_000);
    client.bid(&b2, &1_200); // overbids b1

    // b1 should have a pending refund of 1_000
    assert_eq!(client.get_pending(&b1), 1_000);
    assert_eq!(client.get_info().highest_bid, 1_200);

    // b1 withdraws refund
    client.withdraw(&b1);
    assert_eq!(client.get_pending(&b1), 0);
}

#[test]
fn test_multiple_overbids() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, b1, b2, token) = setup(&env);

    let deadline = env.ledger().sequence() + 100;
    client.start(&seller, &token, &1_000, &500, &deadline);

    client.bid(&b1, &1_000);
    client.bid(&b2, &1_500);
    client.bid(&b1, &2_000);

    // b2 is outbid; b2 pending = 1_500
    assert_eq!(client.get_pending(&b2), 1_500);
    assert_eq!(client.get_info().highest_bid, 2_000);

    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.end();
    assert!(client.get_info().settled);
}

// ---------------------------------------------------------------------------
// Deadline scenario
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_bid_after_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, b1, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 10;
    client.start(&seller, &token, &1_000, &100, &deadline);

    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.bid(&b1, &1_500);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_end_before_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, b1, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 100;
    client.start(&seller, &token, &1_000, &100, &deadline);
    client.bid(&b1, &1_500);
    client.end(); // deadline not reached
}

// ---------------------------------------------------------------------------
// No-bids scenario
// ---------------------------------------------------------------------------

#[test]
fn test_end_with_no_bids() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, _, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 10;
    client.start(&seller, &token, &1_000, &100, &deadline);

    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.end();

    let info = client.get_info();
    assert!(info.settled);
    assert!(info.highest_bidder.is_none());
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_bid_too_low_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, b1, b2, token) = setup(&env);

    let deadline = env.ledger().sequence() + 100;
    client.start(&seller, &token, &1_000, &500, &deadline);

    client.bid(&b1, &1_000);
    client.bid(&b2, &1_200); // needs >= 1_500 (1000 + 500)
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_double_settle_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, b1, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 10;
    client.start(&seller, &token, &1_000, &100, &deadline);
    client.bid(&b1, &1_000);
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.end();
    client.end();
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_start_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, _, _, token) = setup(&env);

    let deadline = env.ledger().sequence() + 100;
    client.start(&seller, &token, &1_000, &100, &deadline);
    client.start(&seller, &token, &1_000, &100, &deadline);
}
