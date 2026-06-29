use soroban_sdk::{Address, Env, Symbol};

pub fn initialized(
    env: &Env,
    admin: &Address,
    name: &soroban_sdk::String,
    symbol: &soroban_sdk::String,
) {
    env.events().publish(
        (Symbol::new(env, "initialized"), admin.clone()),
        (name.clone(), symbol.clone()),
    );
}

pub fn minted(env: &Env, to: &Address, token_id: u32) {
    env.events()
        .publish((Symbol::new(env, "minted"), to.clone()), token_id);
}

pub fn transferred(env: &Env, from: &Address, to: &Address, token_id: u32) {
    env.events().publish(
        (Symbol::new(env, "transferred"), from.clone(), to.clone()),
        token_id,
    );
}

pub fn burned(env: &Env, from: &Address, token_id: u32) {
    env.events()
        .publish((Symbol::new(env, "burned"), from.clone()), token_id);
}

pub fn approved(env: &Env, owner: &Address, spender: &Address, token_id: u32) {
    env.events().publish(
        (Symbol::new(env, "approved"), owner.clone(), spender.clone()),
        token_id,
    );
}
