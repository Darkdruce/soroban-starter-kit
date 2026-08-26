# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Soroban Starter Kit.

Each ADR documents a significant design decision: the context that motivated it, the decision taken, and its consequences.

For general reference documentation (development environment, deployment guides, error codes, etc.), see the parent [docs/](../) directory.

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-storage-tier-choices.md) | Storage Tier Choices | Accepted |
| [0002](0002-error-handling-strategy.md) | Error Handling Strategy | Accepted |
| [0003](0003-admin-model.md) | Admin Model | Accepted |
| [0004](0004-escrow-state-machine.md) | Escrow State Machine Design | Accepted |
| [0005](0005-feature-flags.md) | Feature Flag Design | Accepted |
| [0006](0006-escrow-arbiter-model.md) | Escrow Arbiter Model | Accepted |
| [0007](0007-token-interface-compliance.md) | Token Interface Compliance | Accepted |
| [0008](0008-multisig-arbiter-design.md) | Multi-sig Arbiter Design | Accepted |
| [0009](0009-batch-mint-design.md) | Batch Mint Design — Atomicity & Cap Enforcement | Accepted |
| [0010](0010-no-serde-wasm-targets.md) | No serde — use soroban-sdk native types only | Accepted |

## Format

Each ADR follows this structure:

- **Status** — `Proposed` | `Accepted` | `Deprecated` | `Superseded by ADR-XXXX`
- **Date** — date the decision was accepted
- **Context** — the problem and forces at play
- **Decision** — what was decided
- **Consequences** — trade-offs and follow-on implications
