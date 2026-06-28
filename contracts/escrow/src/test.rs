#![cfg(test)]

use super::*;
use soroban_sdk::token::TokenInterface;
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env, FromVal, IntoVal, String, Symbol,
};

// ---------------------------------------------------------------------------
// MockToken — a no-op token contract so cross-contract calls don't panic.
// Balance defaults to i128::MAX; set DataKey::Balance(addr) to override.
// ---------------------------------------------------------------------------

#[contract]
pub struct MockToken;

#[contractimpl]
impl token::TokenInterface for MockToken {
    fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 {
        0
    }

    fn approve(
        _env: Env,
        _from: Address,
        _spender: Address,
        _amount: i128,
        _expiration_ledger: u32,
    ) {
    }

    fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .instance()
            .get::<Address, i128>(&id)
            .unwrap_or(i128::MAX)
    }

    fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}

    fn transfer_from(_env: Env, _spender: Address, _from: Address, _to: Address, _amount: i128) {}

    fn burn(_env: Env, _from: Address, _amount: i128) {}

    fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) {}

    fn decimals(_env: Env) -> u32 {
        18
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

fn create_escrow_contract<'a>(env: &'a Env) -> (EscrowContractClient<'a>, Address) {
    let contract_address = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(env, &contract_address);
    (client, contract_address)
}

fn create_mock_token(env: &Env) -> Address {
    env.register_contract(None, MockToken)
}

/// Returns (client, contract_address, buyer, seller, arbiter, token, amount).
fn setup_funded_escrow<'a>(
    env: &'a Env,
) -> (
    EscrowContractClient<'a>,
    Address,
    Address,
    Address,
    Address,
    Address,
    i128,
) {
    let buyer = Address::generate(env);
    let seller = Address::generate(env);
    let arbiter = Address::generate(env);
    let token = create_mock_token(env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, contract_address) = create_escrow_contract(env);
    client.initialize(&buyer, &seller, &arbiter, &token, &amount, &deadline);
    client.fund();

    (client, contract_address, buyer, seller, arbiter, token, amount)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token_contract = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, contract_address) = create_escrow_contract(&env);

    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);

    let info = client.get_escrow_info();
    assert_eq!(info.buyer, buyer);
    assert_eq!(info.seller, seller);
    assert_eq!(info.arbiter, arbiter);
    assert_eq!(info.token_contract, token_contract);
    assert_eq!(info.amount, amount);
    assert_eq!(info.deadline, deadline);
    assert_eq!(info.state, EscrowState::Created);

    assert_eq!(
        env.events().all(),
        soroban_sdk::vec![
            &env,
            (
                contract_address.clone(),
                (Symbol::new(&env, "escrow_created"), buyer.clone(), seller.clone()).into_val(&env),
                amount.into_val(&env),
            ),
            (
                contract_address.clone(),
                (Symbol::new(&env, "initialized"), buyer.clone(), seller.clone(), arbiter.clone()).into_val(&env),
                amount.into_val(&env),
            ),
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token_contract = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, _) = create_escrow_contract(&env);

    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);
    // Second call must fail with AlreadyInitialized (#5)
    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);
}

#[test]
#[should_panic]
fn test_initialize_past_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.sequence_number = 10);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token_contract = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = 5u32; // 5 < 10, already in the past

    let (client, _) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_initialize_buyer_equals_seller_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let same = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token = create_mock_token(&env);
    let deadline = env.ledger().sequence() + 100;
    let (client, _) = create_escrow_contract(&env);
    client.initialize(&same, &same, &arbiter, &token, &1_000, &deadline);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_initialize_buyer_equals_arbiter_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let same = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = create_mock_token(&env);
    let deadline = env.ledger().sequence() + 100;
    let (client, _) = create_escrow_contract(&env);
    client.initialize(&same, &seller, &same, &token, &1_000, &deadline);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_initialize_seller_equals_arbiter_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let buyer = Address::generate(&env);
    let same = Address::generate(&env);
    let token = create_mock_token(&env);
    let deadline = env.ledger().sequence() + 100;
    let (client, _) = create_escrow_contract(&env);
    client.initialize(&buyer, &same, &same, &token, &1_000, &deadline);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_initialize_zero_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token = create_mock_token(&env);
    let deadline = env.ledger().sequence() + 100;

    let (client, _) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token, &0, &deadline);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_initialize_negative_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token = create_mock_token(&env);
    let deadline = env.ledger().sequence() + 100;

    let (client, _) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token, &-1, &deadline);
}

#[test]
fn test_initialize_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, buyer, seller, _, _, amount) = setup_funded_escrow(&env);

    let info = client.get_escrow_info();
    assert_eq!(info.buyer, buyer);
    assert_eq!(info.seller, seller);
    assert_eq!(info.amount, amount);
    assert_eq!(info.state, EscrowState::Funded);
}

