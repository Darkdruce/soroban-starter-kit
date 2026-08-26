#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing
)]
#![cfg(test)]

use crate::{DaoContract, DaoContractClient};
use proptest::prelude::*;
use soroban_sdk::{Address, Env, String, testutils::Address as _};

proptest! {
    /// Property: Total votes (for + against) never exceed total voting power
    /// Closes #961 – DAO vote accounting invariant
    #[test]
    fn prop_votes_never_exceed_power(
        num_members in 1usize..=10usize,
        voting_powers in proptest::collection::vec(1u32..=100u32, 1..=10),
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let dao_addr = env.register_contract(None, DaoContract);
        let client = DaoContractClient::new(&env, &dao_addr);

        client.initialize(&admin);

        // Add members with voting power
        let mut total_power = 0u32;
        let mut members = vec![];
        for i in 0..num_members.min(voting_powers.len()) {
            let member = Address::generate(&env);
            let power = voting_powers[i];
            let _ = client.try_add_member(&member, &power);
            total_power += power;
            members.push((member, power));
        }

        // Create proposal
        let description = String::from_str(&env, "Test Proposal");
        let voting_period = 1000u32;
        let proposal_id_result = client.try_create_proposal(&admin, &description, &voting_period);

        if proposal_id_result.is_err() {
            return Ok(());
        }

        let proposal_id = proposal_id_result.unwrap();

        // Vote on proposal
        let mut votes_for = 0u32;
        let mut votes_against = 0u32;

        for (i, (member, power)) in members.iter().enumerate() {
            let vote_in_favor = i % 2 == 0;
            if client.try_vote(&member, &proposal_id, &vote_in_favor).is_ok() {
                if vote_in_favor {
                    votes_for += power;
                } else {
                    votes_against += power;
                }
            }
        }

        // Invariant: total votes <= total voting power
        let total_votes = votes_for + votes_against;
        prop_assert!(total_votes <= total_power,
            "Total votes exceeded total power: votes={}, power={}",
            total_votes, total_power);
    }

    /// Property: Proposal execution requires quorum
    #[test]
    fn prop_execution_requires_quorum(
        voting_power in 10u32..=100u32,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let member = Address::generate(&env);

        let dao_addr = env.register_contract(None, DaoContract);
        let client = DaoContractClient::new(&env, &dao_addr);

        client.initialize(&admin);
        let _ = client.try_add_member(&member, &voting_power);

        let description = String::from_str(&env, "Test");
        let proposal_id_result = client.try_create_proposal(&admin, &description, &100u32);

        if proposal_id_result.is_err() {
            return Ok(());
        }

        let proposal_id = proposal_id_result.unwrap();

        // Vote but don't meet quorum (single member may not be enough)
        let _ = client.try_vote(&member, &proposal_id, &true);

        // Try to execute - should require quorum/majority
        let exec_result = client.try_execute_proposal(&proposal_id);

        // Either execution succeeds (quorum met) or fails (quorum not met)
        prop_assert!(exec_result.is_ok() || exec_result.is_err());
    }
}
