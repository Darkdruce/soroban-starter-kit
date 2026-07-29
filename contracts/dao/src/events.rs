use soroban_sdk::{Address, Env, Symbol};

pub fn initialized(env: &Env, admin: &Address, token: &Address, quorum: i128) {
    env.events().publish(
        (
            Symbol::new(env, "initialized"),
            admin.clone(),
            token.clone(),
        ),
        quorum,
    );
}

pub fn proposal_created(env: &Env, proposer: &Address, proposal_id: u32) {
    env.events()
        .publish((Symbol::new(env, "created"), proposer.clone()), proposal_id);
}

pub fn voted(env: &Env, voter: &Address, proposal_id: u32, support: bool, weight: i128) {
    env.events().publish(
        (Symbol::new(env, "voted"), voter.clone()),
        (proposal_id, support, weight),
    );
}

pub fn proposal_executed(env: &Env, proposal_id: u32) {
    env.events()
        .publish((Symbol::new(env, "executed"),), proposal_id);
}

pub fn proposal_cancelled(env: &Env, admin: &Address, proposal_id: u32) {
    env.events()
        .publish((Symbol::new(env, "cancelled"), admin.clone()), proposal_id);
}

/// Emitted when the original proposer self-cancels before any votes are cast.
pub fn proposal_proposer_cancelled(env: &Env, proposer: &Address, proposal_id: u32) {
    env.events().publish(
        (Symbol::new(env, "prop_cancelled"), proposer.clone()),
        proposal_id,
    );
}
