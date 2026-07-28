#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects, clippy::indexing_slicing)]
//! Integration tests: deploys both TokenContract and EscrowContract in the
//! same Soroban test environment and exercises the full escrow lifecycle.
//!
//! Closes #221 – no integration test between token and escrow contracts.
//! Closes #222 – escrow tests used a mock token address; fund/release path untested.

#![cfg(test)]

use soroban_sdk::{
    Address, Env, String,
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
};

use soroban_escrow_template::{EscrowContract, EscrowContractClient, EscrowState};
use soroban_token_template::{TokenContract, TokenContractClient};

// ── helpers ──────────────────────────────────────────────────────────────────

fn deploy_token<'a>(env: &'a Env, admin: &Address) -> (TokenContractClient<'a>, Address) {
    let addr = env.register_contract(None, TokenContract);
    let client = TokenContractClient::new(env, &addr);
    client.initialize(
        admin,
        &String::from_str(env, "Test Token"),
        &String::from_str(env, "TEST"),
        &18u32,
        &None,
    );
    (client, addr)
}

fn deploy_escrow<'a>(env: &'a Env) -> (EscrowContractClient<'a>, Address) {
    let addr = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(env, &addr);
    (client, addr)
}

// ── #221 / #222: full happy-path lifecycle ────────────────────────────────────

/// initialize → fund → mark_delivered → approve_delivery
/// Verifies that real token balances move correctly at each step.
#[test]
fn test_full_escrow_lifecycle_happy_path() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 200;

    let (token, token_addr) = deploy_token(&env, &token_admin);
    token.mint(&buyer, &amount);
    assert_eq!(token.balance(&buyer), amount);

    let (escrow, escrow_addr) = deploy_escrow(&env);
    escrow.initialize(&buyer, &seller, &arbiter, &token_addr, &amount, &deadline);

    // fund: buyer's tokens move into the escrow contract
    escrow.fund();
    assert_eq!(token.balance(&buyer), 0);
    assert_eq!(token.balance(&escrow_addr), amount);

    // mark delivered by seller
    escrow.mark_delivered();
    assert_eq!(escrow.get_state(), Some(EscrowState::Delivered));

    // buyer approves → tokens released to seller
    escrow.approve_delivery();
    assert_eq!(escrow.get_state(), Some(EscrowState::Completed));
    assert_eq!(token.balance(&escrow_addr), 0);
    assert_eq!(token.balance(&seller), amount);
}

/// initialize → fund → deadline passes → request_refund
/// Verifies tokens are returned to the buyer.
#[test]
fn test_full_escrow_lifecycle_refund_after_deadline() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let amount = 500i128;
    let deadline = env.ledger().sequence() + 200;

    let (token, token_addr) = deploy_token(&env, &token_admin);
    token.mint(&buyer, &amount);

    let (escrow, escrow_addr) = deploy_escrow(&env);
    escrow.initialize(&buyer, &seller, &arbiter, &token_addr, &amount, &deadline);
    escrow.fund();

    assert_eq!(token.balance(&escrow_addr), amount);

    // advance past deadline
    env.ledger().with_mut(|l| l.sequence_number = deadline + 1);
    assert!(escrow.is_deadline_passed());

    escrow.request_refund();
    assert_eq!(escrow.get_state(), Some(EscrowState::Refunded));
    assert_eq!(token.balance(&buyer), amount);
    assert_eq!(token.balance(&escrow_addr), 0);
}

