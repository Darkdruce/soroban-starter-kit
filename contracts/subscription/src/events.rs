use soroban_sdk::{Address, Env, Symbol};

/// Emitted when the contract is initialized.
/// Topics: (Symbol, Address) — event name, provider
pub fn initialized(env: &Env, provider: &Address, token: &Address) {
    env.events().publish(
        (Symbol::new(env, "initialized"), provider.clone()),
        token.clone(),
    );
}

/// Emitted when a subscriber registers a new subscription.
/// Topics: (Symbol, Address) — event name, subscriber
pub fn subscribed(env: &Env, subscriber: &Address, amount: i128, interval_ledgers: u32) {
    env.events().publish(
        (Symbol::new(env, "subscribed"), subscriber.clone()),
        (amount, interval_ledgers),
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
