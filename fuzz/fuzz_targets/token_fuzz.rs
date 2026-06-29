#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects, clippy::indexing_slicing)]
#![no_main]
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use soroban_token_template::TokenContract;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_addr = env.register_contract(None, TokenContract);
    let client = soroban_token_template::TokenContractClient::new(&env, &token_addr);

    // Initialize token
    let _ = client.try_initialize(
        &admin,
        &String::from_str(&env, "Fuzz Token"),
        &String::from_str(&env, "FZZ"),
        &18u32,
        &None,
    );

    // Use first byte to select operation
    let op = data[0] % 6;
    let amount = i128::from_le_bytes([
        data.get(1).copied().unwrap_or(0),
        data.get(2).copied().unwrap_or(0),
        data.get(3).copied().unwrap_or(0),
        data.get(4).copied().unwrap_or(0),
        data.get(5).copied().unwrap_or(0),
        data.get(6).copied().unwrap_or(0),
        data.get(7).copied().unwrap_or(0),
        data.get(8).copied().unwrap_or(0),
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]);

    let to = Address::generate(&env);
    let spender = Address::generate(&env);

    match op {
        0 => {
            // Mint
            let _ = client.try_mint(&to, &amount);
        }
        1 => {
            // Burn
            let _ = client.try_burn(&admin, &amount);
        }
        2 => {
            // Transfer
            let _ = client.try_transfer(&admin, &to, &amount);
        }
        3 => {
            // Approve
            let _ = client.try_approve(&admin, &spender, &amount, &1000u32);
        }
        4 => {
            // Transfer from
            let _ = client.try_transfer_from(&spender, &admin, &to, &amount);
        }
        5 => {
            // Balance
            let _ = client.try_balance(&to);
        }
        _ => {}
    }
});
