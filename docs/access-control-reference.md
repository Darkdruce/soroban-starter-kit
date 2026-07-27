# Access-Control Quick Reference

This page lists every public entry point exposed by each contract in
`contracts/` and which role's `require_auth()` check gates it, so that
auth checks (and their absence) can be audited at a glance instead of
reading each contract's full source. It exists to make gaps like the ones
found in issues #773-775 easy to spot in review.

**Methodology:** for each `#[contractimpl]` entry point, this table records
every `<address>.require_auth()` call reached from that entry point —
including calls made indirectly through a private helper function (for
example `marketplace::list` delegates to `list_impl`, which is where the
`seller.require_auth()` call actually lives). "NONE" means no address is
authenticated on that call path; the Notes column says whether that is
intentional (a read-only query, or a state change that only pays out to an
address already fixed in contract storage) or worth a second look.

This table is a manually cross-checked snapshot of the code at the time of
writing. When you add or change an entry point, please update the
corresponding row in the same PR.

---

## airdrop

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `set_root` | `admin` | |
| `claim` | `recipient` | |
| `claim_batch` | NONE | Permissionless relay: each entry is checked against the merkle root and a per-recipient claimed flag, so a relayer can submit on a recipient's behalf without their signature. |
| `is_claimed` | NONE | Read-only. |
| `get_root` | NONE | Read-only. |

## auction

| Entry Point | Auth Required | Notes |
|---|---|---|
| `start` | `seller` | |
| `bid` | `bidder` | |
| `cancel` | `seller` | |
| `end` | NONE | Permissionless settlement after the deadline; funds move per stored highest bid/seller. |
| `withdraw` | `bidder` | |
| `get_pending` | NONE | Read-only. |
| `get_info` | NONE | Read-only. |

## ballot

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `register_voter` | `admin` | |
| `deregister_voter` | `admin` | |
| `vote` | `voter` | |
| `tally` | `admin` | |
| `get_yes_votes` | NONE | Read-only. |
| `get_no_votes` | NONE | Read-only. |

## bonding-curve

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `buy` | `buyer` | |
| `sell` | `seller` | |
| `get_reserve` | NONE | Read-only. |
| `get_supply` | NONE | Read-only. |
| `get_price` | NONE | Read-only. |

## crowdfund

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `creator` | |
| `pledge` | `pledger` | |
| `withdraw` | `pledger` | |
| `extend_deadline` | `creator` | |
| `claim` | `creator` | |
| `refund` | `pledger` | |
| `get_info` | NONE | Read-only. |
| `get_pledge` | NONE | Read-only. |

## dao

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `create_proposal` | `proposer` | |
| `vote` | `voter` | |
| `execute_proposal` | NONE | Permissionless once quorum/passing conditions are met on-chain. |
| `cancel_proposal` | `admin` | |
| `get_proposal` | NONE | Read-only. |
| `proposal_count` | NONE | Read-only. |

## escrow

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | NONE | Sets up escrow state; no funds move yet. |
| `initialize_with_arbiters` | NONE | Same as above, with an arbiter set configured. |
| `update_amount` | `buyer` | Delegates to `lifecycle::update_amount`. |
| `fund` | `buyer` | Delegates to `lifecycle::fund`. |
| `mark_delivered` | `seller` | Delegates to `lifecycle::mark_delivered`. |
| `approve_delivery` | `buyer` | Delegates to `lifecycle::approve_delivery`. |
| `release_partial` | `buyer` | Delegates to `lifecycle::release_partial`. |
| `request_refund` | `buyer` | Delegates to `lifecycle::request_refund`. |
| `request_partial_refund` | `buyer` | Delegates to `lifecycle::request_partial_refund`. |
| `raise_dispute` | `caller` | Delegates to `dispute::raise_dispute`; `caller` must equal the stored buyer or seller. |
| `resolve_dispute` | `arbiter` or `caller` | Delegates to `dispute::resolve_dispute`; single-arbiter mode checks `arbiter`, multi-arbiter mode checks `caller` after verifying membership in the arbiter set. |
| `claim_dispute_timeout` | `buyer` | Delegates to `dispute::claim_dispute_timeout`; only callable after the dispute timeout elapses. |
| `cancel` | `buyer` | Delegates to `lifecycle::cancel`. |
| `extend_deadline` | `buyer` and `seller` | Delegates to `lifecycle::extend_deadline`; requires both parties' auth. |
| `bump` | NONE | Extends storage TTL only; no state change. |
| `get_escrow_info` / `get_state` / `is_deadline_passed` / `get_remaining_ledgers` / `contract_version` / `version` | NONE | Read-only. |
| `pause` / `unpause` | `admin` | Feature-gated (`pausable`). |
| `propose_upgrade` / `execute_upgrade` | `admin` | |

## lottery

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `buy_ticket` | `buyer` | |
| `commit` | `admin` | |
| `draw` | `admin` | |
| `claim_refund` | `buyer` | |
| `get_info` / `get_winner` / `get_winners` / `get_ticket_count` | NONE | Read-only. |

