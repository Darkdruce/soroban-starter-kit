#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects, clippy::indexing_slicing)]
#![no_main]
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use soroban_token_template::TokenContract;

fn bytes_to_i128(data: &[u8], offset: usize) -> i128 {
    i128::from_le_bytes([
        data.get(offset).copied().unwrap_or(0),
        data.get(offset + 1).copied().unwrap_or(0),
        data.get(offset + 2).copied().unwrap_or(0),
        data.get(offset + 3).copied().unwrap_or(0),
        data.get(offset + 4).copied().unwrap_or(0),
        data.get(offset + 5).copied().unwrap_or(0),
        data.get(offset + 6).copied().unwrap_or(0),
        data.get(offset + 7).copied().unwrap_or(0),
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ])
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 25 {
        return;
    }

    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let holder = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_addr = env.register_contract(None, TokenContract);
    let client = soroban_token_template::TokenContractClient::new(&env, &token_addr);
    let _ = client.try_initialize(
        &admin,
        &String::from_str(&env, "Fuzz Token"),
        &String::from_str(&env, "FZZ"),
        &18u32,
        &None,
    );

    let mint_amount = bytes_to_i128(data, 0);
    let transfer_amount = bytes_to_i128(data, 8);
    let burn_amount = bytes_to_i128(data, 16);

    let _ = client.try_mint(&holder, &mint_amount);
    let _ = client.try_transfer(&holder, &recipient, &transfer_amount);
    let _ = client.try_burn(&recipient, &burn_amount);
});
