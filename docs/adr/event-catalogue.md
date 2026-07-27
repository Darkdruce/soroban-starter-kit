# Event Catalogue

This document lists every event emitted by every contract in the Soroban Starter Kit. For each event, the table shows the event symbol, topic types, data type, and when it is fired.

## Event Publishing Format

In Soroban, events are published as:

```rust
env.events().publish((topic_1, topic_2, ...), data);
```

- **Topics** are indexed fields used for filtering and queries. Each event has 0–3 topics.
- **Data** is the unindexed payload containing details about the event.

---

## Token Contract

| Event | Symbol | Topics | Data Type | When Fired |
|-------|--------|--------|-----------|-----------|
| Initialized | `initialized` | `(Symbol, Address)` → event name, admin | `(String, String, u32)` → name, symbol, decimals | `initialize()` called |
| Minted | `mint` | `(Symbol, Address)` → event name, recipient | `i128` → amount minted | `mint()` called |
| Burned | `burn` | `(Symbol, Address)` → event name, account | `i128` → amount burned | `burn()` or `burn_from()` called |
| Admin Changed | `admin_changed` | `(Symbol, Address)` → event name, old admin | `Address` → new admin | `set_admin()` called |
| Admin Proposed | `admin_proposed` | `(Symbol, Address)` → event name, current admin | `Address` → pending admin | `propose_admin()` called |
| Admin Accepted | `admin_accepted` | `(Symbol, Address)` → event name, new admin | `()` | `accept_admin()` called |
| Admin Proposal Cancelled | `admin_proposal_cancelled` | `(Symbol, Address)` → event name, admin | `()` | `cancel_admin_transfer()` called |
| Approved | `approve` | `(Symbol, Address, Address)` → event name, owner, spender | `i128` → allowance amount | `approve()` called |
| Revoked | `revoke` | `(Symbol, Address, Address)` → event name, owner, spender | `()` | `approve()` called with amount 0 |
| Transferred | `transfer` | `(Symbol, Address, Address)` → event name, from, to | `i128` → amount transferred | `transfer()` or `transfer_from()` called |
| Paused | `paused` | `(Symbol, Address)` → event name, admin | `()` | `pause()` called (pausable feature) |
| Unpaused | `unpaused` | `(Symbol, Address)` → event name, admin | `()` | `unpause()` called (pausable feature) |
| Upgraded | `upgraded` | `(Symbol, Address)` → event name, admin | `BytesN<32>` → new WASM hash | `execute_upgrade()` called (upgradeable feature) |

---

## Escrow Contract

| Event | Symbol | Topics | Data Type | When Fired |
|-------|--------|--------|-----------|-----------|
| Initialized | `initialized` | `(Symbol, Address, Address, Address)` → event name, buyer, seller, arbiter | `i128` → amount | `initialize()` called |
| Escrow Created | `created` | `(Symbol, Address, Address)` → event name, buyer, seller | `i128` → amount | `create()` called |
| Escrow Funded | `funded` | `(Symbol, Address)` → event name, buyer | `i128` → amount funded | `fund()` called |
| Delivery Marked | `marked_delivered` | `(Symbol, Address)` → event name, seller | `()` | `mark_delivered()` called |
| Funds Released | `released` | `(Symbol, Address)` → event name, seller | `i128` → amount released | `release()` called |
| Partial Release | `released_partial` | `(Symbol, Address)` → event name, seller | `i128` → partial amount | `partial_release()` called |
| Funds Refunded | `refunded` | `(Symbol, Address)` → event name, buyer | `i128` → amount refunded | `refund()` called (deadline passed) |
| Paused | `paused` | `(Symbol, Address)` → event name, admin | `()` | `pause()` called |
| Unpaused | `unpaused` | `(Symbol, Address)` → event name, admin | `()` | `unpause()` called |
| Upgraded | `upgraded` | `(Symbol, Address)` → event name, admin | `BytesN<32>` → new WASM hash | `execute_upgrade()` called |

---

## Vesting Contract

| Event | Symbol | Topics | Data Type | When Fired |
|-------|--------|--------|-----------|-----------|
| Initialized | `initialized` | `(Symbol, Address)` → event name, beneficiary | `(i128, u32, u32)` → amount, cliff ledger, end ledger | `initialize()` called |
| Claimed | `claimed` | `(Symbol, Address)` → event name, beneficiary | `i128` → amount claimed | `claim()` called |
| Revoked | `revoked` | `(Symbol, Address)` → event name, admin | `i128` → amount returned to admin | `revoke()` called |

---

## Staking Contract

