#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects, clippy::indexing_slicing)]
#![cfg(test)]

use super::*;
use soroban_sdk::{
    Address, Env, FromVal, IntoVal, Symbol, contract, contractimpl,
    testutils::{Address as _, Events as _},
    vec,
};

#[contract]
pub struct CounterContract;

#[contractimpl]
impl CounterContract {
    pub fn increment(env: Env, amount: u32) -> u32 {
        let current = Self::get(env.clone());
        let next = current + amount;
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "count"), &next);
        next
    }

    pub fn get(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&Symbol::new(&env, "count"))
            .unwrap_or(0)
    }
}

fn create_multisig<'a>(
    env: &'a Env,
) -> (
    MultisigContractClient<'a>,
    Address,
    Address,
    Address,
    Address,
) {
    let alice = Address::generate(env);
    let bob = Address::generate(env);
    let carol = Address::generate(env);
    let contract_address = env.register_contract(None, MultisigContract);
    let client = MultisigContractClient::new(env, &contract_address);

    client.initialize(&vec![env, alice.clone(), bob.clone(), carol.clone()], &2);

    (client, alice, bob, carol, contract_address)
}

#[test]
fn initialize_stores_signers_and_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, alice, bob, carol, contract_address) = create_multisig(&env);

    assert_eq!(client.get_threshold(), Some(2));
    assert_eq!(client.get_signers(), vec![&env, alice, bob, carol]);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_address,
                (Symbol::new(&env, "initialized"), 2u32).into_val(&env),
                3u32.into_val(&env),
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn initialize_rejects_zero_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    let contract_address = env.register_contract(None, MultisigContract);
    let client = MultisigContractClient::new(&env, &contract_address);

    client.initialize(&vec![&env, alice], &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn initialize_rejects_duplicate_signers() {
    let env = Env::default();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    let contract_address = env.register_contract(None, MultisigContract);
    let client = MultisigContractClient::new(&env, &contract_address);

    client.initialize(&vec![&env, alice.clone(), alice], &1);
}

#[test]
fn add_signer_with_threshold_approvals_updates_signer_set() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, bob, carol, _) = create_multisig(&env);
    let dave = Address::generate(&env);

    client.add_signer(&vec![&env, alice.clone(), bob.clone()], &dave, &3);

    assert_eq!(client.get_threshold(), Some(3));
    assert_eq!(client.get_signers(), vec![&env, alice, bob, carol, dave]);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn add_signer_rejects_insufficient_approvals() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, _, _, _) = create_multisig(&env);
    let dave = Address::generate(&env);

    client.add_signer(&vec![&env, alice], &dave, &2);
}

#[test]
fn remove_signer_with_threshold_approvals_updates_signer_set() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, bob, carol, _) = create_multisig(&env);

    client.remove_signer(&vec![&env, alice.clone(), bob.clone()], &carol, &2);

    assert_eq!(client.get_threshold(), Some(2));
    assert_eq!(client.get_signers(), vec![&env, alice, bob]);
    assert!(!client.is_signer(&carol));
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn remove_signer_rejects_threshold_above_remaining_signers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, bob, carol, _) = create_multisig(&env);

    client.remove_signer(&vec![&env, alice, bob], &carol, &3);
}

#[test]
fn propose_transaction_stores_transaction_and_auto_signature() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, _, _, _) = create_multisig(&env);
    let target = env.register_contract(None, CounterContract);

    let tx_id = client.propose_transaction(
        &alice,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 7u32.into_val(&env)],
        &100u32,
    );

    let transaction = client.get_transaction(&tx_id).expect("transaction exists");
    assert_eq!(tx_id, 0);
    assert_eq!(transaction.proposer, alice.clone());
    assert_eq!(transaction.target, target);
    assert_eq!(transaction.signatures, vec![&env, alice]);
    assert!(!transaction.executed);
    assert_eq!(client.signature_count(&tx_id), Some(1));
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn non_signer_cannot_propose_transaction() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, _) = create_multisig(&env);
    let outsider = Address::generate(&env);
    let target = env.register_contract(None, CounterContract);

    client.propose_transaction(
        &outsider,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 1u32.into_val(&env)],
        &100u32,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn signer_cannot_sign_same_transaction_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, _, _, _) = create_multisig(&env);
    let target = env.register_contract(None, CounterContract);
    let tx_id = client.propose_transaction(
        &alice,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 1u32.into_val(&env)],
        &100u32,
    );

    client.sign_transaction(&alice, &tx_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn execute_rejects_when_threshold_not_met() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, _, _, _) = create_multisig(&env);
    let target = env.register_contract(None, CounterContract);
    let tx_id = client.propose_transaction(
        &alice,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 1u32.into_val(&env)],
        &100u32,
    );

    client.execute_transaction(&tx_id);
}

