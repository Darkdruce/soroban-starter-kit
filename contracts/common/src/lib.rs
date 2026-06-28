#![no_std]

use soroban_sdk::{contracttype, Address, Env};

/// Minimum number of ledgers the deadline must be ahead of the current ledger
/// when initializing an escrow. Enforced by the contract; tests must respect
/// this value to avoid generating deadlines the contract would reject.
pub const MIN_DEADLINE_BUFFER: u32 = 10;

/// Storage key for the contract administrator address.
///
/// Used in instance storage to persist the admin [`Address`] across invocations.
///
/// # Examples
///
/// ```ignore
/// use soroban_sdk::{Env, Address};
/// use soroban_common::AdminKey;
///
/// let env = Env::default();
/// let admin_address = Address::generate(&env);
/// env.storage().instance().set(&AdminKey::Admin, &admin_address);
/// ```
#[contracttype]
#[derive(Clone)]
pub enum AdminKey {
    Admin,
}

/// Reads `AdminKey::Admin` from instance storage, panicking if unset.
///
/// # Panics
///
/// Panics with `"admin not set"` if the admin has not been stored yet.
///
/// # Examples
///
/// ```ignore
/// use soroban_sdk::{Env, Address};
/// use soroban_common::{AdminKey, get_admin};
///
/// let env = Env::default();
/// let admin_address = Address::generate(&env);
/// env.storage().instance().set(&AdminKey::Admin, &admin_address);
///
/// let admin: Address = get_admin(&env);
/// assert_eq!(admin, admin_address);
/// ```
#[must_use]
pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&AdminKey::Admin)
        .expect("admin not set")
}

/// Reads `AdminKey::Admin` from instance storage, returning `None` if unset.
///
/// # Examples
///
/// ```ignore
/// use soroban_sdk::{Env, Address};
/// use soroban_common::{AdminKey, try_get_admin};
///
/// let env = Env::default();
///
/// // Before setting admin
/// assert_eq!(try_get_admin(&env), None);
///
/// // After setting admin
/// let admin_address = Address::generate(&env);
/// env.storage().instance().set(&AdminKey::Admin, &admin_address);
/// assert_eq!(try_get_admin(&env), Some(admin_address));
/// ```
#[must_use]
pub fn try_get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&AdminKey::Admin)
}

/// Reads a value from instance storage by key, panicking if missing.
///
/// # Panics
///
/// Panics with `"key not found"` if the key does not exist in instance storage.
///
/// # Examples
///
/// ```ignore
/// use soroban_sdk::{contracttype, Env};
/// use soroban_common::get_instance;
///
/// #[contracttype]
/// #[derive(Clone)]
/// enum DataKey {
///     Amount,
/// }
///
/// let env = Env::default();
/// let amount: i128 = 1000;
/// env.storage().instance().set(&DataKey::Amount, &amount);
///
/// let retrieved: i128 = get_instance(&env, &DataKey::Amount);
/// assert_eq!(retrieved, 1000);
/// ```
pub fn get_instance<K, V>(env: &Env, key: &K) -> V
where
    K: soroban_sdk::TryIntoVal<Env, soroban_sdk::Val>
        + soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
    V: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>,
{
    env.storage().instance().get(key).expect("key not found")
}

/// Extends the TTL of instance storage by `extend_to` ledgers if the current
/// TTL is below `threshold`.
///
/// # Examples
///
/// ```ignore
/// use soroban_sdk::Env;
/// use soroban_common::extend_ttl_instance;
///
/// let env = Env::default();
/// // Keep instance storage alive for ~30 days if TTL drops below ~7 days.
/// extend_ttl_instance(&env, 120_960, 518_400);
/// ```
pub fn extend_ttl_instance(env: &Env, threshold: u32, extend_to: u32) {
    env.storage()
        .instance()
        .extend_ttl(threshold, extend_to);
}

/// Extends the TTL of a persistent storage entry if the current TTL is below
/// `threshold`.
///
/// # Examples
///
/// ```ignore
/// use soroban_sdk::{contracttype, Env, Address};
/// use soroban_common::extend_ttl_persistent;
///
/// #[contracttype]
/// #[derive(Clone)]
/// enum DataKey {
///     Balance(Address),
/// }
///
/// let env = Env::default();
/// let addr = Address::generate(&env);
/// extend_ttl_persistent(&env, &DataKey::Balance(addr), 120_960, 518_400);
/// ```
pub fn extend_ttl_persistent<K>(env: &Env, key: &K, threshold: u32, extend_to: u32)
where
    K: soroban_sdk::TryIntoVal<Env, soroban_sdk::Val>
        + soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, threshold, extend_to);
}

/// Expected wall-clock seconds between consecutive Soroban ledgers.
/// Used to convert ledger counts to approximate durations in doc comments.
pub const LEDGER_SECONDS: u32 = 5;

/// Ledger threshold for TTL extension (~14 days at `LEDGER_SECONDS` seconds per ledger).
/// When remaining TTL falls below this, storage is extended to `LEDGER_BUMP_AMOUNT`.
pub const LEDGER_LIFETIME_THRESHOLD: u32 = 120_960;

/// Target TTL (in ledgers) after each extension (~60 days at `LEDGER_SECONDS` seconds per ledger).
pub const LEDGER_BUMP_AMOUNT: u32 = 518_400;

/// Validates that a deadline is sufficiently far in the future.
///
/// Returns `Ok(())` if `deadline >= current_ledger + MIN_DEADLINE_BUFFER`.
/// Returns an error otherwise.
///
/// # Examples
///
/// ```ignore
/// use soroban_common::validate_deadline;
/// use soroban_sdk::Env;
///
/// let env = Env::default();
/// let deadline = env.ledger().sequence() + 10;
/// validate_deadline(&env, deadline)?; // Ok if MIN_DEADLINE_BUFFER <= 10
/// ```
pub fn validate_deadline<E>(env: &Env, deadline: u32) -> Result<(), E>
where
    E: From<()>,
{
    if deadline < env.ledger().sequence() + MIN_DEADLINE_BUFFER {
        Err(E::from(()))
    } else {
        Ok(())
    }
}