#[test]
fn test_fund() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token_contract = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, contract_address) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);
    client.fund();

    assert_eq!(client.get_state(), Some(EscrowState::Funded));

    // Verify funded event was emitted
    assert_eq!(
        env.events().all(),
        soroban_sdk::vec![
            &env,
            (
                contract_address.clone(),
                (Symbol::new(&env, "created"), buyer.clone(), seller.clone()).into_val(&env),
                amount.into_val(&env),
            ),
            (
                contract_address.clone(),
                (Symbol::new(&env, "initialized"), buyer.clone(), seller.clone(), arbiter.clone()).into_val(&env),
                amount.into_val(&env),
            ),
            (
                contract_address.clone(),
                (Symbol::new(&env, "funded"), buyer.clone()).into_val(&env),
                amount.into_val(&env),
            ),
        ]
    );
}

#[test]
fn test_mark_delivered() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    client.mark_delivered();

    assert_eq!(client.get_state(), Some(EscrowState::Delivered));
}

#[test]
#[should_panic]
fn test_mark_delivered_by_buyer_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _, _, _, _, _) = setup_funded_escrow(&env);
    // Clear auths so seller auth is not present — mark_delivered requires seller
    env.set_auths(&[]);
    client.mark_delivered();
}

#[test]
#[should_panic]
fn test_mark_delivered_by_arbiter_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _, _, _, _, _) = setup_funded_escrow(&env);
    // Clear auths so seller auth is not present — mark_delivered requires seller
    env.set_auths(&[]);
    client.mark_delivered();
}

#[test]
fn test_approve_delivery() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    client.mark_delivered();
    client.approve_delivery();

    assert_eq!(client.get_state(), Some(EscrowState::Completed));
}

#[test]
fn test_raise_dispute() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, buyer, ..) = setup_funded_escrow(&env);
    client.raise_dispute(&buyer);

    assert_eq!(client.get_state(), Some(EscrowState::Disputed));
}

#[test]
fn test_seller_raises_dispute() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _buyer, seller, ..) = setup_funded_escrow(&env);
    client.raise_dispute(&seller);

    assert_eq!(client.get_state(), Some(EscrowState::Disputed));
}

#[test]
fn test_resolve_dispute_to_seller() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, buyer, ..) = setup_funded_escrow(&env);
    client.raise_dispute(&buyer);
    client.resolve_dispute(&true);

    assert_eq!(client.get_state(), Some(EscrowState::Completed));
}

#[test]
fn test_resolve_dispute_to_buyer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, buyer, ..) = setup_funded_escrow(&env);
    client.raise_dispute(&buyer);
    client.resolve_dispute(&false);

    assert_eq!(client.get_state(), Some(EscrowState::Refunded));
}

#[test]
fn test_deadline_passed() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token_contract = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, _) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);

    assert_eq!(client.is_deadline_passed(), false);

    env.ledger().with_mut(|li| li.sequence_number = deadline + 1);

    assert_eq!(client.is_deadline_passed(), true);
}

#[test]
fn test_arbiter_resolve_to_seller() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_address, buyer, seller, _, _, amount) = setup_funded_escrow(&env);
    client.raise_dispute(&buyer);
    client.resolve_dispute(&true);

    assert_eq!(client.get_state(), Some(EscrowState::Completed));
    assert!(!env.events().all().is_empty());
}

#[test]
fn test_arbiter_resolve_to_buyer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, buyer, ..) = setup_funded_escrow(&env);
    client.raise_dispute(&buyer);
    client.resolve_dispute(&false);

    assert_eq!(client.get_state(), Some(EscrowState::Refunded));
    assert!(!env.events().all().is_empty());
}

#[test]
#[should_panic]
fn test_initialize_invalid_token_address() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    // Use a random address that has no contract — decimals() will panic.
    let invalid_token = Address::generate(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, _) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &invalid_token, &amount, &deadline);
}

