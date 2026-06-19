#!/usr/bin/env bash
set -euo pipefail

FC_SOCKET="/tmp/node_a-fc.sock"
FC_SOCKET_PAT="${FC_SOCKET//./\\.}"

pkill -f "firecracker.*$FC_SOCKET_PAT" || true
timeout 10 bash -c "while pkill -0 -f 'firecracker.*$FC_SOCKET_PAT' 2>/dev/null; do sleep 0.1; done" || true
rm -f "$FC_SOCKET"
echo "firecracker stopped"
