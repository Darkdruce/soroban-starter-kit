use crate::errors::EscrowError;
use crate::storage::DataKey;
use soroban_sdk::{Address, Env, token};

pub fn require_admin(env: &Env) -> Result<Address, EscrowError> {
    soroban_common::try_get_admin(env).ok_or(EscrowError::NotInitialized)
}

pub fn transfer_token(env: &Env, from: &Address, to: &Address, amount: i128) {
    let token_contract: Address = soroban_common::get_instance(env, &DataKey::TokenContract);
    token::Client::new(env, &token_contract).transfer(from, to, &amount);
}