## marketplace

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `list` | `seller` | Delegates to `list_impl`. |
| `list_with_expiry` | `seller` | Delegates to `list_impl`. |
| `buy` | `buyer` | |
| `cancel` | `seller` | |
| `sweep_expired` | `seller` | |
| `make_offer` | `buyer` | |
| `accept_offer` | `seller` | |
| `cancel_offer` | `buyer` | |
| `get_listing` / `get_offer` / `get_active_listings` | NONE | Read-only. |

## multisig

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | every address in `signers` | Each initial signer must independently authorize. |
| `add_signer` | every address in `approvals` | Delegates to `require_threshold_approvals`; each approving address must be an existing signer and the set must meet the threshold. |
| `remove_signer` | every address in `approvals` | Same as `add_signer`. |
| `propose_transaction` | `proposer` | Must also be an existing signer. |
| `sign_transaction` | `signer` | Must also be an existing signer. |
| `execute_transaction` | NONE | Permissionless once the stored signature threshold is met. |
| `cleanup_expired` | NONE | Intentionally permissionless per doc comment — removes an expired, unexecuted proposal. |
| `get_signers` / `get_threshold` / `is_signer` / `get_transaction` / `signature_count` / `contract_version` | NONE | Read-only. |

## nft

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `mint` | `admin` | |
| `transfer` | `from` | |
| `burn` | `from` | |
| `approve` | `owner` | |
| `transfer_from` | `spender` | Must hold an active approval or be the owner. |
| `owner_of` / `get_approved` / `token_uri` / `metadata` / `name` / `symbol` / `total_supply` | NONE | Read-only. |

## oracle

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `update_price` | `admin` | |
| `set_publishers` | `admin` | |
| `submit_price` | `publisher` | Must be in the configured publisher set. |
| `get_price` / `get_price_checked` / `get_price_data` / `get_median_price` / `get_twap` | NONE | Read-only. |

## staking

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `stake` | `staker` | |
| `unstake` | `staker` | |
| `claim_rewards` | `staker` | |
| `add_rewards` | `admin` | |
| `set_compounding` | `staker` | |
| `compound` | `staker` | |
| `get_stake` / `get_rewards` / `get_total_staked` / `get_total_rewards` / `contract_version` | NONE | Read-only. |

## subscription

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `provider` | |
| `subscribe` | `subscriber` | |
| `charge` | `provider` | Pulls from the subscriber's pre-approved allowance; subscriber does not sign each charge. |
| `cancel` | `subscriber` | |
| `get_subscription` / `get_provider` / `get_token` | NONE | Read-only. |

## swap

| Entry Point | Auth Required | Notes |
|---|---|---|
| `propose_swap` | `party_a` | |
| `accept_swap` | `party_b` | |
| `cancel_swap` | `party_a` | |
| `get_swap` / `swap_count` | NONE | Read-only. |

## timelock

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `release` | NONE | Permissionless once the target ledger passes; funds go to the fixed stored beneficiary. |
| `cancel` | `admin` | |
| `get_info` / `is_releasable` / `get_remaining_ledgers` | NONE | Read-only. |

## token

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `mint` | `admin` | |
| `batch_mint` | `admin` | |
| `admin_burn` | `admin` | |
| `propose_admin` | `admin` | |
| `accept_admin` | `pending` (proposed new admin) | |
| `cancel_admin_proposal` | `admin` | |
| `set_admin` | `old_admin` | |
| `pause` / `unpause` | `admin` | |
| `freeze_account` / `unfreeze_account` | `admin` | |
| `propose_upgrade` / `execute_upgrade` | `admin` | |
| `set_transfer_hook` | `admin` | |
| `snapshot` | `caller` | |
| `admin` / `total_supply` / `balance_of` / `version` / `contract_version` / `allowance_expiry` / `balance_at` / `max_supply` / `get_transfer_hook` | NONE | Read-only. |
| `approve` *(SEP-41 `TokenInterface`)* | `from` | |
| `transfer` *(SEP-41)* | `from` | |
| `transfer_from` *(SEP-41)* | `spender` | Must hold an active allowance. |
| `burn` *(SEP-41)* | `from` | |
| `burn_from` *(SEP-41)* | `spender` | Must hold an active allowance. |
| `allowance` / `balance` / `decimals` / `name` / `symbol` *(SEP-41)* | NONE | Read-only. |

## vesting

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `claim` | `beneficiary` | |
| `revoke` | `admin` | |
| `admin_release` | `admin` | |
| `get_info` / `claimable` / `contract_version` | NONE | Read-only. |

## wrapped-token

| Entry Point | Auth Required | Notes |
|---|---|---|
| `initialize` | `admin` | |
| `wrap` | `user` | |
| `unwrap` | `user` | |
| `get_total_wrapped` | NONE | Read-only. |

## common

`contracts/common` is a shared library crate (`crate-type = ["rlib"]`) with
no `#[contract]`/`#[contractimpl]` of its own — it has no entry points and
is not deployed independently, so it is out of scope for this table.
