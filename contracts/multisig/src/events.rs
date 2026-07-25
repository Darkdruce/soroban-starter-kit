use soroban_sdk::{Address, Env, Symbol};

pub fn initialized(env: &Env, threshold: u32, signer_count: u32) {
    env.events()
        .publish((Symbol::new(env, "initialized"), threshold), signer_count);
}

pub fn signer_added(env: &Env, signer: &Address, threshold: u32) {
    env.events()
        .publish((Symbol::new(env, "added"), signer.clone()), threshold);
}

pub fn signer_removed(env: &Env, signer: &Address, threshold: u32) {
    env.events()
        .publish((Symbol::new(env, "removed"), signer.clone()), threshold);
}

pub fn transaction_proposed(env: &Env, tx_id: u64, proposer: &Address) {
    env.events()
        .publish((Symbol::new(env, "proposed"), proposer.clone()), tx_id);
}

pub fn transaction_signed(env: &Env, tx_id: u64, signer: &Address, signature_count: u32) {
    env.events().publish(
        (Symbol::new(env, "signed"), signer.clone(), tx_id),
        signature_count,
    );
}

pub fn transaction_executed(env: &Env, tx_id: u64) {
    env.events()
        .publish((Symbol::new(env, "executed"), tx_id), ());
}

pub fn proposal_expired(env: &Env, tx_id: u64) {
    env.events()
        .publish((Symbol::new(env, "expired"), tx_id), ());
}
