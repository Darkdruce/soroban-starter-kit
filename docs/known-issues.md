# Known Issues

This document tracks **accepted design tradeoffs** in the contract templates — decisions that are not bugs, but that anyone deploying or extending a template should understand before relying on it. Where a tradeoff was the subject of an explicit design decision, the entry links to the relevant [ADR](adr/README.md).

If you believe an entry below is actually a defect rather than an intentional tradeoff, please open an issue.

## Airdrop

- `set_root` can be called by the admin at any time with no "finalized" lock — a legitimate root rotation and a malicious root swap look identical on-chain. Consumers should treat root changes as a trusted-admin action.
- `claim_batch` has no upper bound on batch size; an oversized `entries` vector can exceed transaction resource limits before it exceeds any protocol-level cap.

## Auction

- `end()` is permissionless by design so that a stalled seller cannot block settlement.
- Once a bid has been placed, the seller cannot `cancel()` — this is an intentional irrevocability tradeoff that favors bidder protection over seller flexibility.

## Ballot

- `tally()` has no minimum-turnout/quorum requirement — a proposal with very low participation is still tallied as decisive.
- Voter registration is fully admin-gated with no on-chain whitelist proof; the deployer is trusted to register only legitimate voters.

## Bonding Curve

- `buy_cost` / `sell_proceeds` use an averaged-price linear approximation (`(price(old) + price(new)) / 2`) rather than the exact curve integral. This is a deliberate gas/complexity tradeoff that introduces small, curve-shape-dependent rounding versus a mathematically exact bonding curve.
- Mitigated by required `max_cost` / `min_proceeds` slippage bounds on every trade.

## Crowdfund

- `extend_deadline` is capped to a single use per campaign — an intentional anti-abuse limit rather than a general-purpose extension mechanism.
- Funding tiers / stretch goals reported by `get_info` are informational only; they do not gate `claim()`, which sweeps the entire pledged pool once the base goal is met.

## DAO

- Voting weight is read from the voter's **live** token balance at the moment of `vote()`, not a historical snapshot. This is a simpler model than snapshot-based voting but is vulnerable to flash-loan-style vote weighting; it is an accepted tradeoff for template simplicity, not a recommendation for production governance of high-value treasuries.
- `execute_proposal` only transitions proposal state to `Executed` — it does not invoke a target contract. On-chain vote outcomes must be carried out off-chain or via a separate execution layer.

## Escrow

