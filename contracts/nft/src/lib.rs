#![no_std]
#![deny(missing_docs)]
//! Non-fungible token (NFT) contract template.
//!
//! Admin-controlled minting with per-token owners, approved spenders, metadata
//! URIs, and an optional supply cap.

use soroban_sdk::{Address, Env, String, Vec, contract, contractimpl};

mod errors;
mod events;
mod storage;

pub use errors::NftError;
pub use storage::{DataKey, RoyaltyInfo, TokenKey, TokenMetadata};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD, apply_bps_fee};

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn balance(env: &Env, owner: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(owner.clone()))
        .unwrap_or(0)
}

fn set_balance(env: &Env, owner: &Address, amount: u32) {
    let key = DataKey::Balance(owner.clone());
    if amount == 0 {
        env.storage().persistent().remove(&key);
    } else {
        env.storage().persistent().set(&key, &amount);
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
    }
}

fn is_operator(env: &Env, owner: &Address, operator: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::OperatorApproval(owner.clone(), operator.clone()))
        .unwrap_or(false)
}

/// Owner, single-token approved spender, or collection-wide operator.
fn is_authorized_spender(env: &Env, spender: &Address, owner: &Address, token_id: u32) -> bool {
    if spender == owner || is_operator(env, owner, spender) {
        return true;
    }
    env.storage()
        .instance()
        .get::<_, Address>(&TokenKey::Approval(token_id))
        .is_some_and(|approved| &approved == spender)
}

/// Move `token_id` from `from` to `to`, clearing any single-token approval and
/// updating balances. Caller must have already checked authorization.
fn move_token(env: &Env, from: &Address, to: &Address, token_id: u32) {
    env.storage()
        .instance()
        .set(&TokenKey::Owner(token_id), to);
    env.storage()
        .instance()
        .remove(&TokenKey::Approval(token_id));
    if from != to {
        set_balance(env, from, balance(env, from).saturating_sub(1));
        set_balance(env, to, balance(env, to).saturating_add(1));
    }
    bump_instance(env);
    events::transferred(env, from, to, token_id);
}

/// Non-fungible token (NFT) contract.
pub use contract::*;

mod contract {
    #![allow(missing_docs)]
    use super::*;
    use storage::DataKey::*;
    use storage::{RoyaltyInfo, TokenKey, TokenMetadata};

    #[contract]
    pub struct NftContract;

    #[contractimpl]
    impl NftContract {
        /// Initialise the NFT contract.
        ///
        /// `royalty_bps` sets a collection-level default royalty in basis points
        /// (0–10 000). `royalty_recipient` is required when `royalty_bps > 0`.
        /// Both default to no royalty when omitted (`None`).
        ///
        /// # Errors
        /// - [`NftError::AlreadyInitialized`]
        /// - [`NftError::InvalidRoyalty`] if `royalty_bps > 10 000`.
        /// - [`NftError::RoyaltyRecipientMissing`] if `royalty_bps > 0` but
        ///   `royalty_recipient` is `None`.
        pub fn initialize(
            env: Env,
            admin: Address,
            name: String,
            symbol: String,
            max_supply: Option<u32>,
            royalty_bps: Option<u32>,
            royalty_recipient: Option<Address>,
        ) -> Result<(), NftError> {
            if env.storage().instance().has(&Initialized) {
                return Err(NftError::AlreadyInitialized);
            }
            // Validate collection-level royalty parameters.
            if let Some(bps) = royalty_bps {
                if bps > 10_000 {
                    return Err(NftError::InvalidRoyalty);
                }
                if bps > 0 && royalty_recipient.is_none() {
                    return Err(NftError::RoyaltyRecipientMissing);
                }
            }
            admin.require_auth();

            env.storage().instance().set(&Admin, &admin);
            env.storage().instance().set(&Name, &name);
            env.storage().instance().set(&Symbol, &symbol);
            env.storage()
                .instance()
                .set(&DataKey::TotalSupply, &0u32);
            if let Some(supply) = max_supply {
                env.storage().instance().set(&DataKey::MaxSupply, &supply);
            }
            // Store collection-level royalty parameters if provided.
            if let Some(bps) = royalty_bps {
                env.storage().instance().set(&DataKey::RoyaltyBps, &bps);
            }
            if let Some(ref recipient) = royalty_recipient {
                env.storage()
                    .instance()
                    .set(&DataKey::RoyaltyRecipient, recipient);
            }
            env.storage().instance().set(&Initialized, &true);

            bump_instance(&env);
            events::initialized(&env, &admin, name, symbol);
            Ok(())
        }

