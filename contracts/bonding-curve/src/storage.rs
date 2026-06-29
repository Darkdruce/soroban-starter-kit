// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::Address;

#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    Token,
    Reserve,
    Supply,
    Price,
}

/// Price scale factor for fixed-point arithmetic
pub const PRICE_SCALE: i128 = 1_000_000;
