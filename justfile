# List available recipes
default:
    @just --list

# Build all contracts
build:
    cargo build --release

# Run unit tests
test:
    cargo test

# Run unit tests with all features enabled
test-all-features:
    cargo test --all-features

# Run unit tests via cargo-nextest (requires cargo-nextest)
test-nextest:
    cargo nextest run --workspace

# Run Clippy lints
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Format source code
fmt:
    cargo fmt --all

# Verify all Rust files are rustfmt-clean (exits non-zero on failure)
fmt-check:
    ./scripts/format-check.sh

# Auto-fix formatting and common Clippy issues
lint-fix:
    ./scripts/lint-fix.sh

# Re-run tests on any src/ change (requires cargo-watch)
watch:
    cargo watch -w src -x test

# Install git pre-commit hook (format-check + clippy)
install-hooks:
    ./scripts/install-hooks.sh

# Deploy contracts to testnet
deploy-testnet:
    ./scripts/deploy.sh testnet

# Deploy contracts to local node
deploy-local:
    ./scripts/deploy.sh local

# Run benchmarks
bench:
    cargo bench

# Remove build artifacts
clean:
    cargo clean