        /// Mint a new NFT to `to`.
        ///
        /// `token_royalty_bps` and `token_royalty_recipient` are optional per-token
        /// royalty overrides (EIP-2981-style). When set they take precedence over the
        /// collection-level defaults returned by [`royalty_info`]. Both must be `None`
        /// or both must be `Some` (with `token_royalty_bps` in 0–10 000).
        ///
        /// Returns the newly minted `token_id`.
        ///
        /// # Errors
        /// - [`NftError::NotInitialized`]
        /// - [`NftError::NotAuthorized`]
        /// - [`NftError::SupplyCapReached`]
        /// - [`NftError::InvalidRoyalty`] if `token_royalty_bps > 10 000`.
        /// - [`NftError::RoyaltyRecipientMissing`] if `token_royalty_bps > 0` but
        ///   `token_royalty_recipient` is `None`.
        pub fn mint(
            env: Env,
            to: Address,
            token_uri: String,
            token_royalty_bps: Option<u32>,
            token_royalty_recipient: Option<Address>,
        ) -> Result<u32, NftError> {
            let admin: Address = env.storage().instance().get(&Admin).unwrap();
            admin.require_auth();

            let total: u32 = env
                .storage()
                .instance()
                .get(&DataKey::TotalSupply)
                .unwrap_or(0);
            let token_id = total.checked_add(1).ok_or(NftError::SupplyCapReached)?;

            if let Some(max) = env
                .storage()
                .instance()
                .get::<_, u32>(&DataKey::MaxSupply)
            {
                if token_id > max {
                    return Err(NftError::SupplyCapReached);
                }
            }

            // Validate per-token royalty parameters.
            if let Some(bps) = token_royalty_bps {
                if bps > 10_000 {
                    return Err(NftError::InvalidRoyalty);
                }
                if bps > 0 && token_royalty_recipient.is_none() {
                    return Err(NftError::RoyaltyRecipientMissing);
                }
            } else if token_royalty_recipient.is_some() {
                return Err(NftError::InvalidRoyalty);
            }

            env.storage()
                .instance()
                .set(&DataKey::TotalSupply, &token_id);
            env.storage()
                .instance()
                .set(&TokenKey::Owner(token_id), &to);
            env.storage()
                .instance()
                .set(&TokenKey::Uri(token_id), &token_uri);

            // Store per-token royalty overrides if provided.
            if let Some(bps) = token_royalty_bps {
                env.storage()
                    .instance()
                    .set(&TokenKey::TokenRoyaltyBps(token_id), &bps);
            }
            if let Some(ref recipient) = token_royalty_recipient {
                env.storage().instance().set(
                    &TokenKey::TokenRoyaltyRecipient(token_id),
                    recipient,
                );
            }

            set_balance(&env, &to, balance(&env, &to).saturating_add(1));

            bump_instance(&env);
            events::minted(&env, &to, token_id);
            Ok(token_id)
        }

        /// Return royalty information for a token sale (EIP-2981-style view).
        ///
        /// Resolution order:
        /// 1. Per-token royalty BPS and recipient (set at mint time), if present.
        /// 2. Collection-level royalty BPS and recipient (set at initialize time).
        /// 3. No royalty (`None`).
        ///
        /// [`RoyaltyInfo::amount`] is pre-computed as `sale_price * bps / 10_000`.
        ///
        /// # Errors
        /// - [`NftError::NotInitialized`]
        /// - [`NftError::TokenNotFound`] if `token_id` does not exist
        pub fn royalty_info(
            env: Env,
            token_id: u32,
            sale_price: i128,
        ) -> Result<Option<RoyaltyInfo>, NftError> {
            // 1. Per-token overrides.
            let token_bps: Option<u32> = env
                .storage()
                .instance()
                .get(&TokenKey::TokenRoyaltyBps(token_id));
            let token_recipient: Option<Address> = env
                .storage()
                .persistent()
                .get(&TokenKey::TokenRoyaltyRecipient(token_id));

            // A per-token override with `bps == 0` legitimately has no recipient
            // (`mint` only requires a recipient when `bps > 0`), so the override
            // applies whenever `token_bps` is present, not only when a recipient
            // is present too.
            if let Some(bps) = token_bps {
                if bps == 0 {
                    return Ok(None);
                }
                #[allow(
                    clippy::arithmetic_side_effects,
                    clippy::as_conversions,
                    clippy::cast_possible_truncation,
                    clippy::integer_division
                )]
                let amount = (sale_price * bps as i128) / 10_000;
                let recipient = token_recipient.ok_or(NftError::RoyaltyRecipientMissing)?;
                let amount = apply_bps_fee(sale_price, bps).unwrap_or(0);
                return Ok(Some(RoyaltyInfo { recipient, amount }));
            }

