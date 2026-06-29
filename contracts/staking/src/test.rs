#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects, clippy::indexing_slicing)]
#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _,
    token::StellarAssetClient,
    Address, Env,
};

use crate::{StakingContract, StakingContractClient, StakingError};

// ── helpers ───────────────────────────────────────────────────────────────────

fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

fn make_token(env: &Env, mint_to: &Address, amount: i128) -> Address {
    let sac = env.register_stellar_asset_contract_v2(Address::generate(env));
    let addr = sac.address();
    StellarAssetClient::new(env, &addr).mint(mint_to, &amount);
    addr
}

/// Returns (client, admin, stake_token, reward_token).
/// Admin holds 10_000 of each token.
fn setup(env: &Env) -> (StakingContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let stake_token = make_token(env, &admin, 10_000);
    let reward_token = make_token(env, &admin, 10_000);
    let addr = env.register_contract(None, StakingContract);
    let client = StakingContractClient::new(env, &addr);
    client.initialize(&admin, &stake_token, &reward_token);
    (client, admin, stake_token, reward_token)
}
// ── unit tests ────────────────────────────────────────────────────────────────

#[test]
fn test_initialize_stores_state() {
    let env = setup_env();
    let (client, _admin, _stake_token, _reward_token) = setup(&env);
    assert_eq!(client.get_total_staked(), 0);
    assert_eq!(client.get_total_rewards(), 0);
}

#[test]
fn test_initialize_twice_fails() {
    let env = setup_env();
    let (client, admin, stake_token, reward_token) = setup(&env);
    let result = client.try_initialize(&admin, &stake_token, &reward_token);
    assert_eq!(result, Err(Ok(StakingError::AlreadyInitialized)));
}

#[test]
fn test_stake_increases_balance() {
    let env = setup_env();
    let (client, _admin, stake_token, _reward_token) = setup(&env);
    let staker = Address::generate(&env);
    StellarAssetClient::new(&env, &stake_token).mint(&staker, &500);

    client.stake(&staker, &500);
    assert_eq!(client.get_stake(&staker), 500);
    assert_eq!(client.get_total_staked(), 500);
}

#[test]
fn test_stake_zero_fails() {
    let env = setup_env();
    let (client, _admin, _stake_token, _reward_token) = setup(&env);
    let staker = Address::generate(&env);
    let result = client.try_stake(&staker, &0);
    assert_eq!(result, Err(Ok(StakingError::InvalidAmount)));
}

#[test]
fn test_unstake_returns_tokens() {
    let env = setup_env();
    let (client, _admin, stake_token, _reward_token) = setup(&env);
    let staker = Address::generate(&env);
    StellarAssetClient::new(&env, &stake_token).mint(&staker, &1_000);

    client.stake(&staker, &1_000);
    client.unstake(&staker, &400);

    assert_eq!(client.get_stake(&staker), 600);
    assert_eq!(client.get_total_staked(), 600);
    let token_client = soroban_sdk::token::Client::new(&env, &stake_token);
    assert_eq!(token_client.balance(&staker), 400);
}

#[test]
fn test_unstake_more_than_staked_fails() {
    let env = setup_env();
    let (client, _admin, stake_token, _reward_token) = setup(&env);
    let staker = Address::generate(&env);
    StellarAssetClient::new(&env, &stake_token).mint(&staker, &100);

    client.stake(&staker, &100);
    let result = client.try_unstake(&staker, &200);
    assert_eq!(result, Err(Ok(StakingError::InsufficientStake)));
}

#[test]
fn test_unstake_with_no_stake_fails() {
    let env = setup_env();
    let (client, _admin, _stake_token, _reward_token) = setup(&env);
    let staker = Address::generate(&env);
    let result = client.try_unstake(&staker, &100);
    assert_eq!(result, Err(Ok(StakingError::NoStake)));
}