- No global admin exists by design (see [ADR-0003: Admin Model](adr/0003-admin-model.md)) — buyer, seller, and arbiter each hold exactly the authority described in [docs/security.md](security.md#authorization), and no single key can override all three roles.
- Multi-arbiter dispute resolution uses on-chain vote accumulation rather than a threshold signature scheme, trading some on-chain storage/gas cost for auditability. See [ADR-0008: Multi-sig Arbiter Design](adr/0008-multisig-arbiter-design.md).
- Deadline extension requires both parties' authorization, so a non-responsive counterparty can permanently block an extension; this is intentional (neither party can unilaterally change the agreed deadline). See [ADR-0004: Escrow State Machine Design](adr/0004-escrow-state-machine.md) and [ADR-0006: Escrow Arbiter Model](adr/0006-escrow-arbiter-model.md).

## Lottery

- The commit-reveal randomness scheme relies on the admin honestly choosing `secret`/`salt` before committing. A motivated admin can grind many candidate values off-chain and commit only the one that yields a favorable outcome. The on-chain check only proves the *revealed* value matches the *committed* hash — it cannot prove the admin didn't pre-select a favorable secret. Deployers who need trustless randomness should source entropy from an external VRF rather than relying on this template's admin-supplied commit-reveal alone.
- There is no economic penalty for an admin who commits and never reveals; the only mitigation is the buyer-side refund path after `reveal_deadline`.

## Marketplace

- Listings depend on the seller having pre-approved the marketplace as an NFT spender. A stale or revoked approval, or a seller who transfers the NFT elsewhere after listing, causes `buy()` to fail atomically rather than losing funds — a griefing/availability concern, not a fund-safety one.
- `get_active_listings` pagination is capped at a fixed page size as a deliberate DoS mitigation, not an oversight.
- Royalty configuration is fixed at `initialize` with no update path, trading flexibility for a simpler, tamper-proof royalty guarantee.

## Multisig

- Once a proposal reaches its signature threshold, `execute_transaction` allows the multisig to invoke **any** target contract/function with attacker-or-signer-chosen arguments. There is no per-transaction spending cap or destination allow-list — the signer set is fully trusted, matching the general multisig trust model.
- The same threshold that approves ordinary transactions also approves signer-set changes (`add_signer` / `remove_signer`), so a colluding quorum can alter its own membership.
- Proposals carry an `expiry_ledger` to avoid stale proposals lingering indefinitely, an intentional bound on proposal lifetime.

## NFT

- The approval model is single-spender-per-token (`approve` / `transfer_from`); there is no `set_approval_for_all`-style operator model. Callers that need collection-wide operator approval must call `approve` per token.
- `token_id` is caller-supplied `u32` with no auto-increment; uniqueness is enforced on-chain (`TokenAlreadyMinted`) but ID *generation* and collision avoidance across mints is left to the minter.
- `batch_transfer` (see [ADR-0009: Batch Mint Design](adr/0009-batch-mint-design.md) for the precedent this follows) validates ownership of every token before applying any transfer, matching the same validate-all-then-apply pattern used by the token contract's `batch_mint`.

## Oracle

- `get_price` / `update_price` is a single-admin-trusted feed by default. The multi-publisher `get_median_price` path exists as an opt-in alternative — nothing prevents a consumer from calling the weaker single-source `get_price` instead. Contract authors that need aggregation must deliberately choose the median path.
- `set_publishers` fully replaces the publisher set with no timelock, so publisher-set changes take effect immediately.
- Aggregation math (`median`, `twap`) uses saturating arithmetic, which silently clamps rather than erroring on extreme inputs, trading a hard failure for availability.

## Staking

- Reward accounting uses an integer-scaled `reward_per_token` accumulator; very small reward deposits relative to `total_staked` can lose dust to integer-division rounding. This is the standard tradeoff of the accumulator pattern versus per-staker floating point.
- `compound()` requires `stake_token == reward_token`; dual-asset pools cannot auto-compound by design.
- There is no lock-up period or slashing — `unstake` is available immediately, which is intentional for a general-purpose staking template rather than a bonded/slashed validator model.

## Subscription

- `charge()` is bounded only by interval spacing and the subscriber's remaining token allowance, not by an on-chain max-charge count or subscription-length field. Subscribers are expected to size their allowance as the primary spending guardrail.
- `cancel()` flips the subscription to inactive but does not revoke the underlying token allowance; subscribers who want a hard stop should also reduce their token approval separately.

## Swap

- `cancel_swap` after the deadline is intentionally permissionless — anyone may trigger it to return party A's funds, guaranteeing a stalled counterparty can't strand deposited tokens. This means a third party can trigger a state change on a swap they are not part of, though with no effect beyond returning funds to their rightful owner.
- The contract supports only fixed-amount, all-or-nothing swaps; there is no partial-fill or price-improvement logic.

## Timelock

- `cancel()` is available to the admin at any point before `release_ledger`, with no beneficiary consent required — a single admin key fully controls whether the beneficiary ever receives funds. This is an accepted single-party-trust design for a simple timelock; contrast with [Vesting](#vesting), which restricts revocation to the unvested remainder only.
- `release()` is permissionless once the release ledger is reached, so the beneficiary is never dependent on a specific relayer being online.

## Token

- `set_admin` (one-step, immediate) and `propose_admin` / `accept_admin` (two-step, confirmed) both remain available side by side. See [ADR-0003: Admin Model](adr/0003-admin-model.md). Integrators should prefer the two-step path and audit any UI that could accidentally call `set_admin`.
- `batch_mint` validates every recipient/amount against the supply cap before minting any of them (see [ADR-0009: Batch Mint Design](adr/0009-batch-mint-design.md)) — an intentional all-or-nothing tradeoff over partial-success batching.
- The `transfer-hook` feature invokes the configured hook with `try_invoke_contract`, so a failing or malicious hook cannot block transfers — but it also means a hook cannot be relied on as an enforcement point, only an observation point. See [ADR-0005: Feature Flag Design](adr/0005-feature-flags.md).
- The upgrade delay (`upgradeable` feature) is a fixed constant rather than a configurable parameter, trading deployment flexibility for a predictable, auditable timelock.

## Vesting

- `admin_release` is a documented "emergency unlock" escape hatch that lets the admin force-release the full balance early, bypassing the vesting schedule. It intentionally requires no beneficiary consent or additional timelock beyond the standard admin authorization, unlike more conservative vesting designs.
- `vested_amount` uses integer division for linear vesting, which truncates fractional token amounts; the resulting dust remains in the contract and is only recoverable via `revoke` or `admin_release`, not swept separately.

## Wrapped Token

- The 1:1 peg is not reconciled against the underlying token's actual balance on-chain; `get_total_wrapped` is a running counter that assumes `wrap` / `unwrap` are the only paths that mint or burn the wrapped asset. Deploying this template against a fee-on-transfer or rebasing underlying token will silently break the peg — it is designed for standard, non-rebasing Soroban tokens only.
- There is no pause or circuit-breaker if the underlying asset contract misbehaves.

## See Also

- [Threat Model](threat-model.md) — trust assumptions and worst-case-per-role analysis
- [Security Best Practices](security.md)
- [Architecture Decision Records](adr/README.md)
