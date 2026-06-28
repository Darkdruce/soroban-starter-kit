.DEFAULT_GOAL := help

.PHONY: help build test test-all-features clippy fmt fmt-check watch deploy-testnet deploy-local bench clean

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build all contracts
	cargo build --release

test: ## Run unit tests
	cargo test

test-all-features: ## Run unit tests with all features enabled
	cargo test --all-features

clippy: ## Run Clippy lints
	cargo clippy --all-targets --all-features -- -D warnings

fmt: ## Format source code
	cargo fmt --all

fmt-check: ## Verify all Rust files are rustfmt-clean (exits non-zero on failure)
	./scripts/format-check.sh

watch: ## Re-run tests on any src/ change (requires cargo-watch)
	cargo watch -w src -x test

deploy-testnet: ## Deploy contracts to testnet
	./scripts/deploy.sh testnet

deploy-local: ## Deploy contracts to local node
	./scripts/deploy.sh local

bench: ## Run benchmarks
	cargo bench

clean: ## Remove build artifacts
	cargo clean
