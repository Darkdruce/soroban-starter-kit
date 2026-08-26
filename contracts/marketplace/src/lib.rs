#![no_std]
#![deny(missing_docs)]
//! NFT/asset marketplace contract template.
//!
//! Sellers list assets at a fixed price; buyers purchase them, transferring
//! payment to the seller and the asset to the buyer in one transaction.
//! Sellers may also set an optional expiry on a listing, and buyers may
//! propose a lower price via an escrowed offer that the seller can accept.

use soroban_sdk::{Address, Env, Vec, contract, contractclient, contractimpl, token};

mod errors;
mod events;
mod storage;

pub use errors::MarketplaceError;
pub use storage::{DataKey, Listing, ListingEntry, ListingPage};

/// Maximum number of listings a single [`MarketplaceContract::get_active_listings`] call
/// may return, regardless of the requested `limit`.
pub const MAX_LISTINGS_PAGE_SIZE: u32 = 50;

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD, apply_bps_fee};

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn bump_listing(env: &Env, id: u64) {
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::Listing(id), LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn bump_offer(env: &Env, id: u64, buyer: &Address) {
    env.storage().persistent().extend_ttl(
        &DataKey::Offer(id, buyer.clone()),
        LEDGER_LIFETIME_THRESHOLD,
        LEDGER_BUMP_AMOUNT,
    );
}

/// NFT/asset marketplace contract.
pub use contract::*;

mod contract {
    #![allow(missing_docs)]
    use super::*;

    use storage::DataKey::*;

    #[contract]
    pub struct MarketplaceContract;

    #[contractimpl]
    impl MarketplaceContract {
        /// Initialise the marketplace with a payment token, optional royalty BPS, and
        /// royalty recipient.
        ///
        /// `royalty_bps` is in basis points (0 = no royalty, 10 000 = 100 %).
        ///
        /// # Errors
        /// - [`MarketplaceError::AlreadyInitialized`]
        /// - [`MarketplaceError::InvalidRoyalty`] if `royalty_bps > 10_000`.
        /// - [`MarketplaceError::RoyaltyRecipientMissing`] if `royalty_bps > 0` but
        ///   `royalty_recipient` is `None`.
        pub fn initialize(
            env: Env,
            admin: Address,
            payment_token: Address,
            royalty_bps: u32,
            royalty_recipient: Address,
        ) -> Result<(), MarketplaceError> {
            if env.storage().instance().has(&State) {
                return Err(MarketplaceError::AlreadyInitialized);
            }
            if royalty_bps > 10_000 {
                return Err(MarketplaceError::InvalidRoyalty);
            }
            admin.require_auth();

            env.storage().instance().set(&Admin, &admin);
            env.storage().instance().set(&Token, &payment_token);
            env.storage()
                .instance()
                .set(&DataKey::NextListingId, &0u64);
            env.storage()
                .instance()
                .set(&RoyaltyBps, &royalty_bps);
            env.storage()
                .instance()
                .set(&RoyaltyRecipient, &royalty_recipient);
            env.storage().instance().set(&State, &true);

            bump_instance(&env);
            events::initialized(&env, &admin, payment_token);
            Ok(())
        }

        /// List an NFT for sale at a fixed price.
        ///
        /// Returns the listing ID.
        ///
        /// # Errors
        /// - [`MarketplaceError::NotInitialized`]
        /// - [`MarketplaceError::Unauthorized`]
        pub fn list(
            env: Env,
            seller: Address,
            token_id: u32,
            price: i128,
        ) -> Result<u64, MarketplaceError> {
            seller.require_auth();

            let listing_id: u64 = env
                .storage()
                .instance()
                .get(&DataKey::NextListingId)
                .unwrap_or(0);
            let next_id = listing_id
                .checked_add(1)
                .ok_or(MarketplaceError::StorageError)?;
            env.storage().instance().set(&DataKey::NextListingId, &next_id);

            let listing = Listing {
                seller: seller.clone(),
                token_id,
                price,
                active: true,
            };
            env.storage()
                .persistent()
                .set(&DataKey::Listing(listing_id), &listing);
            bump_listing(&env, listing_id);
            bump_instance(&env);

            events::listed(&env, &seller, listing_id, token_id, price);
            Ok(listing_id)
        }

        /// Buy a listed NFT at its fixed price.
        ///
        /// Transfers payment (minus royalty) to the seller and the royalty portion to the royalty
        /// recipient, then transfers the NFT to the buyer.
        ///
        /// # Errors
        /// - [`MarketplaceError::NotInitialized`]
        /// - [`MarketplaceError::ListingNotFound`]
        /// - [`MarketplaceError::ListingNotActive`]
        /// - [`MarketplaceError::InsufficientPayment`]
        pub fn buy(
            env: Env,
            buyer: Address,
            listing_id: u64,
            payment_amount: i128,
        ) -> Result<(), MarketplaceError> {
            buyer.require_auth();

            let listing: Listing = env
                .storage()
                .persistent()
                .get(&DataKey::Listing(listing_id))
                .ok_or(MarketplaceError::ListingNotFound)?;
            if !listing.active {
                return Err(MarketplaceError::ListingNotActive);
            }
            if payment_amount < listing.price {
                return Err(MarketplaceError::InsufficientPayment);
            }

            listing.active = false;
            env.storage()
                .persistent()
                .set(&DataKey::Listing(listing_id), &listing);
            bump_listing(&env, listing_id);
            bump_instance(&env);

            let price = listing.price;
            let royalty_bps: u32 = env
                .storage()
                .instance()
                .get(&RoyaltyBps)
                .unwrap_or(0);
            let royalty_recipient: Address = env
                .storage()
                .instance()
                .get(&RoyaltyRecipient)
                .unwrap();

            let royalty = apply_bps_fee(price, royalty_bps).unwrap_or(0);
            #[allow(clippy::arithmetic_side_effects)]
            let seller_amount = price - royalty;

            let tok = token::Client::new(&env, &PaymentToken);
            tok.transfer(&buyer, &listing.seller, &seller_amount);
            if royalty > 0 {
                tok.transfer(&buyer, &royalty_recipient, &royalty);
            }

            events::sold(&env, &buyer, listing_id, listing.token_id, price);
            Ok(())
        }

        /// Cancel a listing. Only the seller or admin can cancel.
        ///
        /// # Errors
        /// - [`MarketplaceError::NotInitialized`]
        /// - [`MarketplaceError::Unauthorized`]
        /// - [`MarketplaceError::ListingNotFound`]
        /// - [`MarketplaceError::ListingNotActive`]
        pub fn cancel(
            env: Env,
            caller: Address,
            listing_id: u64,
        ) -> Result<(), MarketplaceError> {
            let listing: Listing = env
                .storage()
                .persistent()
                .get(&DataKey::Listing(listing_id))
                .ok_or(MarketplaceError::ListingNotFound)?;
            if !listing.active {
                return Err(MarketplaceError::ListingNotActive);
            }

            let admin: Address = env.storage().instance().get(&Admin).unwrap();
            if caller != listing.seller && caller != admin {
                return Err(MarketplaceError::Unauthorized);
            }

            listing.active = false;
            env.storage()
                .persistent()
                .set(&DataKey::Listing(listing_id), &listing);
            bump_listing(&env, listing_id);
            bump_instance(&env);

            events::cancelled(&env, &caller, listing_id);
            Ok(())
        }

        /// Get listing details.
        ///
        /// # Errors
        /// - [`MarketplaceError::NotInitialized`]
        /// - [`MarketplaceError::ListingNotFound`]
        pub fn get_listing(
            env: Env,
            listing_id: u64,
        ) -> Result<ListingEntry, MarketplaceError> {
            let listing: Listing = env
                .storage()
                .persistent()
                .get(&DataKey::Listing(listing_id))
                .ok_or(MarketplaceError::ListingNotFound)?;
            Ok(ListingEntry {
                listing_id,
                listing,
            })
        }

        /// Get a page of active listings.
        ///
        /// # Errors
        /// - [`MarketplaceError::NotInitialized`]
        pub fn get_active_listings(
            env: Env,
            seller: Address,
            listing_id: u64,
            buyer: Address,
        ) -> Result<(), MarketplaceError> {
            let royalty_bps: u32 = env
                .storage()
                .instance()
                .get(&DataKey::RoyaltyBps)
                .unwrap_or(0);
            let royalty_recipient: Address = env
                .storage()
                .instance()
                .get(&DataKey::RoyaltyRecipient)
                .ok_or(MarketplaceError::NotInitialized)?;
            let payment_token: Address = env
                .storage()
                .instance()
                .get(&DataKey::PaymentToken)
                .ok_or(MarketplaceError::NotInitialized)?;

            seller.require_auth();

            let mut listing: Listing = env
                .storage()
                .persistent()
                .get(&DataKey::Listing(listing_id))
                .ok_or(MarketplaceError::ListingNotFound)?;
            if !listing.active {
                return Err(MarketplaceError::ListingInactive);
            }
            if listing.seller != seller {
                return Err(MarketplaceError::NotAuthorized);
            }

            // Re-check the escrowed offer at accept time.
            let offer_key = DataKey::Offer(listing_id, buyer.clone());
            let amount: i128 = env
                .storage()
                .persistent()
                .get(&offer_key)
                .ok_or(MarketplaceError::OfferNotFound)?;

            // Checks-effects-interactions: mark inactive and clear the offer before external calls.
            listing.active = false;
            env.storage()
                .persistent()
                .set(&DataKey::Listing(listing_id), &listing);
            env.storage().persistent().remove(&offer_key);
            bump_instance(&env);

            #[allow(clippy::arithmetic_side_effects, clippy::as_conversions, clippy::cast_possible_truncation, clippy::integer_division)] // royalty BPS validated <= 10_000 at init
            let royalty = (amount * royalty_bps as i128) / 10_000;
            #[allow(clippy::arithmetic_side_effects)]
            let seller_amount = amount - royalty;

            let tok = token::Client::new(&env, &payment_token);
            tok.transfer(
                &env.current_contract_address(),
                &listing.seller,
                &seller_amount,
            );
            if royalty > 0 {
                tok.transfer(&env.current_contract_address(), &royalty_recipient, &royalty);
            }

            // Transfer the NFT from seller to buyer.
            NftClient::new(&env, &listing.nft_contract).transfer_from(
                &env.current_contract_address(),
                &listing.seller,
                &buyer,
                &listing.token_id,
            );

            events::offer_accepted(&env, listing_id, &buyer, amount);
            Ok(())
        }

        /// Cancel an unaccepted offer, refunding the escrowed amount to the buyer.
        ///
        /// # Errors
        ///
        /// Returns [`MarketplaceError::NotInitialized`] if not initialized.
        /// Returns [`MarketplaceError::OfferNotFound`] if `buyer` has no active offer on this listing.
        pub fn cancel_offer(
            env: Env,
            buyer: Address,
            listing_id: u64,
        ) -> Result<(), MarketplaceError> {
            let payment_token: Address = env
                .storage()
                .instance()
                .get(&DataKey::PaymentToken)
                .ok_or(MarketplaceError::NotInitialized)?;

            buyer.require_auth();

            let offer_key = DataKey::Offer(listing_id, buyer.clone());
            let amount: i128 = env
                .storage()
                .persistent()
                .get(&offer_key)
                .ok_or(MarketplaceError::OfferNotFound)?;

            env.storage().persistent().remove(&offer_key);
            bump_instance(&env);

            token::Client::new(&env, &payment_token).transfer(
                &env.current_contract_address(),
                &buyer,
                &amount,
            );

            events::offer_cancelled(&env, listing_id, &buyer);
            Ok(())
        }

        /// Return listing details, or `None` if not found.
        pub fn get_listing(env: Env, listing_id: u64) -> Option<Listing> {
            env.storage()
                .persistent()
                .get(&DataKey::Listing(listing_id))
        }

        /// Return the escrowed offer amount from `buyer` on `listing_id`, or `None` if none exists.
        pub fn get_offer(env: Env, listing_id: u64, buyer: Address) -> Option<i128> {
            env.storage()
                .persistent()
                .get(&DataKey::Offer(listing_id, buyer))
        }

        /// Enumerate active listings, paginated by listing ID.
        ///
        /// `cursor` is the listing ID to resume scanning from (pass `0` to start from the
        /// beginning). `limit` is the maximum number of active listings to return and is
        /// capped at [`MAX_LISTINGS_PAGE_SIZE`] (a `limit` of `0` is treated as `1`).
        ///
        /// Returns a page of matching listings together with a `next_cursor` to pass to
        /// the following call, or `next_cursor: None` once the end of the listing range
        /// has been reached.
        pub fn get_active_listings(env: Env, cursor: u64, limit: u32) -> ListingPage {
            let capped_limit = limit.clamp(1, MAX_LISTINGS_PAGE_SIZE);
            let next_id: u64 = env
                .storage()
                .instance()
                .get(&DataKey::NextListingId)
                .unwrap_or(0);

            let mut listings = Vec::new(&env);
            let mut id = cursor;
            cursor: u64,
            limit: u32,
        ) -> Result<ListingPage, MarketplaceError> {
            let max = limit.min(50);
            let mut items = Vec::new(&env);
            let mut next_cursor = None;
            let mut current = cursor;

            loop {
                if items.len() >= max {
                    next_cursor = Some(current);
                    break;
                }
                if let Some(listing) = env
                    .storage()
                    .persistent()
                    .get::<_, Listing>(&DataKey::Listing(current))
                {
                    if listing.active {
                        items.push_back(ListingEntry {
                            listing_id: current,
                            listing,
                        });
                    }
                }
                let next = current.checked_add(1);
                match next {
                    Some(n) => current = n,
                    None => break,
                }
            }

            Ok(ListingPage {
                items,
                next_cursor,
            })
        }
    }
}
