#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects, clippy::indexing_slicing)]
#![cfg(test)]

use soroban_sdk::{
    Address, Env,
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
};

use crate::{VestingContract, VestingContractClient, VestingError};

// ── helpers ──────────────────────────────────────────────────────────────────

pub(crate) fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.sequence_number = 100);
    env
}

pub(crate) fn make_token(env: &Env, mint_to: &Address, amount: i128) -> Address {
    let sac = env.register_stellar_asset_contract_v2(Address::generate(env));
    let addr = sac.address();
    StellarAssetClient::new(env, &addr).mint(mint_to, &amount);
    addr
}

pub(crate) fn setup(
    env: &Env,
) -> (
    VestingContractClient,
    Address,
    Address,
    Address,
    u32,
    u32,
    i128,
) {
    let admin = Address::generate(env);
    let beneficiary = Address::generate(env);
    let amount = 1_000i128;
    let token = make_token(env, &admin, amount * 2); // Mint extra to allow for potential multiple schedules
    let cliff = env.ledger().sequence() + 10;
    let end = cliff + 100;
    let addr = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(env, &addr);
    client.initialize(&admin, &token);
    client.create_schedule(&beneficiary, &cliff, &end, &amount);
    (client, admin, beneficiary, token, cliff, end, amount)
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[test]
fn test_initialize_stores_info() {
    let env = setup_env();
    let (client, _admin, beneficiary, _token, cliff, end, amount) = setup(&env);
    let info = client.get_info(&beneficiary).unwrap();
    assert_eq!(info.amount, amount);
    assert_eq!(info.cliff_ledger, cliff);
    assert_eq!(info.end_ledger, end);
    assert_eq!(info.claimed, 0);
    assert!(!info.revoked);
}

#[test]
fn test_initialize_twice_fails() {
    let env = setup_env();
    let (client, admin, beneficiary, token, cliff, end, _) = setup(&env);
    let result = client.try_initialize(&admin, &beneficiary, &token, &cliff, &end, &100i128);
    assert_eq!(result, Err(Ok(VestingError::AlreadyInitialized)));
}

#[test]
fn test_initialize_zero_amount_fails() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token = make_token(&env, &admin, 0);
    let addr = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &addr);
    let result = client.try_initialize(&admin, &beneficiary, &token, &110u32, &200u32, &0i128);
    assert_eq!(result, Err(Ok(VestingError::InvalidAmount)));
}

#[test]
fn test_initialize_invalid_schedule_fails() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token = make_token(&env, &admin, 1000);
    let addr = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &addr);
    // cliff >= end
    let result = client.try_initialize(&admin, &beneficiary, &token, &200u32, &150u32, &1000i128);
    assert_eq!(result, Err(Ok(VestingError::InvalidSchedule)));
}

#[test]
fn test_claim_before_cliff_fails() {
    let env = setup_env();
    let (client, ..) = setup(&env);
    let result = client.try_claim();
    assert_eq!(result, Err(Ok(VestingError::NothingToClaim)));
}

#[test]
fn test_claim_at_cliff_returns_zero() {
    let env = setup_env();
    let (client, _admin, _beneficiary, _token, cliff, _end, _amount) = setup(&env);
    env.ledger().with_mut(|l| l.sequence_number = cliff);
    let result = client.try_claim();
    assert_eq!(result, Err(Ok(VestingError::NothingToClaim)));
}

#[test]
fn test_claim_halfway_through_vesting() {
    let env = setup_env();
    let (client, _admin, _beneficiary, _token, cliff, end, amount) = setup(&env);
    let mid = cliff + (end - cliff) / 2;
    env.ledger().with_mut(|l| l.sequence_number = mid);
    let claimed = client.claim();
    assert!(claimed > 0 && claimed <= amount / 2 + 1);
}

#[test]
fn test_claim_after_end_returns_full_amount() {
    let env = setup_env();
    let (client, _admin, beneficiary, token, _cliff, end, amount) = setup(&env);
    env.ledger().with_mut(|l| l.sequence_number = end + 1);
    let claimed = client.claim();
    assert_eq!(claimed, amount);
    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&beneficiary), amount);
}

