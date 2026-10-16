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
    Address, Bytes, BytesN, Env, MuxedAddress, String, Vec,
    testutils::{Address as _, Ledger as _},
    token::TokenInterface,
};

// ---------------------------------------------------------------------------
// MockToken
// ---------------------------------------------------------------------------

#[contract]
pub struct MockToken;

#[contractimpl]
impl TokenInterface for MockToken {
    fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 {
        0
    }
    fn approve(_env: Env, _from: Address, _spender: Address, _amount: i128, _exp: u32) {}
    fn balance(_env: Env, _id: Address) -> i128 {
        i128::MAX
    }
    fn transfer(_env: Env, _from: Address, _to: MuxedAddress, _amount: i128) {}
    fn transfer_from(_env: Env, _spender: Address, _from: Address, _to: Address, _amount: i128) {}
    fn burn(_env: Env, _from: Address, _amount: i128) {}
    fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) {}
    fn decimals(_env: Env) -> u32 {
        7
    }
    fn name(env: Env) -> String {
        String::from_str(&env, "Mock")
    }
    fn symbol(env: Env) -> String {
        String::from_str(&env, "MCK")
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_commit(env: &Env, secret: &[u8; 32], salt: &[u8; 32]) -> BytesN<32> {
    let mut preimage = Bytes::new(env);
    preimage.extend_from_array(secret);
    preimage.extend_from_array(salt);
    env.crypto().sha256(&preimage).into()
}

fn single_winner_splits(env: &Env) -> Vec<u32> {
    Vec::from_array(env, [10_000u32])
}

fn two_winner_splits(env: &Env) -> Vec<u32> {
    Vec::from_array(env, [6_000u32, 4_000u32])
}

/// Set up a basic lottery with one winner, no ticket cap, no min_participants.
fn setup(env: &Env) -> (LotteryContractClient, Address, Address) {
    let admin = Address::generate(env);
    let token = env.register_contract(None, MockToken);
    let addr = env.register_contract(None, LotteryContract);
    let client = LotteryContractClient::new(env, &addr);
    client.initialize(
        &admin,
        &token,
        &100,
        &1u32,
        &single_winner_splits(env),
        &None,
        &None,
    );
    (client, admin, token)
}

/// Set up with a ticket cap per address.
fn setup_with_cap(env: &Env, cap: u32) -> (LotteryContractClient, Address, Address) {
    let admin = Address::generate(env);
    let token = env.register_contract(None, MockToken);
    let addr = env.register_contract(None, LotteryContract);
    let client = LotteryContractClient::new(env, &addr);
    client.initialize(
        &admin,
        &token,
        &100,
        &1u32,
        &single_winner_splits(env),
        &Some(cap),
        &None,
    );
    (client, admin, token)
}

/// Set up with min_participants.
fn setup_with_min(env: &Env, min: u32) -> (LotteryContractClient, Address, Address) {
    let admin = Address::generate(env);
    let token = env.register_contract(None, MockToken);
    let addr = env.register_contract(None, LotteryContract);
    let client = LotteryContractClient::new(env, &addr);
    client.initialize(
        &admin,
        &token,
        &100,
        &1u32,
        &single_winner_splits(env),
        &None,
        &Some(min),
    );
    (client, admin, token)
}

// ---------------------------------------------------------------------------
// Basic lifecycle tests
// ---------------------------------------------------------------------------

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token) = setup(&env);
    let info = client.get_info();
    assert_eq!(info.admin, admin);
    assert_eq!(info.token, token);
    assert_eq!(info.ticket_price, 100);
    assert_eq!(info.state, LotteryState::Open);
    assert_eq!(info.total_tickets, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token) = setup(&env);
    client.initialize(
        &admin,
        &token,
        &100,
        &1u32,
        &single_winner_splits(&env),
        &None,
        &None,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_initialize_zero_price_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = env.register_contract(None, MockToken);
    let addr = env.register_contract(None, LotteryContract);
    let client = LotteryContractClient::new(&env, &addr);
    client.initialize(
        &admin,
        &token,
        &0,
        &1u32,
        &single_winner_splits(&env),
        &None,
        &None,
    );
}