/// initialize → fund → raise_dispute → arbiter resolves (both paths)
/// Verifies Disputed state is reached and token balances after each resolution.
#[test]
fn test_escrow_dispute_and_arbiter_resolution() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 200;

    // ── Path 1: arbiter releases to seller ──────────────────────────────
    let (token, token_addr) = deploy_token(&env, &token_admin);
    token.mint(&buyer, &amount);

    let (escrow, escrow_addr) = deploy_escrow(&env);
    escrow.initialize(&buyer, &seller, &arbiter, &token_addr, &amount, &deadline);
    escrow.fund();

    assert_eq!(token.balance(&escrow_addr), amount);
    assert_eq!(token.balance(&buyer), 0);

    escrow.raise_dispute(&buyer);
    assert_eq!(escrow.get_state(), Some(EscrowState::Disputed));
    assert_eq!(token.balance(&escrow_addr), amount);

    escrow.resolve_dispute(&true);
    assert_eq!(escrow.get_state(), Some(EscrowState::Completed));
    assert_eq!(token.balance(&seller), amount);
    assert_eq!(token.balance(&escrow_addr), 0);
    assert_eq!(token.balance(&buyer), 0);

    // ── Path 2: arbiter refunds to buyer ────────────────────────────────
    let buyer2 = Address::generate(&env);
    let seller2 = Address::generate(&env);
    let arbiter2 = Address::generate(&env);

    token.mint(&buyer2, &amount);

    let (escrow2, escrow_addr2) = deploy_escrow(&env);
    escrow2.initialize(
        &buyer2,
        &seller2,
        &arbiter2,
        &token_addr,
        &amount,
        &deadline,
    );
    escrow2.fund();

    assert_eq!(token.balance(&escrow_addr2), amount);

    escrow2.raise_dispute(&seller2);
    assert_eq!(escrow2.get_state(), Some(EscrowState::Disputed));

    escrow2.resolve_dispute(&false);
    assert_eq!(escrow2.get_state(), Some(EscrowState::Refunded));
    assert_eq!(token.balance(&buyer2), amount);
    assert_eq!(token.balance(&escrow_addr2), 0);
    assert_eq!(token.balance(&seller2), 0);
}

/// initialize → fund → arbiter resolves in favour of seller
#[test]
fn test_full_escrow_lifecycle_arbiter_resolves_to_seller() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let amount = 750i128;
    let deadline = env.ledger().sequence() + 200;

    let (token, token_addr) = deploy_token(&env, &token_admin);
    token.mint(&buyer, &amount);

    let (escrow, escrow_addr) = deploy_escrow(&env);
    escrow.initialize(&buyer, &seller, &arbiter, &token_addr, &amount, &deadline);
    escrow.fund();

    escrow.raise_dispute(&buyer);
    escrow.resolve_dispute(&true); // true → release to seller
    assert_eq!(escrow.get_state(), Some(EscrowState::Completed));
    assert_eq!(token.balance(&seller), amount);
    assert_eq!(token.balance(&escrow_addr), 0);
}

/// initialize → fund → arbiter resolves in favour of buyer
#[test]
fn test_full_escrow_lifecycle_arbiter_resolves_to_buyer() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let amount = 300i128;
    let deadline = env.ledger().sequence() + 200;

    let (token, token_addr) = deploy_token(&env, &token_admin);
    token.mint(&buyer, &amount);

    let (escrow, escrow_addr) = deploy_escrow(&env);
    escrow.initialize(&buyer, &seller, &arbiter, &token_addr, &amount, &deadline);
    escrow.fund();

    escrow.raise_dispute(&buyer);
    escrow.resolve_dispute(&false); // false → refund to buyer
    assert_eq!(escrow.get_state(), Some(EscrowState::Refunded));
    assert_eq!(token.balance(&buyer), amount);
    assert_eq!(token.balance(&escrow_addr), 0);
}

/// initialize → cancel (no funds involved)
#[test]
fn test_full_escrow_lifecycle_cancel_before_fund() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let amount = 200i128;
    let deadline = env.ledger().sequence() + 200;

    let (token, token_addr) = deploy_token(&env, &token_admin);
    token.mint(&buyer, &amount);

    let (escrow, _) = deploy_escrow(&env);
    escrow.initialize(&buyer, &seller, &arbiter, &token_addr, &amount, &deadline);

    escrow.cancel();
    assert_eq!(escrow.get_state(), Some(EscrowState::Cancelled));
    // buyer still holds all tokens – nothing was transferred
    assert_eq!(token.balance(&buyer), amount);
}

