#!/usr/bin/env bash
set -euo pipefail

echo "Checking Rust formatting..."
if ! cargo fmt --workspace --check; then
    echo ""
    echo "ERROR: Some files are not formatted. Run 'cargo fmt --workspace' to fix."
    exit 1
fi

echo "All files are rustfmt-clean."
