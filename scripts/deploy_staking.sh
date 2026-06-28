#!/bin/bash

# Soroban Staking Contract Deployment Script
# Usage: ./deploy_staking.sh [network]
# Example: ./deploy_staking.sh testnet

set -e

NETWORK=${1:-testnet}
CONTRACT_NAME="soroban-staking-template"

# log_json — emit one structured JSON log line per deploy event on stdout.
# Fields: timestamp (UTC ISO-8601), network, contract, contractId, txHash, status.
# Human-readable progress goes to stderr so stdout stays clean NDJSON for log
# aggregators. Usage: log_json <status> [contractId] [txHash]
log_json() {
  local status="$1" contract_id="${2:-}" tx_hash="${3:-}" ts
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf '{"timestamp":"%s","network":"%s","contract":"staking","contractId":"%s","txHash":"%s","status":"%s"}\n' \
    "$ts" "$NETWORK" "$contract_id" "$tx_hash" "$status"
}

echo "🚀 Deploying Staking Contract to $NETWORK..." >&2

# Build the contract
echo "📦 Building contract..." >&2
log_json building
stellar contract build --manifest-path contracts/staking/Cargo.toml >&2

# Deploy the contract
echo "🌐 Deploying to $NETWORK..." >&2
CONTRACT_ID=$(stellar contract deploy \
    --wasm contracts/staking/target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm \
    --network "$NETWORK")

echo "✅ Staking contract deployed!" >&2
echo "📋 Contract ID: $CONTRACT_ID" >&2
log_json deployed "$CONTRACT_ID"

# Save contract ID
echo "staking: $CONTRACT_ID" >> .contract-ids

# Example initialization (uncomment and fill in values to use)
# ADMIN_ADDRESS="G..."
# STAKE_TOKEN="C..."    # token users deposit to stake
# REWARD_TOKEN="C..."   # token distributed as rewards (can equal STAKE_TOKEN)

# stellar contract invoke \
#     --id "$CONTRACT_ID" \
#     --network "$NETWORK" \
#     --source "$ADMIN_ADDRESS" \
#     -- initialize \
#     --admin "$ADMIN_ADDRESS" \
#     --stake_token "$STAKE_TOKEN" \
#     --reward_token "$REWARD_TOKEN"

# Add rewards (admin only):
# REWARD_AMOUNT=1000000000  # in base units
# stellar contract invoke \
#     --id "$CONTRACT_ID" \
#     --network "$NETWORK" \
#     --source "$ADMIN_ADDRESS" \
#     -- add_rewards \
#     --amount "$REWARD_AMOUNT"

echo "🎉 Staking contract ready for use!" >&2
echo "📝 Save this Contract ID: $CONTRACT_ID" >&2
