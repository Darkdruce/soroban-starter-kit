# Directory Tree

```
soroban-starter-kit/
├── contracts/          # All Soroban smart contract templates (one crate per contract)
│   ├── common/         # Shared utilities and the new-contract scaffold skeleton
│   ├── token/          # Fungible token with mint/burn/admin/allowance
│   ├── escrow/         # Two-party escrow with deadline and arbiter
│   ├── vesting/        # Cliff + linear token vesting with revocation
│   ├── staking/        # Token staking with proportional reward distribution
│   ├── multisig/       # N-of-M threshold wallet
│   ├── subscription/   # Recurring pull-payment contract
│   ├── timelock/       # Time-locked token release to a beneficiary
│   ├── nft/            # Non-fungible token with optional supply cap
│   ├── dao/            # On-chain governance with token-weighted voting
│   ├── swap/           # Atomic two-party token swap with deadline
│   ├── oracle/         # Price oracle with staleness validation
│   ├── lottery/        # Commit-reveal lottery with verifiable winner selection
│   ├── auction/        # Sealed-bid or English auction contract
│   ├── airdrop/        # Merkle-proof airdrop distribution
│   ├── ballot/         # Simple on-chain ballot / poll
│   ├── bonding-curve/  # Token sale along a bonding curve
│   ├── crowdfund/      # Goal-based crowdfunding with refund on failure
│   ├── marketplace/    # NFT or token marketplace with listing/purchase
│   └── wrapped-token/  # Wrapped asset (deposit/withdraw adapter)
├── docs/               # Project documentation (guides, ADRs, API reference)
│   └── adr/            # Architecture Decision Records
├── examples/           # End-to-end usage examples
│   ├── typescript/     # Node.js script exercising the token + escrow lifecycle
│   └── shell/          # Equivalent shell script using the Stellar CLI
├── scripts/            # Build, deploy, setup, and monitoring shell scripts
├── tests/              # Cross-contract integration tests (separate Cargo crate)
├── benches/            # Criterion benchmarks for hot contract paths
├── fuzz/               # cargo-fuzz targets for property-based fuzzing
├── docker/             # Dockerfiles and docker-compose for a local Stellar node
├── infra/              # Terraform modules for testnet/mainnet infrastructure
├── .devcontainer/      # Dev Container / Codespaces configuration
├── .github/            # CI workflows, issue templates, and Dependabot config
├── .githooks/          # Local git hooks (pre-commit checks)
└── .vscode/            # VS Code workspace settings and recommended extensions
```
