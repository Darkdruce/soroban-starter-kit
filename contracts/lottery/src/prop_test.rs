#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing
)]
#![cfg(test)]

extern crate std;

use std::format;

use crate::{LotteryContract, LotteryContractClient};
use proptest::prelude::*;
use soroban_sdk::{Address, Env, Vec, testutils::{Address as _, Ledger as _}, token::StellarAssetClient};

proptest! {
    /// Property: Total payouts never exceed total ticket sales.
    /// Closes #961 – lottery payout accounting invariant.
    #[test]
    fn prop_payouts_never_exceed_sales(
        ticket_price in 10i128..=100i128,
        num_tickets in 1u32..=20u32,
        winner_count in 1u32..=5u32,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.sequence_number = 100);

        let admin = Address::generate(&env);
        let sac_admin = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(sac_admin);
        let token_addr = sac.address();

        let lottery_addr = env.register_contract(None, LotteryContract);
        let client = LotteryContractClient::new(&env, &lottery_addr);

        // Build equal-weight splits that sum to 10_000.
        let splits: soroban_sdk::Vec<u32> = {
            let per = 10_000u32 / winner_count;
            let rem = 10_000u32 % winner_count;
            let mut v = Vec::new(&env);
            for i in 0..winner_count {
                v.push_back(per + if i == 0 { rem } else { 0 });
            }
            v
        };

        let init_result = client.try_initialize(
            &admin,
            &token_addr,
            &ticket_price,
            &winner_count,
            &splits,
            &None,
            &None,
        );

        if init_result.is_err() {
            return Ok(());
        }

        let mut total_sales = 0i128;

        // Buy tickets.
        for _ in 0..num_tickets {
            let participant = Address::generate(&env);
            StellarAssetClient::new(&env, &token_addr).mint(&participant, &ticket_price);

            if client.try_buy_ticket(&participant).is_ok() {
                total_sales += ticket_price;
            }
        }

        // Skip draw if no tickets were sold.
        let info = client.get_info();
        if info.total_tickets == 0 {
            return Ok(());
        }

        // Commit and draw.
        let secret = [42u8; 32];
        let salt = [99u8; 32];
        let mut preimage = soroban_sdk::Bytes::new(&env);
        preimage.extend_from_array(&secret);
        preimage.extend_from_array(&salt);
        let hash: soroban_sdk::BytesN<32> = env.crypto().sha256(&preimage).into();

        let deadline = env.ledger().sequence() + 100;
        let _ = client.try_commit(&hash, &deadline);

        let contract_balance_before =
            soroban_sdk::token::Client::new(&env, &token_addr).balance(&lottery_addr);

        let secret_bytes: soroban_sdk::BytesN<32> =
            soroban_sdk::BytesN::from_array(&env, &secret);
        let salt_bytes: soroban_sdk::BytesN<32> =
            soroban_sdk::BytesN::from_array(&env, &salt);
        let _ = client.try_draw(&secret_bytes, &salt_bytes);

        let contract_balance_after =
            soroban_sdk::token::Client::new(&env, &token_addr).balance(&lottery_addr);

        let total_paid_out = contract_balance_before - contract_balance_after;

        prop_assert!(
            total_paid_out <= total_sales,
            "Payouts exceeded sales: sales={}, payouts={}",
            total_sales,
            total_paid_out
        );
    }

    /// Property: Winner count validation — zero winners must be rejected.
    #[test]
    fn prop_winner_count_bounded(
        winner_count in 0u32..=100u32,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let sac_admin = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(sac_admin);
        let token_addr = sac.address();

        let lottery_addr = env.register_contract(None, LotteryContract);
        let client = LotteryContractClient::new(&env, &lottery_addr);

        // Build a plausible splits vector (may be wrong length — that's fine, it tests validation).
        let splits: soroban_sdk::Vec<u32> = {
            let mut v = Vec::new(&env);
            if winner_count > 0 {
                let per = 10_000u32 / winner_count;
                let rem = 10_000u32 % winner_count;
                for i in 0..winner_count {
                    v.push_back(per + if i == 0 { rem } else { 0 });
                }
            }
            v
        };

        let result = client.try_initialize(
            &admin,
            &token_addr,
            &100i128,
            &winner_count,
            &splits,
            &None,
            &None,
        );

        if winner_count == 0 {
            prop_assert!(result.is_err(), "Zero winners should be rejected");
        }
    }

    /// Property: Ticket price never non-positive.
    #[test]
    fn prop_ticket_price_positive(
        ticket_price in -1_000i128..=1_000i128,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let sac_admin = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(sac_admin);
        let token_addr = sac.address();

        let lottery_addr = env.register_contract(None, LotteryContract);
        let client = LotteryContractClient::new(&env, &lottery_addr);

        let splits: soroban_sdk::Vec<u32> = {
            let mut v = Vec::new(&env);
            v.push_back(10_000u32);
            v
        };

        let result = client.try_initialize(
            &admin,
            &token_addr,
            &ticket_price,
            &1u32,
            &splits,
            &None,
            &None,
        );

        if ticket_price <= 0 {
            prop_assert!(result.is_err(), "Non-positive ticket price should be rejected");
        }
    }

    /// Property: batch buying is equivalent to buying one-by-one.
    #[test]
    fn prop_batch_equals_individual(
        count in 1u32..=10u32,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let sac_admin = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(sac_admin);
        let token_addr = sac.address();

        let lottery_addr = env.register_contract(None, LotteryContract);
        let client = LotteryContractClient::new(&env, &lottery_addr);

        let splits: soroban_sdk::Vec<u32> = {
            let mut v = Vec::new(&env);
            v.push_back(10_000u32);
            v
        };

        client.initialize(
            &admin,
            &token_addr,
            &10i128,
            &1u32,
            &splits,
            &None,
            &None,
        );

        let buyer = Address::generate(&env);
        StellarAssetClient::new(&env, &token_addr).mint(&buyer, &(10i128 * count as i128));
        client.buy_tickets_batch(&buyer, &count);

        prop_assert_eq!(client.get_ticket_count(&buyer), count);
        prop_assert_eq!(client.get_info().total_tickets, count);
    }
}