/// initialize → fund → mark_delivered → approve_delivery with capped-supply token
/// Verifies escrow works correctly when token has a supply cap.
#[test]
fn test_escrow_with_capped_token() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let max_supply = 1_000i128;
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 200;

    // Deploy token with max_supply set
    let addr = env.register_contract(None, TokenContract);
    let token = TokenContractClient::new(&env, &addr);
    token.initialize(
        &token_admin,
        &String::from_str(&env, "Capped Token"),
        &String::from_str(&env, "CAP"),
        &18u32,
        &Some(max_supply),
    );
    let token_addr = addr;

    // Mint full supply to buyer
    token.mint(&buyer, &max_supply);
    assert_eq!(token.balance(&buyer), max_supply);

    let (escrow, escrow_addr) = deploy_escrow(&env);
    escrow.initialize(&buyer, &seller, &arbiter, &token_addr, &amount, &deadline);

    // fund: buyer's tokens move into the escrow contract
    escrow.fund();
    assert_eq!(token.balance(&buyer), 0);
    assert_eq!(token.balance(&escrow_addr), amount);

    // mark delivered by seller
    escrow.mark_delivered();
    assert_eq!(escrow.get_state(), Some(EscrowState::Delivered));

    // buyer approves → tokens released to seller
    escrow.approve_delivery();
    assert_eq!(escrow.get_state(), Some(EscrowState::Completed));
    assert_eq!(token.balance(&escrow_addr), 0);
    assert_eq!(token.balance(&seller), amount);
}

// ── SAC-based token variant (mirrors original escrow tests) ──────────────────

/// Same happy-path but using a Stellar Asset Contract token instead of the
/// custom TokenContract, confirming the escrow works with both token types.
#[test]
fn test_full_escrow_lifecycle_with_sac_token() {
    let env = Env::default();
    env.mock_all_auths();

    let sac_admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 200;

    let sac = env.register_stellar_asset_contract_v2(sac_admin.clone());
    let token_addr = sac.address();
    StellarAssetClient::new(&env, &token_addr).mint(&buyer, &amount);

    let (escrow, escrow_addr) = deploy_escrow(&env);
    escrow.initialize(&buyer, &seller, &arbiter, &token_addr, &amount, &deadline);
    escrow.fund();

    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token_addr).balance(&escrow_addr),
        amount
    );

    escrow.mark_delivered();
    escrow.approve_delivery();

    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token_addr).balance(&seller),
        amount
    );
}

// ── #442: token allowance expiry flow ──────────────────────────────────────

/// mint → approve with short expiry → advance ledger past expiry → transfer_from fails
/// Verifies that expired allowances cannot be used and balances remain unchanged.
#[test]
fn test_token_allowance_expiry_in_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let amount = 1_000i128;
    let expiry = env.ledger().sequence() + 10;

    let (token, _) = deploy_token(&env, &token_admin);
    token.mint(&owner, &amount);
    assert_eq!(token.balance(&owner), amount);

    // approve with short expiry
    token.approve(&owner, &spender, &amount, &expiry);
    assert_eq!(token.allowance(&owner, &spender), amount);

    // advance ledger past expiry
    env.ledger().with_mut(|l| l.sequence_number = expiry + 1);

    // transfer_from should fail due to expired allowance
    let result = token.try_transfer_from(&spender, &owner, &receiver, &amount);
    assert!(result.is_err());

    // verify balances unchanged
    assert_eq!(token.balance(&owner), amount);
    assert_eq!(token.balance(&receiver), 0);
}

// ── #857: marketplace integration test ───────────────────────────────────────

use soroban_marketplace_template::{MarketplaceContract, MarketplaceContractClient};
use soroban_nft_template::{NftContract, NftContractClient};

fn deploy_nft<'a>(env: &'a Env, admin: &Address) -> (NftContractClient<'a>, Address) {
    let addr = env.register_contract(None, NftContract);
    let client = NftContractClient::new(env, &addr);
    client.initialize(
        admin,
        &String::from_str(env, "Test NFT"),
        &String::from_str(env, "TNFT"),
        &10u32,
    );
    (client, addr)
}

fn deploy_marketplace<'a>(env: &'a Env) -> (MarketplaceContractClient<'a>, Address) {
    let addr = env.register_contract(None, MarketplaceContract);
    let client = MarketplaceContractClient::new(env, &addr);
    (client, addr)
}

