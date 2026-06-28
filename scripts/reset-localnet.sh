#!/usr/bin/env bash
# reset-localnet.sh — one-command local network teardown and restart
# Stops, removes, and restarts the local Stellar/Soroban node container,
# then waits for it to report healthy before returning.
# Usage: ./scripts/reset-localnet.sh [max-wait-seconds]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE="docker compose -f $ROOT/docker/docker-compose.yml"
SERVICE="stellar-node"
MAX_WAIT="${1:-180}"
INTERVAL=3

echo "Stopping local Stellar node..."
$COMPOSE stop "$SERVICE"

echo "Removing local Stellar node container (chain data will be lost)..."
$COMPOSE rm -f "$SERVICE"

echo "Starting local Stellar node..."
$COMPOSE up -d "$SERVICE"

CONTAINER_ID="$($COMPOSE ps -q "$SERVICE")"
if [[ -z "$CONTAINER_ID" ]]; then
  echo "ERROR: could not find container for service '$SERVICE'" >&2
  exit 1
fi

echo "Waiting for node health check (timeout: ${MAX_WAIT}s) ..."
elapsed=0
while true; do
  status="$(docker inspect --format='{{.State.Health.Status}}' "$CONTAINER_ID" 2>/dev/null || echo "unknown")"
  if [[ "$status" == "healthy" ]]; then
    echo "Local node ready (${elapsed}s)."
    echo "  RPC:     http://localhost:${LOCAL_RPC_PORT:-8000}"
    echo "  Horizon: http://localhost:${LOCAL_HORIZON_PORT:-8001}"
    exit 0
  fi
  if (( elapsed >= MAX_WAIT )); then
    echo "ERROR: node did not become healthy within ${MAX_WAIT}s (last status: ${status})" >&2
    echo "  - Inspect logs: ./scripts/local-net.sh logs" >&2
    exit 1
  fi
  echo "  Not healthy yet (${elapsed}s elapsed, status: ${status}), retrying in ${INTERVAL}s ..."
  sleep "$INTERVAL"
  elapsed=$(( elapsed + INTERVAL ))
done
