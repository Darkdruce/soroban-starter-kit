#![no_std]
#![deny(missing_docs)]
//! Sealed-deadline auction contract template.
//!
//! A seller lists an item; bidders submit increasing bids until the deadline,
//! outbid bidders reclaim their funds, and anyone can settle the auction once
//! the deadline passes.

use soroban_sdk::{Address, Env, contract, contractimpl, token};

mod errors;
mod events;
mod storage;

pub use errors::AuctionError;
pub use storage::{AuctionInfo, DataKey};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn get_instance<T: soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>(
    env: &Env,
    key: &DataKey,
) -> Result<T, AuctionError> {
    env.storage()
        .instance()
        .get(key)
        .ok_or(AuctionError::NotInitialized)
}

/// English auction contract.
///
/// Lifecycle:
/// - Seller calls `start` to set the token, starting price, minimum bid increment, and deadline.
/// - Bidders call `bid` with increasing amounts. The previous highest bidder's funds are held
///   as a pending refund, collectable via `withdraw`.
/// - After the deadline, anyone calls `end` to settle. On success the seller receives the
///   winning bid; if no bids were placed the auction ends with no transfer.
/// - Outbid bidders call `withdraw` to recover their pending refund at any time.
pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module and re-export
// the public contract API above, keeping the rest of the crate enforced.
mod contract {
    #![allow(missing_docs)]
    use super::*;

    #[contract]
    pub struct AuctionContract;

    #[contractimpl]
    impl AuctionContract {
        /// Start the auction.
        ///
        /// # Errors
        ///
        /// - [`AuctionError::AlreadyInitialized`] if already started.
        /// - [`AuctionError::InvalidAmount`] if `start_price` or `min_increment` <= 0.
        /// - [`AuctionError::InvalidDeadline`] if `deadline` <= current ledger.
        pub fn start(
            env: Env,
            seller: Address,
            token: Address,
            start_price: i128,
            min_increment: i128,
            deadline: u32,
        ) -> Result<(), AuctionError> {
            if env.storage().instance().has(&DataKey::Seller) {
                return Err(AuctionError::AlreadyInitialized);
            }
            if start_price <= 0 || min_increment <= 0 {
                return Err(AuctionError::InvalidAmount);
            }
            if deadline <= env.ledger().sequence() {
                return Err(AuctionError::InvalidDeadline);
            }

            seller.require_auth();

            env.storage().instance().set(&DataKey::Seller, &seller);
            env.storage().instance().set(&DataKey::Token, &token);
            env.storage()
                .instance()
                .set(&DataKey::StartPrice, &start_price);
            env.storage()
                .instance()
                .set(&DataKey::MinIncrement, &min_increment);
            env.storage().instance().set(&DataKey::Deadline, &deadline);
            // highest_bid starts at start_price - 1 so the first bid must be >= start_price
            env.storage()
                .instance()
                .set(&DataKey::HighestBid, &(start_price - 1));
            env.storage().instance().set(&DataKey::Settled, &false);

            bump_instance(&env);
            events::started(&env, &seller, start_price, deadline);
            Ok(())
        }

        /// Place a bid. The bid must be at least `highest_bid + min_increment`.
        /// The previous highest bidder's funds are queued as a pending refund.
        ///
        /// # Errors
        ///
        /// - [`AuctionError::NotInitialized`] if not started.
        /// - [`AuctionError::AuctionEnded`] if the deadline has passed.
        /// - [`AuctionError::BidTooLow`] if `amount` < current highest bid + min_increment.
        pub fn bid(env: Env, bidder: Address, amount: i128) -> Result<(), AuctionError> {
            if amount <= 0 {
                return Err(AuctionError::InvalidAmount);
            }

            let deadline: u32 = get_instance(&env, &DataKey::Deadline)?;
            if env.ledger().sequence() > deadline {
                return Err(AuctionError::AuctionEnded);
            }

            let highest_bid: i128 = get_instance(&env, &DataKey::HighestBid)?;
            let min_increment: i128 = get_instance(&env, &DataKey::MinIncrement)?;
            let start_price: i128 = get_instance(&env, &DataKey::StartPrice)?;

            // First bid must be >= start_price; subsequent bids must be >= highest_bid + min_increment
            let min_required = if highest_bid < start_price {
                start_price
            } else {
                highest_bid + min_increment
            };

            if amount < min_required {
                return Err(AuctionError::BidTooLow);
            }

            bidder.require_auth();

            let token: Address = get_instance(&env, &DataKey::Token)?;
            token::Client::new(&env, &token).transfer(
                &bidder,
                &env.current_contract_address(),
                &amount,
            );

            // Queue previous highest bidder's refund
            let prev_bidder: Option<Address> =
                env.storage().instance().get(&DataKey::HighestBidder);
            if let Some(prev) = prev_bidder {
                let pending: i128 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Pending(prev.clone()))
                    .unwrap_or(0);
                let new_pending = pending + highest_bid;
                env.storage()
                    .persistent()
                    .set(&DataKey::Pending(prev.clone()), &new_pending);
                env.storage().persistent().extend_ttl(
                    &DataKey::Pending(prev),
                    LEDGER_LIFETIME_THRESHOLD,
                    LEDGER_BUMP_AMOUNT,
                );
            }