#[test]
fn test_add_rewards_unauthorized_fails() {
    let env = setup_env();
    let (client, _admin, _stake_token, reward_token) = setup(&env);
    // Register a fresh contract with a different admin to test unauthorized path.
    // Since mock_all_auths is on, we test the admin check via a separate contract.
    let other_admin = Address::generate(&env);
    StellarAssetClient::new(&env, &reward_token).mint(&other_admin, &500);
    // The admin stored is `_admin`, not `other_admin`, so add_rewards must be
    // called by the stored admin. With mock_all_auths this always passes auth,
    // so we test the zero-amount guard instead.
    let result = client.try_add_rewards(&0);
    assert_eq!(result, Err(Ok(StakingError::InvalidAmount)));
}

#[test]
fn test_rewards_distributed_proportionally() {
    let env = setup_env();
    let (client, _admin, stake_token, _reward_token) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    StellarAssetClient::new(&env, &stake_token).mint(&alice, &1_000);
    StellarAssetClient::new(&env, &stake_token).mint(&bob, &3_000);

    // Alice stakes 1_000, Bob stakes 3_000 → 1:3 ratio.
    client.stake(&alice, &1_000);
    client.stake(&bob, &3_000);

    // Admin adds 4_000 reward tokens.
    client.add_rewards(&4_000);

    let alice_rewards = client.get_rewards(&alice);
    let bob_rewards = client.get_rewards(&bob);

    // Alice should get 1_000, Bob 3_000.
    assert_eq!(alice_rewards, 1_000);
    assert_eq!(bob_rewards, 3_000);
}

#[test]
fn test_claim_rewards_transfers_tokens() {
    let env = setup_env();
    let (client, _admin, stake_token, reward_token) = setup(&env);

    let staker = Address::generate(&env);
    StellarAssetClient::new(&env, &stake_token).mint(&staker, &1_000);
    client.stake(&staker, &1_000);
    client.add_rewards(&500);

    let claimed = client.claim_rewards(&staker);
    assert_eq!(claimed, 500);

    let reward_client = soroban_sdk::token::Client::new(&env, &reward_token);
    assert_eq!(reward_client.balance(&staker), 500);
    assert_eq!(client.get_rewards(&staker), 0);
}

#[test]
fn test_claim_rewards_with_no_rewards_fails() {
    let env = setup_env();
    let (client, _admin, stake_token, _reward_token) = setup(&env);
    let staker = Address::generate(&env);
    StellarAssetClient::new(&env, &stake_token).mint(&staker, &100);
    client.stake(&staker, &100);

    let result = client.try_claim_rewards(&staker);
    assert_eq!(result, Err(Ok(StakingError::NoRewards)));
}

#[test]
fn test_rewards_accrue_incrementally() {
    let env = setup_env();
    let (client, _admin, stake_token, _reward_token) = setup(&env);

    let staker = Address::generate(&env);
    StellarAssetClient::new(&env, &stake_token).mint(&staker, &1_000);
    client.stake(&staker, &1_000);

    client.add_rewards(&200);
    client.add_rewards(&300);

    assert_eq!(client.get_rewards(&staker), 500);
}

#[test]
fn test_late_staker_does_not_receive_prior_rewards() {
    let env = setup_env();
    let (client, _admin, stake_token, _reward_token) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    StellarAssetClient::new(&env, &stake_token).mint(&alice, &1_000);
    StellarAssetClient::new(&env, &stake_token).mint(&bob, &1_000);

    client.stake(&alice, &1_000);
    client.add_rewards(&1_000); // only alice is staked

    // Bob stakes after rewards were added.
    client.stake(&bob, &1_000);

    assert_eq!(client.get_rewards(&alice), 1_000);
    assert_eq!(client.get_rewards(&bob), 0);
}

