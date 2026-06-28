use soroban_sdk::{Address, Env, String, Symbol};

/// Emitted when the token is initialized.
/// Topics: (Symbol, Address) — event name, admin
pub fn initialized(env: &Env, admin: &Address, name: String, symbol: String, decimals: u32) {
    env.events().publish((Symbol::new(env, "initialized"), admin.clone()), (name, symbol, decimals));
}

/// Emitted when tokens are minted.
/// Topics: (Symbol, Address) — event name, recipient
pub fn minted(env: &Env, to: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "mint"), to.clone()), amount);
}

/// Emitted when tokens are burned.
/// Topics: (Symbol, Address) — event name, account
pub fn burned(env: &Env, from: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "burn"), from.clone()), amount);
}

/// Emitted when the admin is changed.
/// Topics: (Symbol, Address) — event name, old admin
pub fn admin_changed(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish((Symbol::new(env, "admin_changed"), old_admin.clone()), new_admin.clone());
}

/// Emitted when an allowance is approved.
/// Topics: (Symbol, Address, Address) — event name, owner, spender
pub fn admin_proposed(env: &Env, current_admin: &Address, pending_admin: &Address) {
    env.events().publish((Symbol::new(env, "admin_proposed"), current_admin.clone()), pending_admin.clone());
}

pub fn admin_accepted(env: &Env, new_admin: &Address) {
    env.events().publish((Symbol::new(env, "admin_accepted"), new_admin.clone()), ());
}

pub fn admin_proposal_cancelled(env: &Env, admin: &Address) {
    env.events().publish((Symbol::new(env, "admin_proposal_cancelled"), admin.clone()), ());
}

pub fn approved(env: &Env, from: &Address, spender: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "approve"), from.clone(), spender.clone()), amount);
}

/// Emitted when an allowance is revoked.
/// Topics: (Symbol, Address, Address) — event name, owner, spender
pub fn revoked(env: &Env, from: &Address, spender: &Address) {
    env.events().publish((Symbol::new(env, "revoke"), from.clone(), spender.clone()), ());
}

/// Emitted when tokens are transferred.
/// Topics: (Symbol, Address, Address) — event name, from, to
pub fn transferred(env: &Env, from: &Address, to: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "transfer"), from.clone(), to.clone()), amount);
}

/// Emitted when an account is frozen.
/// Topics: (Symbol, Address) — event name, account
pub fn account_frozen(env: &Env, account: &Address) {
    env.events().publish((Symbol::new(env, "account_frozen"), account.clone()), ());
}

/// Emitted when an account is unfrozen.
/// Topics: (Symbol, Address) — event name, account
pub fn account_unfrozen(env: &Env, account: &Address) {
    env.events().publish((Symbol::new(env, "account_unfrozen"), account.clone()), ());
}

/// Emitted when the contract is paused.
/// Topics: (Symbol, Address) — event name, admin
pub fn paused(env: &Env, admin: &Address) {
    env.events().publish((Symbol::new(env, "paused"), admin.clone()), ());
}

/// Emitted when the contract is unpaused.
/// Topics: (Symbol, Address) — event name, admin
pub fn unpaused(env: &Env, admin: &Address) {
    env.events().publish((Symbol::new(env, "unpaused"), admin.clone()), ());
}

/// Emitted when the contract is upgraded.
/// Topics: (Symbol, Address) — event name, admin
pub fn upgraded(env: &Env, admin: &Address, new_wasm_hash: &soroban_sdk::BytesN<32>) {
    env.events().publish((Symbol::new(env, "upgraded"), admin.clone()), new_wasm_hash.clone());
}
