#![no_std]
#![deny(missing_docs)]
//! NFT/asset marketplace contract template.
//!
//! Sellers list assets at a fixed price; buyers purchase them, transferring
//! payment to the seller and the asset to the buyer in one transaction.

use soroban_sdk::{Address, Env, contract, contractclient, contractimpl, token};

mod errors;
mod events;
mod storage;

pub use errors::MarketplaceError;
pub use storage::{DataKey, Listing};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn bump_listing(env: &Env, id: u64) {
    env.storage().persistent().extend_ttl(
        &DataKey::Listing(id),
        LEDGER_LIFETIME_THRESHOLD,
        LEDGER_BUMP_AMOUNT,
    );
}

pub use contract::*;

// The `#[contract]` / `#[contractimpl]` / `#[contractclient]` macros generate
// undocumented public client items. Confine the missing_docs allowance to this
// module and re-export the public contract API above.
mod contract {
    #![allow(missing_docs)]
    use super::*;

    /// Minimal interface we need from the NFT contract: transfer_from.
    #[contractclient(name = "NftClient")]
    pub trait NftInterface {
        fn transfer_from(env: Env, spender: Address, from: Address, to: Address, token_id: u32);
    }

    /// NFT marketplace contract.
    ///
    /// Lifecycle:
    /// 1. Admin calls `initialize` to set the payment token, royalty BPS (0–10 000), and royalty recipient.
    /// 2. Seller calls `list(nft_contract, token_id, price)` — the seller must first `approve` this
    ///    marketplace contract as a spender on the NFT.
    /// 3. Buyer calls `buy(listing_id)` — pays the seller and the royalty recipient, then the NFT is
    ///    transferred to the buyer.
    /// 4. Seller may call `cancel(listing_id)` to delist before a sale.
    #[contract]
    pub struct MarketplaceContract;

    #[contractimpl]
    impl MarketplaceContract {
        /// Initialize the marketplace.
        ///
        /// `royalty_bps` is in basis points (0 = no royalty, 10 000 = 100 %).
        ///
        /// # Errors
        ///
        /// Returns [`MarketplaceError::AlreadyInitialized`] if already initialized.
        /// Returns [`MarketplaceError::InvalidRoyalty`] if `royalty_bps > 10_000`.
        pub fn initialize(
            env: Env,
            admin: Address,
            payment_token: Address,
            royalty_bps: u32,
            royalty_recipient: Address,
        ) -> Result<(), MarketplaceError> {
            if env.storage().instance().has(&DataKey::Admin) {
                return Err(MarketplaceError::AlreadyInitialized);
            }
            if royalty_bps > 10_000 {
                return Err(MarketplaceError::InvalidRoyalty);
            }

            admin.require_auth();

            // Sanity-check the token address implements the token interface.
            token::Client::new(&env, &payment_token).decimals();

            env.storage().instance().set(&DataKey::Admin, &admin);
            env.storage()
                .instance()
                .set(&DataKey::PaymentToken, &payment_token);
            env.storage()
                .instance()
                .set(&DataKey::RoyaltyBps, &royalty_bps);
            env.storage()
                .instance()
                .set(&DataKey::RoyaltyRecipient, &royalty_recipient);
            env.storage().instance().set(&DataKey::NextListingId, &0u64);
            bump_instance(&env);
            Ok(())
        }