#[test]
fn test_double_claim_second_returns_nothing() {
    let env = setup_env();
    let (client, _admin, _beneficiary, _token, _cliff, end, _amount) = setup(&env);
    env.ledger().with_mut(|l| l.sequence_number = end + 1);
    client.claim();
    let result = client.try_claim();
    assert_eq!(result, Err(Ok(VestingError::NothingToClaim)));
}

#[test]
fn test_revoke_before_cliff_returns_all() {
    let env = setup_env();
    let (client, admin, _beneficiary, token, _cliff, _end, amount) = setup(&env);
    let returned = client.revoke();
    assert_eq!(returned, amount);
    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&admin), amount);
}

#[test]
fn test_revoke_after_end_returns_nothing() {
    let env = setup_env();
    let (client, _admin, _beneficiary, _token, _cliff, end, _amount) = setup(&env);
    env.ledger().with_mut(|l| l.sequence_number = end + 1);
    let returned = client.revoke();
    assert_eq!(returned, 0);
}

#[test]
fn test_revoke_midway_returns_unvested_portion() {
    let env = setup_env();
    let (client, admin, _beneficiary, token, cliff, end, amount) = setup(&env);
    let mid = cliff + (end - cliff) / 2;
    env.ledger().with_mut(|l| l.sequence_number = mid);
    let returned = client.revoke();
    assert!(returned > 0 && returned < amount);
    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&admin), returned);
}

#[test]
fn test_claim_after_revoke_gets_vested_portion() {
    let env = setup_env();
    let (client, _admin, _beneficiary, _token, cliff, end, amount) = setup(&env);
    let mid = cliff + (end - cliff) / 2;
    env.ledger().with_mut(|l| l.sequence_number = mid);
    let returned = client.revoke();
    let claimed = client.claim();
    assert_eq!(claimed + returned, amount);
}

#[test]
fn test_revoke_twice_fails() {
    let env = setup_env();
    let (client, ..) = setup(&env);
    client.revoke();
    let result = client.try_revoke();
    assert_eq!(result, Err(Ok(VestingError::AlreadyRevoked)));
}

#[test]
fn test_claim_after_full_revoke_fails() {
    let env = setup_env();
    let (client, ..) = setup(&env);
    // revoke before cliff — nothing vested, amount capped to 0
    client.revoke();
    let result = client.try_claim();
    assert_eq!(result, Err(Ok(VestingError::NothingToClaim)));
}

#[test]
fn test_get_info_uninitialized_returns_none() {
    let env = setup_env();
    let addr = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &addr);
    assert_eq!(client.get_info(), None);
}

#[test]
fn test_claimable_before_cliff_is_zero() {
    let env = setup_env();
    let (client, ..) = setup(&env);
    assert_eq!(client.claimable(), 0);
}

#[test]
fn test_claimable_after_end_is_full_amount() {
    let env = setup_env();
    let (client, _admin, _beneficiary, _token, _cliff, end, amount) = setup(&env);
    env.ledger().with_mut(|l| l.sequence_number = end + 1);
    assert_eq!(client.claimable(), amount);
}

// ── property tests ────────────────────────────────────────────────────────────

use proptest::prelude::*;

fn prop_setup(
    env: &Env,
    amount: i128,
) -> (VestingContractClient, Address, Address, Address, u32, u32) {
    let admin = Address::generate(env);
    let beneficiary = Address::generate(env);
    let token = make_token(env, &admin, amount);
    let cliff = env.ledger().sequence() + 10;
    let end = cliff + 100;
    let addr = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(env, &addr);
    client.initialize(&admin, &beneficiary, &token, &cliff, &end, &amount);
    (client, admin, beneficiary, token, cliff, end)
}

