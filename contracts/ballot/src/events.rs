use soroban_sdk::{Address, Env, Symbol, Vec};

pub fn initialized(env: &Env, admin: &Address) {
    let topics = (Symbol::new(env, "initialized"),);
    env.events().publish(topics, admin.clone());
}

pub fn voter_registered(env: &Env, voter: &Address) {
    let topics = (Symbol::new(env, "voter_registered"),);
    env.events().publish(topics, voter.clone());
}

pub fn voter_deregistered(env: &Env, voter: &Address) {
    let topics = (Symbol::new(env, "voter_deregistered"),);
    env.events().publish(topics, voter.clone());
}

pub fn voted(env: &Env, voter: &Address, choice: u32) {
    let topics = (Symbol::new(env, "voted"),);
    env.events().publish(topics, (voter.clone(), choice));
}

/// Emitted by `tally` (binary ballot, backward compat).
pub fn tally_result(env: &Env, yes: i128, no: i128) {
    let topics = (Symbol::new(env, "tally_result"),);
    env.events().publish(topics, (yes, no));
}

/// Emitted by `tally_all` (multi-choice ballot, #788).
/// `counts` contains per-choice vote tallies in declaration order.
pub fn tally_all_result(env: &Env, counts: &Vec<i128>) {
    let topics = (Symbol::new(env, "tally_all_result"),);
    env.events().publish(topics, counts.clone());
}