        /// List an NFT for sale.
        ///
        /// The seller must have called `nft_contract.approve(seller, marketplace, token_id, expiry)`
        /// before listing so the marketplace can transfer the NFT on sale.
        ///
        /// # Errors
        ///
        /// Returns [`MarketplaceError::NotInitialized`] if not yet initialized.
        /// Returns [`MarketplaceError::InvalidPrice`] if `price <= 0`.
        pub fn list(
            env: Env,
            seller: Address,
            nft_contract: Address,
            token_id: u32,
            price: i128,
        ) -> Result<u64, MarketplaceError> {
            if !env.storage().instance().has(&DataKey::Admin) {
                return Err(MarketplaceError::NotInitialized);
            }
            if price <= 0 {
                return Err(MarketplaceError::InvalidPrice);
            }

            seller.require_auth();

            let id: u64 = env
                .storage()
                .instance()
                .get(&DataKey::NextListingId)
                .unwrap_or(0);

            let listing = Listing {
                nft_contract,
                token_id,
                seller: seller.clone(),
                price,
                active: true,
            };

            env.storage()
                .persistent()
                .set(&DataKey::Listing(id), &listing);
            env.storage()
                .instance()
                .set(&DataKey::NextListingId, &(id + 1));
            bump_listing(&env, id);
            bump_instance(&env);

            events::listed(&env, id, &seller, price);
            Ok(id)
        }

        /// Buy the NFT in listing `listing_id`.
        ///
        /// Transfers payment (minus royalty) to the seller and the royalty portion to the royalty
        /// recipient, then transfers the NFT to the buyer.
        ///
        /// # Errors
        ///
        /// Returns [`MarketplaceError::NotInitialized`] if not initialized.
        /// Returns [`MarketplaceError::ListingNotFound`] if `listing_id` does not exist.
        /// Returns [`MarketplaceError::ListingInactive`] if the listing was cancelled or already sold.
        pub fn buy(env: Env, buyer: Address, listing_id: u64) -> Result<(), MarketplaceError> {
            let payment_token: Address = env
                .storage()
                .instance()
                .get(&DataKey::PaymentToken)
                .ok_or(MarketplaceError::NotInitialized)?;
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

            buyer.require_auth();

            let mut listing: Listing = env
                .storage()
                .persistent()
                .get(&DataKey::Listing(listing_id))
                .ok_or(MarketplaceError::ListingNotFound)?;

            if !listing.active {
                return Err(MarketplaceError::ListingInactive);
            }

            // Checks-effects-interactions: mark inactive before external calls.
            listing.active = false;
            env.storage()
                .persistent()
                .set(&DataKey::Listing(listing_id), &listing);
            bump_listing(&env, listing_id);
            bump_instance(&env);

            let price = listing.price;
            let royalty = (price * royalty_bps as i128) / 10_000;
            let seller_amount = price - royalty;

            let tok = token::Client::new(&env, &payment_token);
            tok.transfer(&buyer, &listing.seller, &seller_amount);
            if royalty > 0 {
                tok.transfer(&buyer, &royalty_recipient, &royalty);
            }

            // Transfer the NFT from seller to buyer.
            NftClient::new(&env, &listing.nft_contract).transfer_from(
                &env.current_contract_address(),
                &listing.seller,
                &buyer,
                &listing.token_id,
            );

            events::sold(&env, listing_id, &buyer, price);
            Ok(())
        }

        /// Cancel a listing. Only the original seller may cancel.
        ///
        /// # Errors
        ///
        /// Returns [`MarketplaceError::NotInitialized`] if not initialized.
        /// Returns [`MarketplaceError::ListingNotFound`] if the listing does not exist.
        /// Returns [`MarketplaceError::ListingInactive`] if already sold or cancelled.
        /// Returns [`MarketplaceError::NotAuthorized`] if caller is not the seller.
        pub fn cancel(env: Env, seller: Address, listing_id: u64) -> Result<(), MarketplaceError> {
            if !env.storage().instance().has(&DataKey::Admin) {
                return Err(MarketplaceError::NotInitialized);
            }

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

            listing.active = false;
            env.storage()
                .persistent()
                .set(&DataKey::Listing(listing_id), &listing);
            bump_listing(&env, listing_id);
            bump_instance(&env);

            events::cancelled(&env, listing_id, &seller);
            Ok(())
        }

        /// Return listing details, or `None` if not found.
        pub fn get_listing(env: Env, listing_id: u64) -> Option<Listing> {
            env.storage()
                .persistent()
                .get(&DataKey::Listing(listing_id))
        }
    }
}

#[cfg(test)]
mod test;
