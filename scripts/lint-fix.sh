#!/usr/bin/env bash
# Auto-fix formatting and common Clippy issues across the workspace.
set -euo pipefail

echo "Running cargo fmt --all..."
cargo fmt --all

echo "Running cargo clippy --fix..."
cargo clippy --workspace --fix --allow-dirty --allow-staged

echo "Done. Review the changes with 'git diff' before committing."