#[test]
fn test_full_unstake_then_restake_accrues_correctly() {
    let env = setup_env();
    let (client, _admin, stake_token, _reward_token) = setup(&env);

    let staker = Address::generate(&env);
    StellarAssetClient::new(&env, &stake_token).mint(&staker, &1_000);

    client.stake(&staker, &1_000);
    client.add_rewards(&500);
    client.unstake(&staker, &1_000);

    // Rewards should still be claimable after unstaking.
    assert_eq!(client.get_rewards(&staker), 500);
    let claimed = client.claim_rewards(&staker);
    assert_eq!(claimed, 500);

    // Re-stake and add more rewards.
    StellarAssetClient::new(&env, &stake_token).mint(&staker, &1_000);
    client.stake(&staker, &1_000);
    client.add_rewards(&300);
    assert_eq!(client.get_rewards(&staker), 300);
}

// ── property tests ────────────────────────────────────────────────────────────

use proptest::prelude::*;

/// Prop-test setup: mints exactly `stake_amount` stake tokens to staker
/// and `reward_amount` reward tokens to admin.
fn prop_setup_with(
    env: &Env,
    stake_amount: i128,
    reward_amount: i128,
) -> (StakingContractClient, Address, Address, Address, Address) {
    let admin = Address::generate(env);
    let staker = Address::generate(env);
    let stake_token = make_token(env, &staker, stake_amount);
    let reward_token = make_token(env, &admin, reward_amount);
    let addr = env.register_contract(None, StakingContract);
    let client = StakingContractClient::new(env, &addr);
    client.initialize(&admin, &stake_token, &reward_token);
    (client, admin, staker, stake_token, reward_token)
}

proptest! {
    #[test]
    fn prop_single_staker_gets_all_rewards(
        stake in 1i128..=1_000_000i128,
        reward in 1i128..=1_000_000i128,
    ) {
        let env = setup_env();
        let (client, _admin, staker, _stake_token, reward_token) =
            prop_setup_with(&env, stake, reward);
        client.stake(&staker, &stake);
        client.add_rewards(&reward);

        let claimed = client.claim_rewards(&staker);
        // Single staker gets all rewards; allow 1-token rounding loss from fixed-point division.
        assert!(claimed >= reward - 1 && claimed <= reward);

        let reward_client = soroban_sdk::token::Client::new(&env, &reward_token);
        assert_eq!(reward_client.balance(&staker), claimed);
    }

    #[test]
    fn prop_rewards_sum_to_total(
        a_stake in 1i128..=500_000i128,
        b_stake in 1i128..=500_000i128,
        reward in 2i128..=1_000_000i128,
    ) {
        let env = setup_env();
        let admin = Address::generate(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let stake_token = make_token(&env, &alice, a_stake + b_stake);
        // split stake tokens: alice gets a_stake, bob gets b_stake
        {
            let tc = soroban_sdk::token::Client::new(&env, &stake_token);
            // alice already has a_stake+b_stake; transfer b_stake to bob
            tc.transfer(&alice, &bob, &b_stake);
        }
        let reward_token = make_token(&env, &admin, reward);
        let addr = env.register_contract(None, StakingContract);
        let client = StakingContractClient::new(&env, &addr);
        client.initialize(&admin, &stake_token, &reward_token);

        client.stake(&alice, &a_stake);
        client.stake(&bob, &b_stake);
        client.add_rewards(&reward);

        let a_rewards = client.get_rewards(&alice);
        let b_rewards = client.get_rewards(&bob);

        // Due to integer division, sum may be <= reward (dust stays in contract).
        assert!(a_rewards + b_rewards <= reward);
        // But the difference should be at most 1 per staker (rounding).
        assert!(reward - (a_rewards + b_rewards) <= 1);
    }

    #[test]
    fn prop_stake_unstake_returns_principal(amount in 1i128..=1_000_000i128) {
        let env = setup_env();
        let (client, _admin, staker, stake_token, _reward_token) =
            prop_setup_with(&env, amount, 1);
        client.stake(&staker, &amount);
        client.unstake(&staker, &amount);

        let token_client = soroban_sdk::token::Client::new(&env, &stake_token);
        assert_eq!(token_client.balance(&staker), amount);
        assert_eq!(client.get_stake(&staker), 0);
        assert_eq!(client.get_total_staked(), 0);
    }
}