            env.storage()
                .instance()
                .set(&DataKey::HighestBidder, &bidder);
            env.storage().instance().set(&DataKey::HighestBid, &amount);

            bump_instance(&env);
            events::bid_placed(&env, &bidder, amount);
            Ok(())
        }

        /// Settle the auction after the deadline. Transfers the winning bid to the seller.
        /// If no bids were placed, emits `ended_no_bids` and nothing is transferred.
        ///
        /// # Errors
        ///
        /// - [`AuctionError::NotInitialized`] if not started.
        /// - [`AuctionError::AuctionNotEnded`] if the deadline has not passed.
        /// - [`AuctionError::AlreadyEnded`] if already settled.
        pub fn end(env: Env) -> Result<(), AuctionError> {
            get_instance::<Address>(&env, &DataKey::Seller)?; // ensure initialized

            let deadline: u32 = get_instance(&env, &DataKey::Deadline)?;
            if env.ledger().sequence() <= deadline {
                return Err(AuctionError::AuctionNotEnded);
            }

            let settled: bool = get_instance(&env, &DataKey::Settled)?;
            if settled {
                return Err(AuctionError::AlreadyEnded);
            }

            env.storage().instance().set(&DataKey::Settled, &true);

            let start_price: i128 = get_instance(&env, &DataKey::StartPrice)?;
            let highest_bid: i128 = get_instance(&env, &DataKey::HighestBid)?;
            let winner: Option<Address> = env.storage().instance().get(&DataKey::HighestBidder);

            bump_instance(&env);

            if highest_bid < start_price || winner.is_none() {
                events::ended_no_bids(&env);
                return Ok(());
            }

            let seller: Address = get_instance(&env, &DataKey::Seller)?;
            let token: Address = get_instance(&env, &DataKey::Token)?;
            let winner = winner.unwrap(); // safe: checked above

            token::Client::new(&env, &token).transfer(
                &env.current_contract_address(),
                &seller,
                &highest_bid,
            );

            events::ended(&env, &winner, highest_bid);
            Ok(())
        }

        /// Withdraw a pending refund (available for outbid bidders).
        ///
        /// # Errors
        ///
        /// - [`AuctionError::NothingToWithdraw`] if caller has no pending refund.
        pub fn withdraw(env: Env, bidder: Address) -> Result<(), AuctionError> {
            bidder.require_auth();

            let pending: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Pending(bidder.clone()))
                .unwrap_or(0);
            if pending <= 0 {
                return Err(AuctionError::NothingToWithdraw);
            }

            env.storage()
                .persistent()
                .remove(&DataKey::Pending(bidder.clone()));

            let token: Address = get_instance(&env, &DataKey::Token)?;
            token::Client::new(&env, &token).transfer(
                &env.current_contract_address(),
                &bidder,
                &pending,
            );

            events::withdrawn(&env, &bidder, pending);
            Ok(())
        }

        /// Return auction details.
        #[must_use]
        pub fn get_info(env: Env) -> Result<AuctionInfo, AuctionError> {
            Ok(AuctionInfo {
                seller: get_instance(&env, &DataKey::Seller)?,
                token: get_instance(&env, &DataKey::Token)?,
                start_price: get_instance(&env, &DataKey::StartPrice)?,
                min_increment: get_instance(&env, &DataKey::MinIncrement)?,
                deadline: get_instance(&env, &DataKey::Deadline)?,
                highest_bid: get_instance(&env, &DataKey::HighestBid)?,
                highest_bidder: env.storage().instance().get(&DataKey::HighestBidder),
                settled: get_instance(&env, &DataKey::Settled)?,
            })
        }

        /// Return a bidder's pending refund amount.
        #[must_use]
        pub fn get_pending(env: Env, bidder: Address) -> i128 {
            env.storage()
                .persistent()
                .get(&DataKey::Pending(bidder))
                .unwrap_or(0)
        }
    }
}

mod test;