#[test]
fn test_buy_ticket() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);
    let buyer = Address::generate(&env);
    client.buy_ticket(&buyer);
    assert_eq!(client.get_info().total_tickets, 1);
    assert_eq!(client.get_ticket_count(&buyer), 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_commit_with_no_tickets_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);
    let secret = [1u8; 32];
    let salt = [2u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    client.commit(&hash, &(env.ledger().sequence() + 100));
}

#[test]
fn test_full_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);
    client.buy_ticket(&buyer1);
    client.buy_ticket(&buyer2);

    let secret = [42u8; 32];
    let salt = [99u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    client.commit(&hash, &(env.ledger().sequence() + 100));
    assert_eq!(client.get_info().state, LotteryState::Committed);

    let secret_bytes: BytesN<32> = BytesN::from_array(&env, &secret);
    let salt_bytes: BytesN<32> = BytesN::from_array(&env, &salt);
    let winners = client.draw(&secret_bytes, &salt_bytes);
    let winner = winners.get(0).unwrap();

    assert!(winner == buyer1 || winner == buyer2);
    assert_eq!(client.get_info().state, LotteryState::Drawn);
    assert_eq!(client.get_winner(), winner);
    assert_eq!(client.get_winners(), winners);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_draw_wrong_preimage_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let buyer = Address::generate(&env);
    client.buy_ticket(&buyer);

    let secret = [1u8; 32];
    let salt = [2u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    client.commit(&hash, &(env.ledger().sequence() + 100));

    let bad_secret: BytesN<32> = BytesN::from_array(&env, &[9u8; 32]);
    let salt_bytes: BytesN<32> = BytesN::from_array(&env, &salt);
    client.draw(&bad_secret, &salt_bytes);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_buy_ticket_after_commit_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let buyer = Address::generate(&env);
    client.buy_ticket(&buyer);

    let secret = [1u8; 32];
    let salt = [2u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    client.commit(&hash, &(env.ledger().sequence() + 100));

    let another = Address::generate(&env);
    client.buy_ticket(&another);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_draw_after_draw_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let buyer = Address::generate(&env);
    client.buy_ticket(&buyer);

    let secret = [1u8; 32];
    let salt = [2u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    client.commit(&hash, &(env.ledger().sequence() + 100));

    let secret_bytes: BytesN<32> = BytesN::from_array(&env, &secret);
    let salt_bytes: BytesN<32> = BytesN::from_array(&env, &salt);
    client.draw(&secret_bytes, &salt_bytes);
    client.draw(&secret_bytes, &salt_bytes);
}

// ---------------------------------------------------------------------------
// #1254 — buy_tickets_batch
// ---------------------------------------------------------------------------

#[test]
fn test_buy_tickets_batch_single_buyer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let buyer = Address::generate(&env);
    client.buy_tickets_batch(&buyer, &3);

    assert_eq!(client.get_info().total_tickets, 3);
    assert_eq!(client.get_ticket_count(&buyer), 3);
}

#[test]
fn test_buy_tickets_batch_multiple_buyers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);
    client.buy_tickets_batch(&buyer1, &2);
    client.buy_tickets_batch(&buyer2, &3);

    assert_eq!(client.get_info().total_tickets, 5);
    assert_eq!(client.get_ticket_count(&buyer1), 2);
    assert_eq!(client.get_ticket_count(&buyer2), 3);
}

#[test]
#[should_panic(expected = "Error(Contract, #18)")]
fn test_buy_tickets_batch_zero_count_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);
    let buyer = Address::generate(&env);
    client.buy_tickets_batch(&buyer, &0);
}

