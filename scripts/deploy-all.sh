#!/usr/bin/env bash
# deploy-all.sh — deploy every workspace contract in one command
# Usage: ./scripts/deploy-all.sh [testnet|mainnet|local] [--identity <name>]
#
# Reuses scripts/deploy.sh (build + deploy for a single contract) for each
# contract directory under contracts/, prints a contract -> deployed ID
# summary table, and stops immediately on the first failure.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="$ROOT/contracts"
DEPLOY_SCRIPT="$ROOT/scripts/deploy.sh"

NETWORK="${1:-testnet}"
shift || true

[[ -d "$CONTRACTS_DIR" ]] || { echo "Contracts directory not found: $CONTRACTS_DIR" >&2; exit 1; }

NAMES=()
for dir in "$CONTRACTS_DIR"/*/; do
  name="$(basename "$dir")"
  # "common" is a shared library crate (crate-type = rlib), not a deployable contract.
  [[ "$name" == "common" ]] && continue
  NAMES+=("$name")
done

STATUSES=()
IDS=()
FAILED_CONTRACT=""

echo "Deploying ${#NAMES[@]} contracts to $NETWORK..." >&2

for name in "${NAMES[@]}"; do
  echo "" >&2
  echo "═══ $name ═══" >&2
  if "$DEPLOY_SCRIPT" "$NETWORK" "$name" "$@" >&2; then
    id="$(grep "^$name: " "$ROOT/.contract-ids" 2>/dev/null | tail -1 | cut -d' ' -f2)"
    STATUSES+=("ok")
    IDS+=("${id:-unknown}")
  else
    STATUSES+=("FAILED")
    IDS+=("-")
    FAILED_CONTRACT="$name"
    echo "" >&2
    echo "ERROR: deploy failed for '$name' — stopping (fail-fast)." >&2
    break
  fi
done

echo "" >&2
echo "Deploy summary ($NETWORK):"
printf "%-20s %-8s %s\n" "CONTRACT" "STATUS" "CONTRACT ID"
printf "%-20s %-8s %s\n" "--------------------" "--------" "--------------------------------------------"
for i in "${!STATUSES[@]}"; do
  printf "%-20s %-8s %s\n" "${NAMES[$i]}" "${STATUSES[$i]}" "${IDS[$i]}"
done

if [[ -n "$FAILED_CONTRACT" ]]; then
  echo "" >&2
  echo "Deploy stopped after failure on '$FAILED_CONTRACT' (${#STATUSES[@]}/${#NAMES[@]} attempted)." >&2
  exit 1
fi
