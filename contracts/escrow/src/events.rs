use soroban_sdk::{Address, Env, Symbol};

/// Emitted when the escrow is initialized.
/// Topics: (Symbol, Address, Address, Address) — event name, buyer, seller, arbiter
pub fn initialized(env: &Env, buyer: &Address, seller: &Address, arbiter: &Address, amount: i128) {
    env.events().publish(
        (Symbol::new(env, "initialized"), buyer.clone(), seller.clone(), arbiter.clone()),
        amount,
    );
}

/// Emitted when an escrow is created.
/// Topics: (Symbol, Address, Address) — event name, buyer, seller
pub fn escrow_created(env: &Env, buyer: &Address, seller: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "created"), buyer.clone(), seller.clone()), amount);
}

/// Emitted when an escrow is funded.
/// Topics: (Symbol, Address) — event name, buyer
pub fn escrow_funded(env: &Env, buyer: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "funded"), buyer.clone()), amount);
}

/// Emitted when delivery is marked.
/// Topics: (Symbol, Address) — event name, seller
pub fn delivery_marked(env: &Env, seller: &Address) {
    env.events().publish((Symbol::new(env, "marked_delivered"), seller.clone()), ());
}

/// Emitted when funds are released to the seller.
/// Topics: (Symbol, Address) — event name, seller
pub fn funds_released(env: &Env, seller: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "released"), seller.clone()), amount);
}

/// Emitted when funds are refunded to the buyer.
/// Topics: (Symbol, Address) — event name, buyer
pub fn partial_release(env: &Env, seller: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "released_partial"), seller.clone()), amount);
}

pub fn funds_refunded(env: &Env, buyer: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "refunded"), buyer.clone()), amount);
}

/// Emitted when the escrow amount is updated.
/// Topics: (Symbol, Address) — event name, buyer
pub fn amount_updated(env: &Env, buyer: &Address, new_amount: i128) {
    env.events().publish((Symbol::new(env, "amount_updated"), buyer.clone()), new_amount);
}

/// Emitted when the escrow is cancelled.
/// Topics: (Symbol, Address) — event name, buyer
pub fn escrow_cancelled(env: &Env, buyer: &Address) {
    env.events().publish((Symbol::new(env, "escrow_cancelled"), buyer.clone()), ());
}

/// Emitted when the deadline is extended.
/// Topics: (Symbol, Address) — event name, buyer
pub fn deadline_extended(env: &Env, buyer: &Address, new_deadline: u32) {
    env.events().publish((Symbol::new(env, "deadline_extended"), buyer.clone()), new_deadline);
}

/// Emitted when a dispute is raised.
/// Topics: (Symbol, Address) — event name, caller
pub fn dispute_raised(env: &Env, caller: &Address) {
    env.events().publish((Symbol::new(env, "dispute_raised"), caller.clone()), ());
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