proptest! {
    #[test]
    fn prop_initialize_stores_amount(amount in 1i128..=1_000_000i128) {
        let env = setup_env();
        let (client, ..) = prop_setup(&env, amount);
        let info = client.get_info().unwrap();
        assert_eq!(info.amount, amount);
        assert_eq!(info.claimed, 0);
        assert!(!info.revoked);
    }

    #[test]
    fn prop_claim_after_end_yields_full(amount in 1i128..=1_000_000i128) {
        let env = setup_env();
        let (client, _admin, beneficiary, token, _cliff, end) = prop_setup(&env, amount);
        env.ledger().with_mut(|l| l.sequence_number = end + 1);
        let claimed = client.claim();
        assert_eq!(claimed, amount);
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        assert_eq!(token_client.balance(&beneficiary), amount);
    }

    #[test]
    fn prop_revoke_before_cliff_returns_all(amount in 1i128..=1_000_000i128) {
        let env = setup_env();
        let (client, admin, _beneficiary, token, _cliff, _end) = prop_setup(&env, amount);
        let returned = client.revoke();
        assert_eq!(returned, amount);
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        assert_eq!(token_client.balance(&admin), amount);
    }

    #[test]
    fn prop_revoke_plus_claim_equals_total(
        amount in 2i128..=1_000_000i128,
        pct in 0u32..=100u32,
    ) {
        let env = setup_env();
        let (client, _admin, _beneficiary, _token, cliff, end) = prop_setup(&env, amount);
        let ledger = cliff + (end - cliff) * pct / 100;
        env.ledger().with_mut(|l| l.sequence_number = ledger);
        let returned = client.revoke();
        let claimed = client.try_claim().unwrap_or(Ok(0)).unwrap_or(0);
        assert_eq!(returned + claimed, amount);
    }

    #[test]
    fn prop_claimable_monotone(
        amount in 1i128..=1_000_000i128,
        t1_pct in 0u32..=100u32,
        t2_pct in 0u32..=100u32,
    ) {
        let env = setup_env();
        let (client, ..) = prop_setup(&env, amount);
        let info = client.get_info().unwrap();
        let cliff = info.cliff_ledger;
        let end = info.end_ledger;

        let l1 = cliff + (end - cliff) * t1_pct / 100;
        let l2 = cliff + (end - cliff) * t2_pct / 100;

        env.ledger().with_mut(|l| l.sequence_number = l1);
        let c1 = client.claimable();
        env.ledger().with_mut(|l| l.sequence_number = l2);
        let c2 = client.claimable();

        if l2 >= l1 {
            assert!(c2 >= c1);
        } else {
            assert!(c2 <= c1);
        }
    }
}

// ── #712 admin_release tests ─────────────────────────────────────────────────

#[test]
fn test_admin_release_before_cliff_sends_all_to_beneficiary() {
    let env = setup_env();
    let (client, _admin, beneficiary, token, _cliff, _end, amount) = setup(&env);
    // Still before cliff (ledger = 100, cliff = 110)
    let released = client.admin_release();
    assert_eq!(released, amount);
    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&beneficiary), amount);
}

#[test]
fn test_admin_release_marks_revoked() {
    let env = setup_env();
    let (client, ..) = setup(&env);
    client.admin_release();
    let info = client.get_info().unwrap();
    assert!(info.revoked);
}

#[test]
fn test_admin_release_after_cliff_fails() {
    let env = setup_env();
    let (client, _admin, _beneficiary, _token, cliff, _end, _amount) = setup(&env);
    // Advance past cliff
    env.ledger().with_mut(|l| l.sequence_number = cliff);
    let result = client.try_admin_release();
    assert_eq!(result, Err(Ok(VestingError::CliffAlreadyPassed)));
}

#[test]
fn test_admin_release_twice_fails() {
    let env = setup_env();
    let (client, ..) = setup(&env);
    client.admin_release();
    let result = client.try_admin_release();
    assert_eq!(result, Err(Ok(VestingError::AlreadyRevoked)));
}

#[test]
fn test_admin_release_emits_event() {
    let env = setup_env();
    let (client, admin, _beneficiary, _token, _cliff, _end, amount) = setup(&env);
    client.admin_release();
    use soroban_sdk::{IntoVal, Symbol, testutils::Events as _};
    let all = env.events().all();
    let found = all.iter().any(|(_, topics, data)| {
        topics == (Symbol::new(&env, "admin_released"), admin.clone()).into_val(&env)
            && data == amount.into_val(&env)
    });
    assert!(found, "admin_released event not emitted");
}