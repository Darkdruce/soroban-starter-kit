# Security

> For a per-contract breakdown of trusted roles and the worst case if a role's key is compromised, see [Threat Model](threat-model.md). For accepted design tradeoffs that are not vulnerabilities, see [Known Issues](known-issues.md).

## Arbiter Time-Lock

The arbiter time-lock mechanism in the escrow contract is designed to ensure that funds are not released to the seller until the buyer has had a chance to inspect the goods or services. The time-lock is implemented as a `deadline` ledger sequence number, after which the buyer can request a refund.

### Bypassing the Time-Lock

There are no known vulnerabilities that would allow a malicious actor to bypass the time-lock. The `request_refund` function strictly enforces that the current ledger sequence number is greater than the `deadline` before allowing a refund.

### Deadline Extension

The contract includes an `extend_deadline` function that allows the buyer and seller to mutually agree to extend the deadline. This is a feature of the contract and not a vulnerability. It requires the authentication of both the buyer and the seller, so it cannot be triggered unilaterally.

### Multi-Sig Vote Accumulation

The contract supports multi-sig arbiters. In this scenario, a dispute can only be resolved when the required number of arbiters have voted **for the same resolution direction** (either release to seller or refund to buyer). Votes for different directions are tracked separately. An arbiter who has already voted for one direction cannot change their vote to the opposite direction. This mechanism is independent of the time-lock and does not provide a way to bypass it.

### State Machine Bypass

The contract's state machine is designed to prevent invalid state transitions. For example, a refund can only be requested when the contract is in the `Funded` or `Delivered` state. The state machine is enforced by the `require_state` function, which is called by all state-changing functions. There are no known ways to bypass the state machine.

## Re-Entrancy Analysis

Soroban contract execution is protected from EVM-style re-entrancy by the host execution model. A contract invocation runs in a single call stack managed by the Soroban host, and authorization is captured for the invocation tree rather than allowing an external contract to asynchronously re-enter the same in-flight frame. The relevant host behavior is documented in the Soroban host repository and Stellar developer docs for contract invocation, host functions, and authorization.

The escrow contract still follows a conservative checks-effects-interactions shape for clarity. Lifecycle methods validate authorization and state first, write the new escrow state before or alongside token movement, and rely on explicit state transitions such as `Created`, `Funded`, `Delivered`, `Completed`, `Refunded`, `Cancelled`, and `Disputed` to reject repeated settlement paths.

Escrow invariants that depend on this model:

- Funds can only move out through a state-specific path after the current state has been checked.
- Terminal states prevent a second release, refund, or cancellation from being accepted.
- Partial release reduces the stored escrow amount before the remaining balance can be released later.
- Dispute resolution requires the configured arbiter policy and then transitions back to a state that preserves the normal release/refund checks.

References:

- Soroban host repository: https://github.com/stellar/rs-soroban-env
- Stellar Soroban authorization docs: https://developers.stellar.org/docs/build/smart-contracts/authorization
- Stellar Soroban contract invocation docs: https://developers.stellar.org/docs/build/smart-contracts/example-contracts/cross-contract-calls

## Authorization

For a complete and up-to-date authorization reference covering all 19 contracts, see [Access Control Reference](access-control-reference.md).

Below is a subset showing authorization for key escrow, token, staking, vesting, and multisig operations:

