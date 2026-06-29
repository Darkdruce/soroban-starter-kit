# ADR-0010: No serde — use soroban-sdk native types only

**Status:** Accepted  
**Date:** 2026-06-29  
**Issue:** #702

---

## Context

`serde` and its derive macros add measurable compile-time cost and pull in the
`serde_derive` proc-macro crate, which is incompatible with `#![no_std]` WASM
targets without additional feature flags (`serde/alloc` or `serde/std`).

The question was raised: does this project actually use `serde`, and if so,
should it be replaced with a lighter alternative?

### Audit results

A full-text search across all `*.rs` source files and `Cargo.toml` manifests
found **zero occurrences** of `serde` in contract or test code.  The only
reference to `serde` in the repository is in `deny.toml`, where it appears as a
comment explaining a transitive dependency version conflict inside
`soroban-sdk`'s own macro crate (`serde_with_macros`). That transitive
dependency is pulled in by `soroban-sdk-macros`, not by this project.

### Why serde is not needed here

All on-chain data in Soroban is stored and transmitted as XDR-encoded
`soroban_sdk` types (`Address`, `Symbol`, `Map`, `Vec`, `Bytes`, `i128`,
`u32`, …). Serialisation is handled entirely by the host environment via the
`soroban-sdk` derive macros (`#[contracttype]`, `#[contracterror]`). There is
no JSON, CBOR, or custom binary format required in any contract.

---

## Decision

**Do not add `serde` to any contract crate.**  All serialisation uses
`soroban-sdk` native types and the `#[contracttype]` / `#[contracterror]`
macros provided by the SDK.

---

## Consequences

- **Compile time**: no change required; `serde` was never a direct dependency
  and is not present in any `[dependencies]` section.
- **WASM binary size**: unaffected; serde code was never compiled into contract
  WASM.
- **Future work**: if off-chain tooling (TypeScript bindings, indexers) needs
  JSON (de)serialisation of contract types, it should be done in the off-chain
  layer using `@stellar/stellar-sdk` type conversion helpers, **not** by adding
  `serde` to the contract crates.
- **Deny list**: the `deny.toml` version-conflict comment for
  `serde_with_macros` / `darling` is a transitive-dependency artefact of
  `soroban-sdk-macros` and requires no action from this project.