| Event | Symbol | Topics | Data Type | When Fired |
|-------|--------|--------|-----------|-----------|
| Initialized | `initialized` | `(Symbol, Address)` → event name, admin | `(Address, Address)` → stake token, reward token | `initialize()` called |
| Staked | `staked` | `(Symbol, Address)` → event name, staker | `(i128, i128)` → amount staked, new total | `stake()` called |
| Unstaked | `unstaked` | `(Symbol, Address)` → event name, staker | `(i128, i128)` → amount unstaked, remaining stake | `unstake()` called |
| Rewards Claimed | `claimed_rewards` | `(Symbol, Address)` → event name, staker | `i128` → reward amount claimed | `claim_rewards()` called |
| Rewards Added | `added_rewards` | `(Symbol, Address)` → event name, admin | `(i128, i128)` → reward amount, new total | `add_rewards()` called |

---

## Multisig Contract

| Event | Symbol | Topics | Data Type | When Fired |
|-------|--------|--------|-----------|-----------|
| Initialized | `initialized` | `(Symbol, u32)` → event name, threshold | `u32` → signer count | `initialize()` called |
| Signer Added | `added` | `(Symbol, Address)` → event name, signer | `u32` → new threshold | `add_signer()` called |
| Signer Removed | `removed` | `(Symbol, Address)` → event name, signer | `u32` → new threshold | `remove_signer()` called |
| Transaction Proposed | `proposed` | `(Symbol, Address)` → event name, proposer | `u64` → transaction ID | `propose()` called |
| Transaction Signed | `signed` | `(Symbol, Address, u64)` → event name, signer, tx ID | `u32` → signature count | `sign()` called |
| Transaction Executed | `executed` | `(Symbol, u64)` → event name, tx ID | `()` | `execute()` called (threshold met) |

---

## DAO Contract

| Event | Symbol | Topics | Data Type | When Fired |
|-------|--------|--------|-----------|-----------|
| Initialized | `initialized` | `(Symbol, Address, Address)` → event name, admin, token | `i128` → quorum | `initialize()` called |
| Proposal Created | `created` | `(Symbol, Address)` → event name, proposer | `u32` → proposal ID | `create_proposal()` called |
| Voted | `voted` | `(Symbol, Address)` → event name, voter | `(u32, bool, i128)` → proposal ID, support, voting weight | `vote()` called |
| Proposal Executed | `executed` | `(Symbol,)` → event name | `u32` → proposal ID | `execute()` called (quorum + majority met) |
| Proposal Cancelled | `cancelled` | `(Symbol, Address)` → event name, admin | `u32` → proposal ID | `cancel_proposal()` called (admin) |

---

## Event Publishing Patterns

### Indexing Strategy

Topics are indexed for efficient querying:

- **First topic (always)**: Event symbol (e.g., `initialized`, `transfer`, `voted`)
- **Second topic**: Primary actor (e.g., address performing action: sender, staker, proposer)
- **Third topic**: Secondary context (e.g., recipient, spender, or secondary party)

### Data Type Conventions

- Use **single values** if one piece of information: `i128`, `Address`, `u32`
- Use **tuples** for multi-value payloads: `(i128, u32)` for amount and ledger
- Use `()` (unit type) if no data needed beyond topics

### TTL Management

Events are broadcast to the Soroban network but are **not** subject to TTL extension. They are archived according to the network's archival policies (typically ~1 year of history).

---

## Querying Events

### Stellar SDK Example (JavaScript)

```javascript
import * as StellarSDK from "@stellar/stellar-sdk";

const server = new StellarSDK.Server(rpcUrl);
const ledgers = await server.getLedgers()
  .eventFilter({
    contractId: "CAAAA...",
    topics: ["transfer", addr],
    type: "contract",
  })
  .call();

ledgers.records.forEach((event) => {
  console.log("Topic:", event.topic);
  console.log("Data:", event.value);
});
```

### Grep Example (Off-Chain Indexer)

```bash
# Search for all "transfer" events in a JSON event log
jq '.[] | select(.topic[0] == "transfer")' events.json
```

---

## Consistency with Source

This catalogue is generated from the event emission calls in each contract's `src/events.rs`:

- Token: `contracts/token/src/events.rs`
- Escrow: `contracts/escrow/src/events.rs`
- Vesting: `contracts/vesting/src/events.rs`
- Staking: `contracts/staking/src/events.rs`
- Multisig: `contracts/multisig/src/events.rs`
- DAO: `contracts/dao/src/events.rs`

To keep this catalogue in sync, verify against the source before each release. A CI lint check validates that event names and topic signatures match the published code.

---

## See Also

- [Architecture: Event Model](architecture.md#event-model)
- [Soroban Events Documentation](https://soroban.stellar.org/docs/learn/events)
- [Integration Guide: Event Streams](integration-guide.md)