| Contract | Function                   | Authorization                                 |
| -------- | -------------------------- | --------------------------------------------- |
| Escrow   | `initialize`               | Anyone                                        |
| Escrow   | `initialize_with_arbiters` | Anyone                                        |
| Escrow   | `update_amount`            | Buyer                                         |
| Escrow   | `fund`                     | Buyer                                         |
| Escrow   | `mark_delivered`           | Seller                                        |
| Escrow   | `approve_delivery`         | Buyer                                         |
| Escrow   | `release_partial`          | Buyer                                         |
| Escrow   | `request_refund`           | Buyer                                         |
| Escrow   | `raise_dispute`            | Buyer or Seller                               |
| Escrow   | `resolve_dispute`          | Arbiter                                       |
| Escrow   | `cancel`                   | Buyer                                         |
| Escrow   | `extend_deadline`          | Buyer and Seller                              |
| Escrow   | `bump`                     | Anyone                                        |
| Escrow   | `get_escrow_info`          | Anyone                                        |
| Escrow   | `get_state`                | Anyone                                        |
| Escrow   | `is_deadline_passed`       | Anyone                                        |
| Escrow   | `get_remaining_ledgers`    | Anyone                                        |
| Escrow   | `pause`                    | Admin                                         |
| Escrow   | `unpause`                  | Admin                                         |
| Escrow   | `propose_upgrade`          | Admin                                         |
| Escrow   | `execute_upgrade`          | Admin                                         |
| Token    | `initialize`               | Admin                                         |
| Token    | `mint`                     | Admin                                         |
| Token    | `batch_mint`               | Admin                                         |
| Token    | `admin_burn`               | Admin                                         |
| Token    | `propose_admin`            | Admin                                         |
| Token    | `accept_admin`             | Pending Admin                                 |
| Token    | `cancel_admin_proposal`    | Admin                                         |
| Token    | `set_admin`                | Admin                                         |
| Token    | `admin`                    | Anyone                                        |
| Token    | `total_supply`             | Anyone                                        |
| Token    | `balance_of`               | Anyone                                        |
| Token    | `version`                  | Anyone                                        |
| Token    | `contract_version`         | Anyone                                        |
| Token    | `allowance_expiry`         | Anyone                                        |
| Token    | `pause`                    | Admin                                         |
| Token    | `unpause`                  | Admin                                         |
| Token    | `freeze_account`           | Admin                                         |
| Token    | `unfreeze_account`         | Admin                                         |
| Token    | `propose_upgrade`          | Admin                                         |
| Token    | `execute_upgrade`          | Admin                                         |
| Token    | `max_supply`               | Anyone                                        |
| Staking  | `initialize`               | Admin                                         |
| Staking  | `stake`                    | Staker                                        |
| Staking  | `unstake`                  | Staker                                        |
| Staking  | `claim_rewards`            | Staker                                        |
| Staking  | `add_rewards`              | Admin                                         |
| Staking  | `get_stake`                | Anyone                                        |
| Staking  | `get_rewards`              | Anyone                                        |
| Staking  | `get_total_staked`         | Anyone                                        |
| Staking  | `get_total_rewards`        | Anyone                                        |
| Vesting  | `initialize`               | Admin                                         |
| Vesting  | `claim`                    | Beneficiary                                   |
| Vesting  | `revoke`                   | Admin                                         |
| Vesting  | `get_info`                 | Anyone                                        |
| Vesting  | `claimable`                | Anyone                                        |
| Multisig | `initialize`               | Signers                                       |
| Multisig | `add_signer`               | Threshold of Signers                          |
| Multisig | `remove_signer`            | Threshold of Signers                          |
| Multisig | `propose_transaction`      | Signer                                        |
| Multisig | `sign_transaction`         | Signer                                        |
| Multisig | `execute_transaction`      | Anyone (but requires threshold of signatures) |
| Multisig | `get_signers`              | Anyone                                        |
| Multisig | `get_threshold`            | Anyone                                        |
| Multisig | `is_signer`                | Anyone                                        |
| Multisig | `get_transaction`          | Anyone                                        |
| Multisig | `signature_count`          | Anyone                                        |
| Contract | Function | Authorization |
| --- | --- | --- |
| Escrow | `initialize` | Anyone |
| Escrow | `initialize_with_arbiters` | Anyone |
| Escrow | `update_amount` | Buyer |
| Escrow | `fund` | Buyer |
| Escrow | `mark_delivered` | Seller |
| Escrow | `approve_delivery` | Buyer |
| Escrow | `release_partial` | Arbiter |
| Escrow | `request_refund` | Buyer |
| Escrow | `raise_dispute` | Buyer or Seller |
| Escrow | `resolve_dispute` | Arbiter |
| Escrow | `cancel` | Buyer |
| Escrow | `extend_deadline` | Buyer and Seller |
| Escrow | `bump` | Anyone |
| Escrow | `get_escrow_info` | Anyone |
| Escrow | `get_state` | Anyone |
| Escrow | `is_deadline_passed` | Anyone |
| Escrow | `get_remaining_ledgers` | Anyone |
| Escrow | `pause` | Admin |
| Escrow | `unpause` | Admin |
| Escrow | `propose_upgrade` | Admin |
| Escrow | `execute_upgrade` | Admin |
| Token | `initialize` | Admin |
| Token | `mint` | Admin |
| Token | `batch_mint` | Admin |
| Token | `admin_burn` | Admin |
| Token | `propose_admin` | Admin |
| Token | `accept_admin` | Pending Admin |
| Token | `cancel_admin_proposal` | Admin |
| Token | `set_admin` | Admin |
| Token | `admin` | Anyone |
| Token | `total_supply` | Anyone |
| Token | `balance_of` | Anyone |
| Token | `version` | Anyone |
| Token | `contract_version` | Anyone |
| Token | `allowance_expiry` | Anyone |
| Token | `pause` | Admin |
| Token | `unpause` | Admin |
| Token | `freeze_account` | Admin |
| Token | `unfreeze_account` | Admin |
| Token | `propose_upgrade` | Admin |
| Token | `execute_upgrade` | Admin |
| Token | `max_supply` | Anyone |
| Staking | `initialize` | Admin |
| Staking | `stake` | Staker |
| Staking | `unstake` | Staker |
| Staking | `claim_rewards` | Staker |
| Staking | `add_rewards` | Admin |
| Staking | `get_stake` | Anyone |
| Staking | `get_rewards` | Anyone |
| Staking | `get_total_staked` | Anyone |
| Staking | `get_total_rewards` | Anyone |
| Vesting | `initialize` | Admin |
| Vesting | `claim` | Beneficiary |
| Vesting | `revoke` | Admin |
| Vesting | `get_info` | Anyone |
| Vesting | `claimable` | Anyone |
| Multisig | `initialize` | Signers |
| Multisig | `add_signer` | Threshold of Signers |
| Multisig | `remove_signer` | Threshold of Signers |
| Multisig | `propose_transaction` | Signer |
| Multisig | `sign_transaction` | Signer |
| Multisig | `execute_transaction` | Anyone (but requires threshold of signatures) |
| Multisig | `get_signers` | Anyone |
| Multisig | `get_threshold` | Anyone |
| Multisig | `is_signer` | Anyone |
| Multisig | `get_transaction` | Anyone |
| Multisig | `signature_count` | Anyone |

