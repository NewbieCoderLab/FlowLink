#!/usr/bin/env bash
set -euo pipefail

SECONDS_TO_RUN="${1:-3}"
PEER_IP="${2:-}"
SERVICE_PORT="${SERVICE_PORT:-42424}"
UDP_PORT="${UDP_PORT:-42425}"

cd "$(dirname "$0")/../src-tauri"
ARGS=(
  --seconds "$SECONDS_TO_RUN" \
  --service-port "$SERVICE_PORT" \
  --udp-port "$UDP_PORT"
)
if [[ -n "$PEER_IP" ]]; then
  ARGS+=(--peer-ip "$PEER_IP")
fi

cargo run --example discovery_smoke -- "${ARGS[@]}"