#[test]
fn execute_runs_target_call_once_when_threshold_met() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, bob, _, _) = create_multisig(&env);
    let target = env.register_contract(None, CounterContract);
    let counter = CounterContractClient::new(&env, &target);
    let tx_id = client.propose_transaction(
        &alice,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 5u32.into_val(&env)],
        &100u32,
    );

    client.sign_transaction(&bob, &tx_id);
    let result = client.execute_transaction(&tx_id);
    let value = u32::from_val(&env, &result);

    assert_eq!(value, 5);
    assert_eq!(counter.get(), 5);
    assert!(
        client
            .get_transaction(&tx_id)
            .expect("transaction exists")
            .executed
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn execute_rejects_second_execution() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, bob, _, _) = create_multisig(&env);
    let target = env.register_contract(None, CounterContract);
    let tx_id = client.propose_transaction(
        &alice,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 5u32.into_val(&env)],
        &100u32,
    );

    client.sign_transaction(&bob, &tx_id);
    client.execute_transaction(&tx_id);
    client.execute_transaction(&tx_id);
}

// ── #715 proposal expiry tests ───────────────────────────────────────────────

#[test]
fn proposal_stores_expiry_ledger() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, _, _, _) = create_multisig(&env);
    let target = env.register_contract(None, CounterContract);

    let tx_id = client.propose_transaction(
        &alice,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 1u32.into_val(&env)],
        &50u32,
    );

    let tx = client.get_transaction(&tx_id).expect("transaction exists");
    assert_eq!(tx.expiry_ledger, env.ledger().sequence() + 50);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn sign_after_expiry_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, bob, _, _) = create_multisig(&env);
    let target = env.register_contract(None, CounterContract);

    let tx_id = client.propose_transaction(
        &alice,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 1u32.into_val(&env)],
        &10u32,
    );

    // Advance ledger past expiry
    env.ledger().with_mut(|l| l.sequence_number += 11);
    client.sign_transaction(&bob, &tx_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn execute_after_expiry_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, bob, _, _) = create_multisig(&env);
    let target = env.register_contract(None, CounterContract);

    let tx_id = client.propose_transaction(
        &alice,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 1u32.into_val(&env)],
        &10u32,
    );
    client.sign_transaction(&bob, &tx_id);

    // Advance ledger past expiry
    env.ledger().with_mut(|l| l.sequence_number += 11);
    client.execute_transaction(&tx_id);
}

#[test]
fn cleanup_expired_removes_proposal_and_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, _, _, contract_address) = create_multisig(&env);
    let target = env.register_contract(None, CounterContract);

    let tx_id = client.propose_transaction(
        &alice,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 1u32.into_val(&env)],
        &10u32,
    );

    // Advance past expiry
    env.ledger().with_mut(|l| l.sequence_number += 11);
    client.cleanup_expired(&tx_id);

    // Proposal should be gone
    assert_eq!(client.get_transaction(&tx_id), None);

    // expired event emitted
    let found = env.events().all().iter().any(|(_, topics, _)| {
        topics == (Symbol::new(&env, "expired"), tx_id).into_val(&env)
    });
    assert!(found, "expired event not emitted");
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn cleanup_not_yet_expired_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, alice, _, _, _) = create_multisig(&env);
    let target = env.register_contract(None, CounterContract);

    let tx_id = client.propose_transaction(
        &alice,
        &target,
        &Symbol::new(&env, "increment"),
        &vec![&env, 1u32.into_val(&env)],
        &100u32,
    );

    // Not yet expired — should fail
    client.cleanup_expired(&tx_id);
}
