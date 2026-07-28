use soroban_sdk::{Address, Env, Symbol};

/// Emitted when the contract is initialized.
/// Topics: (Symbol, Address) — event name, provider
pub fn initialized(env: &Env, provider: &Address, token: &Address) {
    env.events().publish(
        (Symbol::new(env, "initialized"), provider.clone()),
        token.clone(),
    );
}

/// Emitted when a new plan is registered by the provider.
/// Topics: (Symbol, Symbol) — event name, plan_id
pub fn plan_registered(env: &Env, plan_id: &Symbol, amount: i128, interval_ledgers: u32) {
    env.events().publish(
        (Symbol::new(env, "plan_registered"), plan_id.clone()),
        (amount, interval_ledgers),
    );
}

/// Emitted when a plan's status is updated by the provider.
/// Topics: (Symbol, Symbol) — event name, plan_id
pub fn plan_updated(env: &Env, plan_id: &Symbol, active: bool) {
    env.events().publish(
        (Symbol::new(env, "plan_updated"), plan_id.clone()),
        active,
    );
}

/// Emitted when a subscriber registers a new subscription.
/// Topics: (Symbol, Address) — event name, subscriber
pub fn subscribed(env: &Env, subscriber: &Address, plan_id: &Symbol, amount: i128, interval_ledgers: u32) {
    env.events().publish(
        (Symbol::new(env, "subscribed"), subscriber.clone()),
        (plan_id.clone(), amount, interval_ledgers),
    );
}

/// Emitted when the provider successfully charges a subscriber.
/// Topics: (Symbol, Address, Address) — event name, subscriber, provider
pub fn charged(env: &Env, subscriber: &Address, provider: &Address, amount: i128) {
    env.events().publish(
        (
            Symbol::new(env, "charged"),
            subscriber.clone(),
            provider.clone(),
        ),
        amount,
    );
}

/// Emitted when a subscriber cancels their subscription.
/// Topics: (Symbol, Address) — event name, subscriber
pub fn cancelled(env: &Env, subscriber: &Address) {
    env.events()
        .publish((Symbol::new(env, "cancelled"), subscriber.clone()), ());
}

/// Emitted when a subscriber's trial period is completed.
/// Topics: (Symbol, Address) — event name, subscriber
pub fn trial_completed(env: &Env, subscriber: &Address) {
    env.events()
        .publish((Symbol::new(env, "trial_completed"), subscriber.clone()), ());
}