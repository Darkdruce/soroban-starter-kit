use soroban_sdk::{Address, Env, symbol_short};

pub fn listed(env: &Env, listing_id: u64, seller: &Address, price: i128) {
    env.events().publish(
        (symbol_short!("listed"), listing_id),
        (seller.clone(), price),
    );
}

pub fn sold(env: &Env, listing_id: u64, buyer: &Address, price: i128) {
    env.events()
        .publish((symbol_short!("sold"), listing_id), (buyer.clone(), price));
}

pub fn cancelled(env: &Env, listing_id: u64, seller: &Address) {
    env.events()
        .publish((symbol_short!("cancel"), listing_id), seller.clone());
}

pub fn swept(env: &Env, listing_id: u64, seller: &Address) {
    env.events()
        .publish((symbol_short!("swept"), listing_id), seller.clone());
}

pub fn offer_made(env: &Env, listing_id: u64, buyer: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("offered"), listing_id),
        (buyer.clone(), amount),
    );
}

pub fn offer_accepted(env: &Env, listing_id: u64, buyer: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("offracc"), listing_id),
        (buyer.clone(), amount),
    );
}

pub fn offer_cancelled(env: &Env, listing_id: u64, buyer: &Address) {
    env.events()
        .publish((symbol_short!("offrcncl"), listing_id), buyer.clone());
}
