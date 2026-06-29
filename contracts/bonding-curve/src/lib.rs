#![no_std]
#![deny(missing_docs)]
//! Bonding-curve token sale contract template.
//!
//! Token price scales deterministically with supply along a bonding curve;
//! buyers mint tokens by paying the curve price and sellers burn to redeem.

use soroban_sdk::{contract, contractimpl, token, Address, Env};

mod errors;
mod events;
mod storage;

#[cfg(test)]
mod test;

pub use errors::BondingCurveError;
pub use storage::{DataKey, PRICE_SCALE};

use soroban_common::{extend_ttl_instance, LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump(env: &Env) {
    extend_ttl_instance(env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Linear bonding curve: price = reserve / (supply + 1)
///
/// Buy increases supply and price. Sell decreases both.
/// Reserve is held in the contract.
fn calculate_price(reserve: i128, supply: i128) -> Result<i128, BondingCurveError> {
    if supply + 1 == 0 {
        return Err(BondingCurveError::Overflow);
    }
    Ok(reserve * PRICE_SCALE / (supply + 1))
}

/// Compute cost to buy `amount` tokens: integral from supply to supply+amount of price dx
fn buy_cost(reserve: i128, supply: i128, amount: i128) -> Result<i128, BondingCurveError> {
    if amount <= 0 {
        return Err(BondingCurveError::InvalidAmount);
    }

    let old_supply = supply;
    let new_supply = supply.checked_add(amount).ok_or(BondingCurveError::Overflow)?;

    // Linear curve: cost ≈ reserve * (1/(old_supply+1) + ... + 1/(new_supply+1))
    // Simplified: reserve * amount / (supply + 1) + reserve * amount^2 / (2 * (supply+1)^2)
    // For minimal gas, use average price approximation:
    let avg_price = (calculate_price(reserve, old_supply)? + calculate_price(reserve, new_supply)?) / 2;
    let cost = amount * avg_price / PRICE_SCALE;
    Ok(cost)
}

/// Compute proceeds from selling `amount` tokens
fn sell_proceeds(reserve: i128, supply: i128, amount: i128) -> Result<i128, BondingCurveError> {
    if amount <= 0 || amount > supply {
        return Err(BondingCurveError::InvalidAmount);
    }

    let old_supply = supply;
    let new_supply = supply - amount;

    let avg_price = (calculate_price(reserve, old_supply)? + calculate_price(reserve, new_supply)?) / 2;
    let proceeds = amount * avg_price / PRICE_SCALE;
    Ok(proceeds)
}

/// Bonding curve token contract.
///
/// Linear curve: price increases with supply.
/// Buy adds to supply and consumes reserve.
/// Sell removes from supply and returns reserve.
pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module and re-export
// the public contract API above, keeping the rest of the crate enforced.
mod contract {
    #![allow(missing_docs)]
    use super::*;

    #[contract]
    pub struct BondingCurveContract;

#[contractimpl]
impl BondingCurveContract {
    /// Initialize the bonding curve contract.
    ///
    /// # Errors
    /// - [`BondingCurveError::AlreadyInitialized`] if called more than once.
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
    ) -> Result<(), BondingCurveError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(BondingCurveError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Reserve, &0i128);
        env.storage().instance().set(&DataKey::Supply, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::Price, &calculate_price(0, 0)?);

        bump(&env);
        events::initialized(&env, &admin, &token);
        Ok(())
    }

    /// Buy `amount` tokens by paying from the reserve.
    ///
    /// # Errors
    /// - [`BondingCurveError::NotInitialized`] if the contract has not been initialized.
    /// - [`BondingCurveError::InvalidAmount`] if `amount` <= 0.
    pub fn buy(env: Env, buyer: Address, amount: i128, max_cost: i128) -> Result<(), BondingCurveError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(BondingCurveError::NotInitialized);
        }
        if amount <= 0 {
            return Err(BondingCurveError::InvalidAmount);
        }
        buyer.require_auth();

        let token: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(BondingCurveError::NotInitialized)?;

        let reserve: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Reserve)
            .unwrap_or(0i128);
        let supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Supply)
            .unwrap_or(0i128);

        let cost = buy_cost(reserve, supply, amount)?;
        if cost > max_cost {
            return Err(BondingCurveError::InvalidAmount);
        }

        token::Client::new(&env, &token).transfer(&buyer, &env.current_contract_address(), &cost);

        let new_supply = supply + amount;
        let new_reserve = reserve + cost;
        let new_price = calculate_price(new_reserve, new_supply)?;

        env.storage().instance().set(&DataKey::Supply, &new_supply);
        env.storage().instance().set(&DataKey::Reserve, &new_reserve);
        env.storage().instance().set(&DataKey::Price, &new_price);

        bump(&env);
        events::bought(&env, &buyer, amount, cost);
        Ok(())
    }

    /// Sell `amount` tokens to withdraw from the reserve.
    ///
    /// # Errors
    /// - [`BondingCurveError::NotInitialized`] if the contract has not been initialized.
    /// - [`BondingCurveError::InvalidAmount`] if `amount` <= 0 or exceeds supply.
    /// - [`BondingCurveError::InsufficientReserve`] if the reserve is insufficient.
    pub fn sell(env: Env, seller: Address, amount: i128, min_proceeds: i128) -> Result<(), BondingCurveError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(BondingCurveError::NotInitialized);
        }
        if amount <= 0 {
            return Err(BondingCurveError::InvalidAmount);
        }
        seller.require_auth();

        let token: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(BondingCurveError::NotInitialized)?;

        let reserve: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Reserve)
            .unwrap_or(0i128);
        let supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Supply)
            .unwrap_or(0i128);

        if amount > supply {
            return Err(BondingCurveError::InvalidAmount);
        }

        let proceeds = sell_proceeds(reserve, supply, amount)?;
        if proceeds < min_proceeds {
            return Err(BondingCurveError::InvalidAmount);
        }
        if proceeds > reserve {
            return Err(BondingCurveError::InsufficientReserve);
        }

        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &seller,
            &proceeds,
        );

        let new_supply = supply - amount;
        let new_reserve = reserve - proceeds;
        let new_price = calculate_price(new_reserve, new_supply)?;

        env.storage().instance().set(&DataKey::Supply, &new_supply);
        env.storage().instance().set(&DataKey::Reserve, &new_reserve);
        env.storage().instance().set(&DataKey::Price, &new_price);

        bump(&env);
        events::sold(&env, &seller, amount, proceeds);
        Ok(())
    }

    /// Get current reserve.
    pub fn get_reserve(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Reserve)
            .unwrap_or(0i128)
    }

    /// Get current supply.
    pub fn get_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Supply)
            .unwrap_or(0i128)
    }

    /// Get current price per token.
    pub fn get_price(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Price)
            .unwrap_or(0i128)
    }
}
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[test]
    fn test_price_increases_with_supply() {
        let env = Env::default();
        env.ledger().with_mut(|le| {
            le.timestamp = 1;
        });

        let admin = Address::random(&env);
        let buyer = Address::random(&env);
        let token = Address::random(&env);

        let contract = BondingCurveContractClient::new(&env, &env.current_contract_id());

        contract.initialize(&admin, &token);

        let initial_price = contract.get_price();
        assert!(initial_price >= 0);

        // Buy first batch - should be cheap
        contract.buy(&buyer, &100i128, &i128::MAX);
        let price_after_first = contract.get_price();

        // Buy second batch - should be more expensive
        contract.buy(&buyer, &100i128, &i128::MAX);
        let price_after_second = contract.get_price();

        assert!(price_after_first > initial_price);
        assert!(price_after_second > price_after_first);
    }

    #[test]
    fn test_buy_sell_1_to_1_reserve() {
        let env = Env::default();
        env.ledger().with_mut(|le| {
            le.timestamp = 1;
        });

        let admin = Address::random(&env);
        let trader = Address::random(&env);
        let token = Address::random(&env);

        let contract = BondingCurveContractClient::new(&env, &env.current_contract_id());

        contract.initialize(&admin, &token);

        let initial_reserve = contract.get_reserve();
        assert_eq!(initial_reserve, 0);

        // Buy tokens
        contract.buy(&trader, &100i128, &i128::MAX);
        let reserve_after_buy = contract.get_reserve();
        assert!(reserve_after_buy > 0);

        // Sell half back
        contract.sell(&trader, &50i128, &0i128);
        let reserve_after_sell = contract.get_reserve();

        // Reserve should have decreased but not to zero (slippage)
        assert!(reserve_after_sell > 0);
        assert!(reserve_after_sell < reserve_after_buy);
    }

    #[test]
    fn test_overflow_safety() {
        let env = Env::default();
        env.ledger().with_mut(|le| {
            le.timestamp = 1;
        });

        let admin = Address::random(&env);
        let buyer = Address::random(&env);
        let token = Address::random(&env);

        let contract = BondingCurveContractClient::new(&env, &env.current_contract_id());

        contract.initialize(&admin, &token);

        // Try to buy with invalid amount
        let result = contract.try_buy(&buyer, &-100i128, &i128::MAX);
        assert!(result.is_err());
    }
}