#[test]
#[should_panic]
fn test_cancel_by_seller_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, contract_address) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token, &amount, &deadline);

    // Only authorize the seller — buyer.require_auth() inside cancel() will fail.
    use soroban_sdk::testutils::{MockAuth, MockAuthInvoke};
    env.mock_auths(&[MockAuth {
        address: &seller,
        invoke: &MockAuthInvoke {
            contract: &contract_address,
            fn_name: "cancel",
            args: soroban_sdk::vec![&env].into(),
            sub_invokes: &[],
        },
    }]);
    client.cancel();
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_cancel_after_funded_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    // State is now Funded; cancel() requires Created state → InvalidState (#2).
    client.cancel();
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_fund_insufficient_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    // Set buyer's balance to 0 in the mock token's storage
    env.as_contract(&token, || {
        env.storage().instance().set(&buyer, &0i128);
    });

    let (client, _) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token, &amount, &deadline);
    // buyer has balance 0 < amount 1000 → InsufficientFunds (#7)
    client.fund();
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_get_escrow_info_uninitialized_fails() {
    let env = Env::default();
    let (client, _) = create_escrow_contract(&env);
    // Calling get_escrow_info on uninitialized contract should panic with NotInitialized (#6)
    let _ = client.get_escrow_info();
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_bump_uninitialized_fails() {
    let env = Env::default();
    let (client, _) = create_escrow_contract(&env);
    // Calling bump on uninitialized contract should fail with NotInitialized (#6)
    let _ = client.bump();
}

#[test]
fn test_bump_initialized_succeeds() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    // bump should succeed on initialized escrow
    client.bump();
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_fund_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    // Already Funded; calling fund again must fail with InvalidState (#2)
    client.fund();
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_request_refund_before_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    // Deadline is sequence + 100; current sequence is 0 → deadline not reached → DeadlineNotReached (#4)
    client.request_refund();
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_approve_delivery_without_mark_delivered_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    // Escrow is Funded; approve_delivery requires Delivered state → InvalidState (#2)
    client.approve_delivery();
}

#[test]
#[should_panic]
fn test_approve_delivery_by_seller_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _buyer, seller, ..) = setup_funded_escrow(&env);
    client.mark_delivered();
    
    // Clear auths and only authorize seller — approve_delivery requires buyer auth
    use soroban_sdk::testutils::{MockAuth, MockAuthInvoke};
    let contract_address = env.register_contract(None, EscrowContract);
    env.mock_auths(&[MockAuth {
        address: &seller,
        invoke: &MockAuthInvoke {
            contract: &contract_address,
            fn_name: "approve_delivery",
            args: soroban_sdk::vec![&env].into(),
            sub_invokes: &[],
        },
    }]);
    client.approve_delivery();
}

#[test]
#[should_panic]
fn test_approve_delivery_by_arbiter_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _buyer, _seller, arbiter, ..) = setup_funded_escrow(&env);
    client.mark_delivered();
    
    // Clear auths and only authorize arbiter — approve_delivery requires buyer auth
    use soroban_sdk::testutils::{MockAuth, MockAuthInvoke};
    let contract_address = env.register_contract(None, EscrowContract);
    env.mock_auths(&[MockAuth {
        address: &arbiter,
        invoke: &MockAuthInvoke {
            contract: &contract_address,
            fn_name: "approve_delivery",
            args: soroban_sdk::vec![&env].into(),
            sub_invokes: &[],
        },
    }]);
    client.approve_delivery();
}

#[test]
fn test_release_partial() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _buyer, _seller, _arbiter, _token, amount) = setup_funded_escrow(&env);
    let partial = amount / 2;
    
    client.release_partial(&partial);
    
    // Verify state is still Funded
    assert_eq!(client.get_state(), Some(EscrowState::Funded));
    
    // Verify amount was decremented
    let info = client.get_escrow_info();
    assert_eq!(info.amount, amount - partial);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_release_partial_invalid_state() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, _) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token, &amount, &deadline);
    
    // Try to release_partial in Created state — should fail with InvalidState
    client.release_partial(&500i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_release_partial_exceeds_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _buyer, _seller, _arbiter, _token, amount) = setup_funded_escrow(&env);
    
    // Try to release more than available — should fail with InsufficientFunds
    client.release_partial(&(amount + 1));
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_release_partial_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _buyer, _seller, _arbiter, _token, _amount) = setup_funded_escrow(&env);
    
    // Try to release zero amount — should fail with InvalidAmount
    client.release_partial(&0i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_resolve_dispute_wrong_state_fails() {
    let env = Env::default();
    env.mock_all_auths();

    // Escrow is Funded but never Disputed — resolve_dispute must reject with InvalidState (#2)
    let (client, ..) = setup_funded_escrow(&env);
    client.resolve_dispute(&true);
}

// ---------------------------------------------------------------------------
// Feature-gated tests
// ---------------------------------------------------------------------------

#[cfg(feature = "pausable")]
mod pausable_tests {
    use super::*;
    use soroban_common::AdminKey;

    fn setup_with_admin<'a>(
        env: &'a Env,
    ) -> (EscrowContractClient<'a>, Address, Address) {
        let admin = Address::generate(env);
        let buyer = Address::generate(env);
        let seller = Address::generate(env);
        let arbiter = Address::generate(env);
        let token = create_mock_token(env);
        let amount = 1_000i128;
        let deadline = env.ledger().sequence() + 100;

        let (client, contract_address) = create_escrow_contract(env);
        // Set admin directly in contract storage before initializing
        env.as_contract(&contract_address, || {
            env.storage().instance().set(&AdminKey::Admin, &admin);
        });
        client.initialize(&buyer, &seller, &arbiter, &token, &amount, &deadline);
        (client, admin, buyer)
    }

    #[test]
    fn test_pause_blocks_fund() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, _buyer) = setup_with_admin(&env);

        client.pause();
        assert!(client.try_fund().is_err());
    }

    #[test]
    fn test_unpause_restores_fund() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, _buyer) = setup_with_admin(&env);

        client.pause();
        client.unpause();
        client.fund();
        assert_eq!(client.get_state(), Some(EscrowState::Funded));
    }

    #[test]
    fn test_pause_emits_event() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, _buyer) = setup_with_admin(&env);

        client.pause();

        use soroban_sdk::{testutils::Events as _, IntoVal, Symbol};
        let all = env.events().all();
        let last = all.last().unwrap();
        let (_, topics, _) = last;
        assert_eq!(topics, (Symbol::new(&env, "paused"), admin).into_val(&env));
    }

    #[test]
    fn test_unpause_emits_event() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, _buyer) = setup_with_admin(&env);

        client.pause();
        client.unpause();

        use soroban_sdk::{testutils::Events as _, IntoVal, Symbol};
        let all = env.events().all();
        let last = all.last().unwrap();
        let (_, topics, _) = last;
        assert_eq!(topics, (Symbol::new(&env, "unpaused"), admin).into_val(&env));
    }
}

