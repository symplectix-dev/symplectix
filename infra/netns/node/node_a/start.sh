#!/usr/bin/env bash
set -euo pipefail

NODE="node_a"
FC_SOCKET="/tmp/node_a-fc.sock"
FC_CONFIG="$(dirname "$0")/firecracker_config.json"

if [[ ! -f "$FC_CONFIG" ]]; then
  echo "error: not found: $FC_CONFIG" >&2
  exit 1
fi

FC_SOCKET_PAT="${FC_SOCKET//./\\.}"
if pkill -0 -f "firecracker.*$FC_SOCKET_PAT" 2>/dev/null; then
  echo "firecracker is already running (socket=$FC_SOCKET)"
  exit 0
fi
FC_LOG="/data/firecracker/node_a/fc.log"
rm -f "$FC_SOCKET"
ip netns exec "$NODE" /usr/local/bin/firecracker \
  --api-sock "$FC_SOCKET" \
  --config-file "$FC_CONFIG" >> "$FC_LOG" 2>&1 &
FC_PID=$!
for i in $(seq 1 30); do
  [[ -S "$FC_SOCKET" ]] && break
  if ! kill -0 "$FC_PID" 2>/dev/null; then
    rm -f "$FC_SOCKET"
    echo "error: firecracker exited, check $FC_LOG" >&2
    exit 1
  fi
  sleep 0.5
done
if [[ ! -S "$FC_SOCKET" ]]; then
  kill "$FC_PID" 2>/dev/null
  rm -f "$FC_SOCKET"
  echo "error: firecracker socket not created after 15s, check $FC_LOG" >&2
  exit 1
fi
echo "firecracker started in $NODE (pid=$FC_PID, socket=$FC_SOCKET)"
