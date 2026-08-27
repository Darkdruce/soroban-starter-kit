use soroban_sdk::{Address, Env, Symbol};

pub fn initialized(env: &Env, admin: &Address, token: &Address) {
    env.events()
        .publish((Symbol::new(env, "initialized"),), (admin.clone(), token.clone()));
}

pub fn bought(env: &Env, buyer: &Address, tokens: i128, cost: i128) {
    env.events()
        .publish((Symbol::new(env, "bought"),), (buyer.clone(), tokens, cost));
}

pub fn sold(env: &Env, seller: &Address, tokens: i128, proceeds: i128) {
    env.events()
        .publish((Symbol::new(env, "sold"),), (seller.clone(), tokens, proceeds));
}
