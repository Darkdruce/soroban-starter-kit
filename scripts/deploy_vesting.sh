#!/bin/bash

# Soroban Vesting Contract Deployment Script
# Usage: ./deploy_vesting.sh [network]
# Example: ./deploy_vesting.sh testnet

set -e

NETWORK=${1:-testnet}
CONTRACT_NAME="soroban-vesting-template"

# log_json — emit one structured JSON log line per deploy event on stdout.
# Fields: timestamp (UTC ISO-8601), network, contract, contractId, txHash, status.
# Human-readable progress goes to stderr so stdout stays clean NDJSON for log
# aggregators. Usage: log_json <status> [contractId] [txHash]
log_json() {
  local status="$1" contract_id="${2:-}" tx_hash="${3:-}" ts
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf '{"timestamp":"%s","network":"%s","contract":"vesting","contractId":"%s","txHash":"%s","status":"%s"}\n' \
    "$ts" "$NETWORK" "$contract_id" "$tx_hash" "$status"
}

echo "🚀 Deploying Vesting Contract to $NETWORK..." >&2

# Build the contract
echo "📦 Building contract..." >&2
log_json building
stellar contract build --manifest-path contracts/vesting/Cargo.toml >&2

# Deploy the contract
echo "🌐 Deploying to $NETWORK..." >&2
CONTRACT_ID=$(stellar contract deploy \
    --wasm contracts/vesting/target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm \
    --network "$NETWORK")

echo "✅ Vesting contract deployed!" >&2
echo "📋 Contract ID: $CONTRACT_ID" >&2
log_json deployed "$CONTRACT_ID"

# Save contract ID
echo "vesting: $CONTRACT_ID" >> .contract-ids

# Example initialization (uncomment and fill in values to use)
# ADMIN_ADDRESS="G..."
# BENEFICIARY_ADDRESS="G..."
# TOKEN_CONTRACT="C..."
# AMOUNT=1000000000   # total tokens to vest (in stroops / base units)
# CLIFF_LEDGER=500000  # ledger sequence at which vesting begins
# END_LEDGER=600000    # ledger sequence at which all tokens are fully vested

# stellar contract invoke \
#     --id "$CONTRACT_ID" \
#     --network "$NETWORK" \
#     --source "$ADMIN_ADDRESS" \
#     -- initialize \
#     --admin "$ADMIN_ADDRESS" \
#     --beneficiary "$BENEFICIARY_ADDRESS" \
#     --token "$TOKEN_CONTRACT" \
#     --cliff_ledger "$CLIFF_LEDGER" \
#     --end_ledger "$END_LEDGER" \
#     --amount "$AMOUNT"

echo "🎉 Vesting contract ready for use!" >&2
echo "📝 Save this Contract ID: $CONTRACT_ID" >&2
