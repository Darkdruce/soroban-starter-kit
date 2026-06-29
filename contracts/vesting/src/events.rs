use soroban_sdk::{Address, Env, Symbol};

pub fn initialized(
    env: &Env,
    beneficiary: &Address,
    amount: i128,
    cliff_ledger: u32,
    end_ledger: u32,
) {
    env.events().publish(
        (Symbol::new(env, "initialized"), beneficiary.clone()),
        (amount, cliff_ledger, end_ledger),
    );
}

pub fn claimed(env: &Env, beneficiary: &Address, amount: i128) {
    env.events()
        .publish((Symbol::new(env, "claimed"), beneficiary.clone()), amount);
}

pub fn revoked(env: &Env, admin: &Address, returned: i128) {
    env.events()
        .publish((Symbol::new(env, "revoked"), admin.clone()), returned);
}
