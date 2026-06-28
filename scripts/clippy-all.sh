#!/usr/bin/env bash
# Run cargo clippy across every contract with all relevant feature combinations.
# Surfaces warnings hidden behind conditional compilation flags.
set -euo pipefail

CONTRACTS_DIR="$(cd "$(dirname "$0")/../contracts" && pwd)"
STATUS=0

has_feature() {
    grep -qE "^$1\s*=\s*\[\]" "$2"
}

run_clippy() {
    local manifest="$1"
    local label="$2"
    shift 2
    printf '\n==> %s [%s]\n' "$(basename "$(dirname "$manifest")")" "$label"
    if ! cargo clippy --manifest-path "$manifest" "$@" -- -D warnings; then
        STATUS=1
    fi
}

for contract_dir in "$CONTRACTS_DIR"/*/; do
    manifest="$contract_dir/Cargo.toml"
    [ -f "$manifest" ] || continue

    run_clippy "$manifest" "default"

    if has_feature "pausable" "$manifest"; then
        run_clippy "$manifest" "pausable" --features pausable
    fi

    if has_feature "upgradeable" "$manifest"; then
        run_clippy "$manifest" "upgradeable" --features upgradeable
    fi

    if has_feature "pausable" "$manifest" && has_feature "upgradeable" "$manifest"; then
        run_clippy "$manifest" "pausable+upgradeable" --features pausable,upgradeable
    fi
done

if [ "$STATUS" -eq 0 ]; then
    echo -e '\nAll clippy checks passed with zero warnings.'
else
    echo -e '\nOne or more clippy checks failed.' >&2
fi

exit "$STATUS"