#[test]
fn test_buy_tickets_batch_respects_cap() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_with_cap(&env, 5);

    let buyer = Address::generate(&env);
    client.buy_tickets_batch(&buyer, &5);
    assert_eq!(client.get_ticket_count(&buyer), 5);
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_buy_tickets_batch_exceeds_cap_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_with_cap(&env, 5);

    let buyer = Address::generate(&env);
    client.buy_tickets_batch(&buyer, &6);
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_buy_tickets_batch_cumulative_cap_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_with_cap(&env, 5);

    let buyer = Address::generate(&env);
    client.buy_tickets_batch(&buyer, &3);
    // Second call pushes total to 6, over the cap of 5.
    client.buy_tickets_batch(&buyer, &3);
}

#[test]
fn test_buy_ticket_delegates_to_batch() {
    // buy_ticket should behave identically to buy_tickets_batch(_, _, 1).
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let buyer = Address::generate(&env);
    client.buy_ticket(&buyer);

    assert_eq!(client.get_ticket_count(&buyer), 1);
    assert_eq!(client.get_info().total_tickets, 1);
}

// ---------------------------------------------------------------------------
// #1252 — round bytes in entropy
// ---------------------------------------------------------------------------

#[test]
fn test_multi_winner_draw_selects_distinct_winners() {
    let env = Env::default();
    env.mock_all_auths();

    // 2 winners
    let admin = Address::generate(&env);
    let token = env.register_contract(None, MockToken);
    let lottery_addr = env.register_contract(None, LotteryContract);
    let client = LotteryContractClient::new(&env, &lottery_addr);
    client.initialize(
        &admin,
        &token,
        &100,
        &2u32,
        &two_winner_splits(&env),
        &None,
        &None,
    );

    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);
    let buyer3 = Address::generate(&env);
    client.buy_ticket(&buyer1);
    client.buy_ticket(&buyer2);
    client.buy_ticket(&buyer3);

    let secret = [7u8; 32];
    let salt = [13u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    client.commit(&hash, &(env.ledger().sequence() + 100));

    let secret_bytes: BytesN<32> = BytesN::from_array(&env, &secret);
    let salt_bytes: BytesN<32> = BytesN::from_array(&env, &salt);
    let winners = client.draw(&secret_bytes, &salt_bytes);

    assert_eq!(winners.len(), 2);
    // Two distinct winners.
    assert_ne!(winners.get(0).unwrap(), winners.get(1).unwrap());
}

#[test]
fn test_multi_winner_draw_winners_are_participants() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token = env.register_contract(None, MockToken);
    let lottery_addr = env.register_contract(None, LotteryContract);
    let client = LotteryContractClient::new(&env, &lottery_addr);
    client.initialize(
        &admin,
        &token,
        &50,
        &2u32,
        &two_winner_splits(&env),
        &None,
        &None,
    );

    let buyers: Vec<Address> = {
        let mut v = Vec::new(&env);
        for _ in 0..4 {
            let b = Address::generate(&env);
            client.buy_ticket(&b);
            v.push_back(b);
        }
        v
    };

    let secret = [3u8; 32];
    let salt = [5u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    client.commit(&hash, &(env.ledger().sequence() + 100));

    let secret_bytes: BytesN<32> = BytesN::from_array(&env, &secret);
    let salt_bytes: BytesN<32> = BytesN::from_array(&env, &salt);
    let winners = client.draw(&secret_bytes, &salt_bytes);

    for w in winners.iter() {
        assert!(buyers.contains(&w));
    }
}

// ---------------------------------------------------------------------------
// #1253 — indexed persistent storage
// ---------------------------------------------------------------------------

#[test]
fn test_ticket_count_persists_across_calls() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let buyer = Address::generate(&env);
    client.buy_ticket(&buyer);
    client.buy_ticket(&buyer);
    client.buy_ticket(&buyer);

    assert_eq!(client.get_ticket_count(&buyer), 3);
    assert_eq!(client.get_info().total_tickets, 3);
}