            // 2. Collection-level defaults.
            let collection_bps: Option<u32> = env.storage().instance().get(&DataKey::RoyaltyBps);
            let collection_recipient: Option<Address> =
                env.storage().instance().get(&DataKey::RoyaltyRecipient);

            if let (Some(bps), Some(recipient)) = (collection_bps, collection_recipient) {
                if bps == 0 {
                    return Ok(None);
                }
                #[allow(
                    clippy::arithmetic_side_effects,
                    clippy::as_conversions,
                    clippy::cast_possible_truncation,
                    clippy::integer_division
                )]
                let amount = (sale_price * bps as i128) / 10_000;
                let amount = apply_bps_fee(sale_price, bps).unwrap_or(0);
                return Ok(Some(RoyaltyInfo { recipient, amount }));
            }

            // 3. No royalty configured.
            Ok(None)
        }

        /// Transfer token `token_id` from `from` to `to`. Requires auth from `from` (the owner).
        ///
        /// # Errors
        /// - [`NftError::NotInitialized`]
        /// - [`NftError::TokenNotFound`]
        /// - [`NftError::NotAuthorized`]
        pub fn transfer(
            env: Env,
            from: Address,
            to: Address,
            token_id: u32,
        ) -> Result<(), NftError> {
            from.require_auth();

            let owner: Address = env
                .storage()
                .instance()
                .get(&TokenKey::Owner(token_id))
                .ok_or(NftError::TokenNotFound)?;
            if owner != from {
                return Err(NftError::NotAuthorized);
            }

            move_token(&env, &from, &to, token_id);
            Ok(())
        }

        /// Transfer `token_id` from `from` to `to` on behalf of the owner.
        ///
        /// `spender` must authorize the call and be the owner, the approved
        /// spender for `token_id`, or an operator approved via
        /// [`set_approval_for_all`]. Any single-token approval is cleared.
        ///
        /// # Errors
        /// - [`NftError::TokenNotFound`]
        /// - [`NftError::NotOwner`] if `from` does not own `token_id`
        /// - [`NftError::NotApproved`] if `spender` is not authorized
        pub fn transfer_from(
            env: Env,
            spender: Address,
            from: Address,
            to: Address,
            token_id: u32,
        ) -> Result<(), NftError> {
            spender.require_auth();

            let owner: Address = env
                .storage()
                .instance()
                .get(&TokenKey::Owner(token_id))
                .ok_or(NftError::TokenNotFound)?;
            if owner != from {
                return Err(NftError::NotOwner);
            }
            if !is_authorized_spender(&env, &spender, &owner, token_id) {
                return Err(NftError::NotApproved);
            }

            move_token(&env, &from, &to, token_id);
            Ok(())
        }

        /// Grant or revoke `operator` permission to transfer and burn all of
        /// `owner`'s tokens. Requires auth from `owner`.
        pub fn set_approval_for_all(env: Env, owner: Address, operator: Address, approved: bool) {
            owner.require_auth();

            let key = DataKey::OperatorApproval(owner.clone(), operator.clone());
            if approved {
                env.storage().persistent().set(&key, &true);
                env.storage()
                    .persistent()
                    .extend_ttl(&key, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
            } else {
                env.storage().persistent().remove(&key);
            }
            events::approval_for_all(&env, &owner, &operator, approved);
        }

        /// Whether `operator` is approved to manage all of `owner`'s tokens.
        pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
            is_operator(&env, &owner, &operator)
        }

        /// Permanently destroy `token_id`.
        ///
        /// `caller` must authorize the call and be the owner, the approved
        /// spender, or an approved operator. Removes owner, URI, approval and
        /// royalty data, decrements the owner's balance and the active supply.
        ///
        /// # Errors
        /// - [`NftError::TokenNotFound`]
        /// - [`NftError::NotApproved`] if `caller` is not authorized
        pub fn burn(env: Env, caller: Address, token_id: u32) -> Result<(), NftError> {
            caller.require_auth();

            let owner: Address = env
                .storage()
                .instance()
                .get(&TokenKey::Owner(token_id))
                .ok_or(NftError::TokenNotFound)?;
            if !is_authorized_spender(&env, &caller, &owner, token_id) {
                return Err(NftError::NotApproved);
            }

            let instance = env.storage().instance();
            instance.remove(&TokenKey::Owner(token_id));
            instance.remove(&TokenKey::Uri(token_id));
            instance.remove(&TokenKey::Approval(token_id));
            instance.remove(&TokenKey::TokenRoyaltyBps(token_id));
            instance.remove(&TokenKey::TokenRoyaltyRecipient(token_id));
            env.storage()
                .persistent()
                .remove(&TokenKey::TokenRoyaltyRecipient(token_id));

            set_balance(&env, &owner, balance(&env, &owner).saturating_sub(1));
            let burned: u32 = instance.get(&DataKey::Burned).unwrap_or(0);
            instance.set(&DataKey::Burned, &burned.saturating_add(1));

            bump_instance(&env);
            events::burned(&env, &caller, token_id);
            Ok(())
        }

        /// Number of tokens currently owned by `owner`.
        pub fn balance_of(env: Env, owner: Address) -> u32 {
            balance(&env, &owner)
        }

        /// Approve `spender` to transfer or sell token `token_id` on behalf of the owner.
        ///
        /// # Errors
        /// - [`NftError::NotInitialized`]
        /// - [`NftError::TokenNotFound`]
        /// - [`NftError::NotAuthorized`]
        pub fn approve(
            env: Env,
            owner: Address,
            spender: Address,
            token_id: u32,
        ) -> Result<(), NftError> {
            owner.require_auth();

            let current_owner: Address = env
                .storage()
                .instance()
                .get(&TokenKey::Owner(token_id))
                .ok_or(NftError::TokenNotFound)?;
            if current_owner != owner {
                return Err(NftError::NotAuthorized);
            }

            env.storage().instance().set(
                &TokenKey::Approval(token_id),
                &spender,
            );
            bump_instance(&env);
            events::approved(&env, &owner, &spender, token_id);
            Ok(())
        }

        /// Get the owner of `token_id`.
        ///
        /// # Errors
        /// - [`NftError::NotInitialized`]
        /// - [`NftError::TokenNotFound`]
        pub fn owner_of(env: Env, token_id: u32) -> Result<Address, NftError> {
            env.storage()
                .instance()
                .get(&TokenKey::Owner(token_id))
                .ok_or(NftError::TokenNotFound)
        }

        /// Get the metadata URI of `token_id`.
        ///
        /// # Errors
        /// - [`NftError::NotInitialized`]
        /// - [`NftError::TokenNotFound`]
        pub fn token_uri(env: Env, token_id: u32) -> Result<String, NftError> {
            env.storage()
                .instance()
                .get(&TokenKey::Uri(token_id))
                .ok_or(NftError::TokenNotFound)
        }

        /// Get active supply (minted minus burned).
        pub fn total_supply(env: Env) -> u32 {
            let minted: u32 = env
                .storage()
                .instance()
                .get(&DataKey::TotalSupply)
                .unwrap_or(0);
            let burned: u32 = env.storage().instance().get(&DataKey::Burned).unwrap_or(0);
            minted.saturating_sub(burned)
        }

        /// Get the collection-level royalty BPS.
        pub fn get_royalty_bps(env: Env) -> Option<u32> {
            env.storage().instance().get(&DataKey::RoyaltyBps)
        }

        /// Get the collection-level royalty recipient.
        pub fn get_royalty_recipient(env: Env) -> Option<Address> {
            env.storage().instance().get(&DataKey::RoyaltyRecipient)
        }
    }
}