/// Full lifecycle: list → buy (with royalty split) → verify NFT ownership transfer
/// Closes #857 – integration test harness for marketplace (post build-fix)
#[test]
fn test_marketplace_full_lifecycle_with_royalty() {
    let env = Env::default();
    env.mock_all_auths();

    let nft_admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let marketplace_admin = Address::generate(&env);
    let royalty_recipient = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Deploy NFT contract and mint a token to seller
    let (nft, nft_addr) = deploy_nft(&env, &nft_admin);
    let token_id = 1u32;
    nft.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "https://example.com/token/1"),
    );
    assert_eq!(nft.owner_of(token_id).unwrap(), seller);

    // Deploy payment token and mint to buyer
    let (token, token_addr) = deploy_token(&env, &token_admin);
    let price = 1_000i128;
    let buyer_initial_balance = 5_000i128;
    token.mint(&buyer, &buyer_initial_balance);
    assert_eq!(token.balance(&buyer), buyer_initial_balance);

    // Deploy and initialize marketplace with 2.5% royalty
    let (marketplace, marketplace_addr) = deploy_marketplace(&env);
    let royalty_bps = 250u32; // 2.5%
    marketplace.initialize(
        &marketplace_admin,
        &token_addr,
        &royalty_bps,
        &royalty_recipient,
    );

    // Seller approves marketplace to transfer the NFT
    nft.approve(&token_id, &marketplace_addr);

    // Seller lists the NFT
    let listing_id = marketplace.list(&seller, &nft_addr, &token_id, &price);
    let listing = marketplace.get_listing(listing_id).unwrap();
    assert_eq!(listing.seller, seller);
    assert_eq!(listing.price, price);
    assert_eq!(listing.active, true);

    // Buyer purchases the NFT
    marketplace.buy(&buyer, &listing_id);

    // Verify NFT ownership transferred to buyer
    assert_eq!(nft.owner_of(token_id).unwrap(), buyer);

    // Verify payment distribution with royalty split
    let royalty_amount = (price * i128::from(royalty_bps)) / 10_000i128;
    let seller_amount = price - royalty_amount;
    assert_eq!(token.balance(&seller), seller_amount);
    assert_eq!(token.balance(&royalty_recipient), royalty_amount);
    assert_eq!(token.balance(&buyer), buyer_initial_balance - price);

    // Verify listing is now inactive
    let listing_after = marketplace.get_listing(listing_id).unwrap();
    assert_eq!(listing_after.active, false);
}

/// List → cancel → verify NFT remains with seller
/// Closes #857 – integration test harness for marketplace (post build-fix)
#[test]
fn test_marketplace_cancel_listing() {
    let env = Env::default();
    env.mock_all_auths();

    let nft_admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let marketplace_admin = Address::generate(&env);
    let royalty_recipient = Address::generate(&env);
    let seller = Address::generate(&env);

    // Deploy NFT contract and mint a token to seller
    let (nft, nft_addr) = deploy_nft(&env, &nft_admin);
    let token_id = 1u32;
    nft.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "https://example.com/token/1"),
    );
    assert_eq!(nft.owner_of(token_id).unwrap(), seller);

    // Deploy payment token
    let (_, token_addr) = deploy_token(&env, &token_admin);

    // Deploy and initialize marketplace
    let (marketplace, marketplace_addr) = deploy_marketplace(&env);
    marketplace.initialize(&marketplace_admin, &token_addr, &250u32, &royalty_recipient);

    // Seller approves and lists the NFT
    nft.approve(&token_id, &marketplace_addr);
    let listing_id = marketplace.list(&seller, &nft_addr, &token_id, &1_000i128);

    // Seller cancels the listing
    marketplace.cancel(&seller, &listing_id);

    // Verify NFT still belongs to seller
    assert_eq!(nft.owner_of(token_id).unwrap(), seller);

    // Verify listing is now inactive
    let listing = marketplace.get_listing(listing_id).unwrap();
    assert_eq!(listing.active, false);
}
