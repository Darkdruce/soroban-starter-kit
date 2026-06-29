#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects, clippy::indexing_slicing)]
#![cfg(test)]

use super::*;
use soroban_sdk::{
    Address, Env, contract, contractimpl, contracttype,
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
};

// ---------------------------------------------------------------------------
// Minimal mock NFT contract for testing
// ---------------------------------------------------------------------------

#[contracttype]
enum NftKey {
    Owner(u32),
    Approved(u32),
}

#[contract]
pub struct MockNft;

#[contractimpl]
impl MockNft {
    pub fn init(env: Env, owner: Address, token_id: u32) {
        env.storage()
            .persistent()
            .set(&NftKey::Owner(token_id), &owner);
    }

    pub fn approve(env: Env, _caller: Address, spender: Address, token_id: u32, _expiry: u32) {
        env.storage()
            .persistent()
            .set(&NftKey::Approved(token_id), &spender);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, token_id: u32) {
        let approved: Address = env
            .storage()
            .persistent()
            .get(&NftKey::Approved(token_id))
            .expect("no approval");
        assert_eq!(approved, spender);
        let owner: Address = env
            .storage()
            .persistent()
            .get(&NftKey::Owner(token_id))
            .expect("no owner");
        assert_eq!(owner, from);
        env.storage()
            .persistent()
            .set(&NftKey::Owner(token_id), &to);
    }

    pub fn owner_of(env: Env, token_id: u32) -> Address {
        env.storage()
            .persistent()
            .get(&NftKey::Owner(token_id))
            .expect("token not found")
    }
}

// ---------------------------------------------------------------------------
// Test setup
// ---------------------------------------------------------------------------

struct TestEnv<'a> {
    env: Env,
    client: MarketplaceContractClient<'a>,
    marketplace: Address,
    token: Address,
    nft: Address,
    admin: Address,
    seller: Address,
    buyer: Address,
    royalty_recipient: Address,
}

fn setup<'a>(env: &'a Env) -> TestEnv<'a> {
    env.mock_all_auths();

    let admin = Address::generate(env);
    let seller = Address::generate(env);
    let buyer = Address::generate(env);
    let royalty_recipient = Address::generate(env);

    // Deploy payment token and mint to buyer
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    StellarAssetClient::new(env, &token).mint(&buyer, &10_000i128);

    // Deploy mock NFT and init token_id=1 owned by seller
    let nft = env.register_contract(None, MockNft);
    MockNftClient::new(env, &nft).init(&seller, &1u32);

    // Deploy marketplace and initialize
    let marketplace = env.register_contract(None, MarketplaceContract);
    MarketplaceContractClient::new(env, &marketplace).initialize(
        &admin,
        &token,
        &250u32,
        &royalty_recipient,
    );

    // Approve marketplace as NFT spender for token_id=1
    MockNftClient::new(env, &nft).approve(
        &seller,
        &marketplace,
        &1u32,
        &(env.ledger().sequence() + 10_000),
    );

    let client = MarketplaceContractClient::new(env, &marketplace);

    TestEnv {
        env: env.clone(),
        client,
        marketplace,
        token,
        nft,
        admin,
        seller,
        buyer,
        royalty_recipient,
    }
}

fn tok<'a>(env: &'a Env, token: &Address) -> TokenClient<'a> {
    TokenClient::new(env, token)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_rejects_duplicate() {
    let env = Env::default();
    let t = setup(&env);
    let res = t
        .client
        .try_initialize(&t.admin, &t.token, &100u32, &t.royalty_recipient);
    assert!(res.is_err());
}

#[test]
fn test_list_and_get_listing() {
    let env = Env::default();
    let t = setup(&env);
    let id = t.client.list(&t.seller, &t.nft, &1u32, &1_000i128);
    assert_eq!(id, 0);
    let listing = t.client.get_listing(&id).expect("listing");
    assert!(listing.active);
    assert_eq!(listing.price, 1_000);
    assert_eq!(listing.seller, t.seller);
}

#[test]
fn test_list_rejects_zero_price() {
    let env = Env::default();
    let t = setup(&env);
    let res = t.client.try_list(&t.seller, &t.nft, &1u32, &0i128);
    assert!(res.is_err());
}

#[test]
fn test_buy_happy_path() {
    let env = Env::default();
    let t = setup(&env);

    let price = 1_000i128;
    let id = t.client.list(&t.seller, &t.nft, &1u32, &price);

    let seller_before = tok(&env, &t.token).balance(&t.seller);
    let royalty_before = tok(&env, &t.token).balance(&t.royalty_recipient);
    let buyer_before = tok(&env, &t.token).balance(&t.buyer);

    t.client.buy(&t.buyer, &id);

    let royalty = (price * 250) / 10_000; // 25
    let seller_amount = price - royalty; // 975

    assert_eq!(
        tok(&env, &t.token).balance(&t.seller),
        seller_before + seller_amount
    );
    assert_eq!(
        tok(&env, &t.token).balance(&t.royalty_recipient),
        royalty_before + royalty
    );
    assert_eq!(tok(&env, &t.token).balance(&t.buyer), buyer_before - price);

    // Verify NFT transferred to buyer
    assert_eq!(MockNftClient::new(&env, &t.nft).owner_of(&1u32), t.buyer);

    // Listing now inactive
    let listing = t.client.get_listing(&id).expect("listing");
    assert!(!listing.active);
}

#[test]
fn test_buy_inactive_listing_fails() {
    let env = Env::default();
    let t = setup(&env);
    let id = t.client.list(&t.seller, &t.nft, &1u32, &500i128);
    t.client.buy(&t.buyer, &id);
    let res = t.client.try_buy(&t.buyer, &id);
    assert!(res.is_err());
}

#[test]
fn test_cancel_listing() {
    let env = Env::default();
    let t = setup(&env);
    let id = t.client.list(&t.seller, &t.nft, &1u32, &500i128);
    t.client.cancel(&t.seller, &id);
    let listing = t.client.get_listing(&id).expect("listing");
    assert!(!listing.active);
}

#[test]
fn test_cancel_already_cancelled_fails() {
    let env = Env::default();
    let t = setup(&env);
    let id = t.client.list(&t.seller, &t.nft, &1u32, &500i128);
    t.client.cancel(&t.seller, &id);
    let res = t.client.try_cancel(&t.seller, &id);
    assert!(res.is_err());
}

#[test]
fn test_non_seller_cannot_cancel() {
    let env = Env::default();
    let t = setup(&env);
    let id = t.client.list(&t.seller, &t.nft, &1u32, &500i128);
    let other = Address::generate(&env);
    let res = t.client.try_cancel(&other, &id);
    assert!(res.is_err());
}

#[test]
fn test_invalid_royalty_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let marketplace = env.register_contract(None, MarketplaceContract);
    let res = MarketplaceContractClient::new(&env, &marketplace)
        .try_initialize(&admin, &token, &10_001u32, &admin);
    assert!(res.is_err());
}