#[test]
fn test_multiple_buyers_total_tickets() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    for _ in 0..5 {
        let buyer = Address::generate(&env);
        client.buy_ticket(&buyer);
    }
    assert_eq!(client.get_info().total_tickets, 5);
}

// ---------------------------------------------------------------------------
// #1255 — rollover
// ---------------------------------------------------------------------------

#[test]
fn test_rollover_resets_to_open() {
    let env = Env::default();
    env.mock_all_auths();
    // min_participants = 3, but we only have 1 buyer.
    let (client, _admin, _token) = setup_with_min(&env, 3);

    let buyer = Address::generate(&env);
    client.buy_ticket(&buyer);

    let secret = [1u8; 32];
    let salt = [2u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    let deadline = env.ledger().sequence() + 10;
    client.commit(&hash, &deadline);

    assert_eq!(client.get_info().state, LotteryState::Committed);

    // Advance past the reveal deadline so any caller may trigger rollover.
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);

    client.rollover();

    assert_eq!(client.get_info().state, LotteryState::Open);
    // Tickets should still be counted.
    assert_eq!(client.get_info().total_tickets, 1);
}

#[test]
fn test_rollover_allows_new_tickets_and_draw() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token) = setup_with_min(&env, 2);

    // Round 1: only 1 buyer — rollover.
    let buyer1 = Address::generate(&env);
    client.buy_ticket(&buyer1);

    let secret = [1u8; 32];
    let salt = [2u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    let deadline = env.ledger().sequence() + 10;
    client.commit(&hash, &deadline);
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    client.rollover();

    // Round 2: add another buyer, now we have 2 distinct addresses.
    let buyer2 = Address::generate(&env);
    client.buy_ticket(&buyer2);

    let secret2 = [9u8; 32];
    let salt2 = [8u8; 32];
    let hash2 = make_commit(&env, &secret2, &salt2);
    let deadline2 = env.ledger().sequence() + 10;
    client.commit(&hash2, &deadline2);

    let secret_bytes: BytesN<32> = BytesN::from_array(&env, &secret2);
    let salt_bytes: BytesN<32> = BytesN::from_array(&env, &salt2);
    let winners = client.draw(&secret_bytes, &salt_bytes);
    assert_eq!(winners.len(), 1);
    let w = winners.get(0).unwrap();
    assert!(w == buyer1 || w == buyer2);
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")]
fn test_rollover_fails_when_enough_participants() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token) = setup_with_min(&env, 2);

    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);
    client.buy_ticket(&buyer1);
    client.buy_ticket(&buyer2);

    let secret = [1u8; 32];
    let salt = [2u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    let deadline = env.ledger().sequence() + 10;
    client.commit(&hash, &deadline);
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);

    // Should fail because we already have 2 distinct participants (>= min_participants=2).
    client.rollover();
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")]
fn test_rollover_fails_when_not_committed() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_with_min(&env, 3);
    // Still Open — rollover should not be available.
    client.rollover();
}

#[test]
fn test_rollover_without_min_uses_winner_count_fallback() {
    // Without min_participants, rollover should still be available when
    // distinct participants < winner_count.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token = env.register_contract(None, MockToken);
    let lottery_addr = env.register_contract(None, LotteryContract);
    let client = LotteryContractClient::new(&env, &lottery_addr);
    // 2 winners required, no explicit min_participants.
    client.initialize(
        &admin,
        &token,
        &100,
        &2u32,
        &two_winner_splits(&env),
        &None,
        &None,
    );

    // Only 1 distinct participant — below winner_count of 2.
    let buyer = Address::generate(&env);
    client.buy_ticket(&buyer);
    client.buy_ticket(&buyer);

    let secret = [1u8; 32];
    let salt = [2u8; 32];
    let hash = make_commit(&env, &secret, &salt);
    let deadline = env.ledger().sequence() + 10;
    client.commit(&hash, &deadline);
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);

    client.rollover();
    assert_eq!(client.get_info().state, LotteryState::Open);
}
