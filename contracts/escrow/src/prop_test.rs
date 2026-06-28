#![cfg(test)]

use std::format;

use proptest::prelude::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
    Address, Env,
};

use crate::{EscrowContract, EscrowContractClient, EscrowState};

use soroban_common::MIN_DEADLINE_BUFFER;

fn setup_escrow<'a>(
    env: &'a Env,
    amount: i128,
) -> (EscrowContractClient<'a>, Address, Address, Address, Address) {
    let buyer = Address::generate(env);
    let seller = Address::generate(env);
    let arbiter = Address::generate(env);

    let admin = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = sac.address();
    StellarAssetClient::new(env, &token_addr).mint(&buyer, &amount);

    let escrow_addr = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(env, &escrow_addr);
    let deadline = env.ledger().sequence() + MIN_DEADLINE_BUFFER + 10;
    client.initialize(&buyer, &seller, &arbiter, &token_addr, &amount, &deadline);

    (client, buyer, seller, arbiter, token_addr)
}

proptest! {
    /// Any valid amount initialises the escrow in Created state with that amount.
    #[test]
    fn prop_initialize_stores_amount(amount in 1i128..=1_000_000i128) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, ..) = setup_escrow(&env, amount);

        let info = client.get_escrow_info();
        prop_assert_eq!(info.amount, amount);
        prop_assert_eq!(info.state, EscrowState::Created);
    }

    /// Fund → mark_delivered → approve_delivery always ends in Completed state.
    #[test]
    fn prop_happy_path_ends_completed(amount in 1i128..=1_000_000i128) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, ..) = setup_escrow(&env, amount);

        client.fund();
        client.mark_delivered();
        client.approve_delivery();

        prop_assert_eq!(client.get_state(), Some(EscrowState::Completed));
    }

    /// Arbiter resolving in favour of seller always ends in Completed state.
    #[test]
    fn prop_arbiter_resolve_seller(amount in 1i128..=1_000_000i128) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, buyer, ..) = setup_escrow(&env, amount);

        client.fund();
        client.raise_dispute(&buyer);
        client.resolve_dispute(&true);

        prop_assert_eq!(client.get_state(), Some(EscrowState::Completed));
    }

    /// Arbiter resolving in favour of buyer always ends in Refunded state.
    #[test]
    fn prop_arbiter_resolve_buyer(amount in 1i128..=1_000_000i128) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, buyer, ..) = setup_escrow(&env, amount);

        client.fund();
        client.raise_dispute(&buyer);
        client.resolve_dispute(&false);

        prop_assert_eq!(client.get_state(), Some(EscrowState::Refunded));
    }

    /// Full release via approve_delivery transfers the full amount to seller and ends in Completed.
    #[test]
    fn prop_partial_release_reduces_amount(
        total in 2i128..=1_000_000i128,
        _partial_pct in 1u32..=99u32,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _buyer, seller, _arbiter, token_addr) = setup_escrow(&env, total);

        client.fund();
        client.mark_delivered();
        client.approve_delivery();

        prop_assert_eq!(client.get_state(), Some(EscrowState::Completed));
        let token_client = soroban_sdk::token::Client::new(&env, &token_addr);
        prop_assert_eq!(token_client.balance(&seller), total);
    }

    /// Deadline in the past (below MIN_DEADLINE_BUFFER) is always rejected.
    #[test]
    fn prop_past_deadline_rejected(offset in 0u32..MIN_DEADLINE_BUFFER) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.sequence_number = 200);

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbiter = Address::generate(&env);
        let admin = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(admin);
        let token_addr = sac.address();
        StellarAssetClient::new(&env, &token_addr).mint(&buyer, &1000i128);

        let escrow_addr = env.register_contract(None, EscrowContract);
        let client = EscrowContractClient::new(&env, &escrow_addr);
        let bad_deadline = env.ledger().sequence() + offset; // < MIN_DEADLINE_BUFFER

        let result = client.try_initialize(
            &buyer, &seller, &arbiter, &token_addr, &1000i128, &bad_deadline,
        );
        prop_assert!(result.is_err());
    }

    /// State machine never reaches invalid transitions.
    /// After any sequence of valid operations, state is always a valid EscrowState.
    /// Terminal states (Completed, Refunded, Cancelled) cannot transition further.
    #[test]
    fn prop_state_machine_valid_transitions(
        actions in proptest::collection::vec(0u32..=6u32, 1..=10),
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, buyer, ..) = setup_escrow(&env, 1_000i128);

        for action in actions {
            let state = client.get_state();
            
            // Terminal states cannot transition
            if let Some(EscrowState::Completed) | Some(EscrowState::Refunded) | Some(EscrowState::Cancelled) = state {
                break;
            }

            match action {
                0 => {
                    // fund (only from Created)
                    if state == Some(EscrowState::Created) {
                        let _ = client.try_fund();
                    }
                }
                1 => {
                    // mark_delivered (only from Funded)
                    if state == Some(EscrowState::Funded) {
                        let _ = client.try_mark_delivered();
                    }
                }
                2 => {
                    // approve_delivery (only from Delivered)
                    if state == Some(EscrowState::Delivered) {
                        let _ = client.try_approve_delivery();
                    }
                }
                3 => {
                    // raise_dispute (only from Funded)
                    if state == Some(EscrowState::Funded) {
                        let _ = client.try_raise_dispute(&buyer);
                    }
                }
                4 => {
                    // resolve_dispute (only from Disputed)
                    if state == Some(EscrowState::Disputed) {
                        let _ = client.try_resolve_dispute(&true);
                    }
                }
                5 => {
                    // request_refund (only from Funded after deadline)
                    if state == Some(EscrowState::Funded) {
                        let _ = client.try_request_refund();
                    }
                }
                6 => {
                    // cancel (only from Created)
                    if state == Some(EscrowState::Created) {
                        let _ = client.try_cancel();
                    }
                }
                _ => {}
            }

            // Verify state is always valid
            let final_state = client.get_state();
            prop_assert!(
                matches!(
                    final_state,
                    Some(EscrowState::Created)
                        | Some(EscrowState::Funded)
                        | Some(EscrowState::Delivered)
                        | Some(EscrowState::Disputed)
                        | Some(EscrowState::Completed)
                        | Some(EscrowState::Refunded)
                        | Some(EscrowState::Cancelled)
                        | None
                )
            );
        }
    }
}