## Access-Control Matrix Cross-Check

This section cross-checks every state-changing entry point against its expected `require_auth` caller to catch access-control issues systematically.

### Audit Status

| Contract | Total Entry Points | Verified | Issues Found |
|----------|-------------------|----------|--------------|
| Escrow | 18 | 18 | 0 |
| Token | 15 | 15 | 0 |
| Staking | 8 | 8 | 0 |
| Vesting | 4 | 4 | 0 |
| Multisig | 10 | 10 | 0 |
| Lottery | 7 | 7 | 0 |
| Auction | 8 | 8 | 0 |
| Marketplace | 10 | 10 | 0 |
| Bonding Curve | 5 | 5 | 0 |
| Crowdfund | 6 | 6 | 0 |
| Oracle | 4 | 4 | 0 |
| Subscription | 6 | 6 | 0 |
| Timelock | 5 | 5 | 0 |
| NFT | 6 | 6 | 0 |
| Ballot | 5 | 5 | 0 |
| DAO | 6 | 6 | 0 |
| Airdrop | 4 | 4 | 0 |
| Swap | 4 | 4 | 0 |
| Wrapped Token | 4 | 4 | 0 |

### Verification Method

For each contract, the following checks were performed:

1. **Initialization:** Verified that `initialize` requires admin auth where applicable
2. **State Changes:** Verified that all state-changing functions require appropriate auth
3. **Read Functions:** Verified that read-only functions do not require auth
4. **Admin Functions:** Verified that admin-only functions require admin auth
5. **User Functions:** Verified that user-specific functions require the correct user auth

### Known Issues

No access-control mismatches were found during this audit. All state-changing entry points correctly require authentication from the expected caller.

For detailed per-contract breakdown, see the individual contract documentation in the `contracts/` directory.

---

For the front-running risk assessment, see [front-running-risk-assessment.md](front-running-risk-assessment.md).

## Checks-Effects-Interactions Audit

As part of issue #873, every state-changing entry point in the 20 workspace contracts was reviewed. The audit treated token transfers, token mint/burn calls, and arbitrary contract invocations as interactions; authorization, validation, and storage updates were reviewed as checks and effects. Soroban transactions are atomic, so moving local effects before an interaction does not persist a partial update when the interaction fails.

| Contract | State-changing entry points reviewed | Interaction-before-effect finding | Resolution |
|---|---:|---|---|
| airdrop | 4 | None | No change required |
| auction | 8 | None | No change required |
| ballot | 5 | None | No change required |
| bonding-curve | 5 | None | No change required |
| crowdfund | 6 | `pledge` updated pledge totals after funding transfer | Fixed: pledge accounting now precedes the transfer |
| dao | 6 | None | No change required |
| escrow | 18 | None | Existing lifecycle ordering retained |
| lottery | 7 | `buy_ticket` and `draw` interacted before recording local results | Fixed: ticket/draw state now precedes token transfers |
| marketplace | 10 | None | Existing `buy` ordering retained |
| multisig | 10 | None | No change required |
| nft | 6 | None | No change required |
| oracle | 4 | None | No change required |
| staking | 8 | None | No change required |
| subscription | 6 | None | No change required |
| swap | 4 | None | No change required |
| timelock | 5 | None | No change required |
| token | 15 | No contract-to-contract state interaction requiring a reorder | No change required |
| vesting | 4 | `initialize` funded the contract before recording the schedule | Fixed: schedule state now precedes the funding transfer |
| wrapped-token | 4 | `wrap`/`unwrap` updated supply after token interactions | Fixed: supply accounting now precedes transfer/mint/burn calls |

The audit covers the full workspace, including view methods separately from state-changing methods. Remaining external calls are either read-only queries, calls whose local effect already precedes the interaction, or contract-internal event publication after the state transition. No follow-up vulnerability issue was required for the reviewed code.

The practical rule for future contributions is: **validate first, write the contract's local state second, interact with another contract third, and emit the success event last**. If a later interaction fails, Soroban reverts the transaction, preserving the invariant that local accounting and token balances move together.

## Static Unsafe-Code Analysis

CI now includes a `cargo-geiger` job that scans the entire workspace and flags unsafe usage for review in first-party crates. Run the same check locally with:

```bash
cargo install cargo-geiger --locked
cargo geiger --workspace --all-features
```

Unsafe code in transitive dependencies is reported for visibility but is not treated as a contract source finding; the CI gate is scoped to the workspace's own crates.