#[cfg(feature = "upgradeable")]
mod upgradeable_tests {
    use super::*;
    use soroban_common::AdminKey;

    #[test]
    fn test_upgrade_requires_admin() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, contract_address, ..) = setup_funded_escrow(&env);
        let admin = Address::generate(&env);
        env.as_contract(&contract_address, || {
            env.storage().instance().set(&AdminKey::Admin, &admin);
        });
        let dummy_hash = soroban_sdk::BytesN::from_array(&env, &[0u8; 32]);
        // Auth passes; the upgrade itself fails because the hash is invalid.
        let _ = client.try_upgrade(&dummy_hash);
    }

    #[test]
    fn test_upgrade_emits_event() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, contract_address, ..) = setup_funded_escrow(&env);
        let admin = Address::generate(&env);
        env.as_contract(&contract_address, || {
            env.storage().instance().set(&AdminKey::Admin, &admin);
        });
        let dummy_hash = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);
        // The upgraded event is emitted before update_current_contract_wasm, so
        // even though the call fails (invalid hash), the event is still captured.
        let _ = client.try_upgrade(&dummy_hash);

        use soroban_sdk::{testutils::Events as _, IntoVal, Symbol};
        let all = env.events().all();
        // Find the upgraded event
        let found = all.iter().any(|(_, topics, _)| {
            topics == (Symbol::new(&env, "upgraded"), admin.clone()).into_val(&env)
        });
        assert!(found, "upgraded event not emitted");
    }
}

// Tests for new features
#[test]
fn test_update_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token = create_mock_token(&env);
    let deadline = env.ledger().sequence() + 100;
    let (client, _) = create_escrow_contract(&env);

    client.initialize(&buyer, &seller, &arbiter, &token, &1_000, &deadline);
    
    // Update amount before funding
    client.update_amount(&2_000);
    
    let info = client.get_escrow_info();
    assert_eq!(info.amount, 2_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_update_amount_zero_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token = create_mock_token(&env);
    let deadline = env.ledger().sequence() + 100;
    let (client, _) = create_escrow_contract(&env);

    client.initialize(&buyer, &seller, &arbiter, &token, &1_000, &deadline);
    client.update_amount(&0);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_update_amount_after_funding_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, ..) = setup_funded_escrow(&env);
    
    // Try to update amount after funding
    client.update_amount(&2_000);
}

#[test]
fn test_initialize_with_arbiters() {
    let env = Env::default();
    env.mock_all_auths();
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter1 = Address::generate(&env);
    let arbiter2 = Address::generate(&env);
    let arbiter3 = Address::generate(&env);
    let token = create_mock_token(&env);
    let deadline = env.ledger().sequence() + 100;
    let (client, _) = create_escrow_contract(&env);

    let arbiters = soroban_sdk::vec![&env, arbiter1.clone(), arbiter2.clone(), arbiter3.clone()];
    client.initialize_with_arbiters(&buyer, &seller, &arbiters, &token, &1_000, &deadline, &2);
    
    let info = client.get_escrow_info();
    assert_eq!(info.amount, 1_000);
}
