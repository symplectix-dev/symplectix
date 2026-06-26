#!/usr/bin/env bash
# Stop a Firecracker VM.
#
# Reads fc_config.json from the current directory (BUILD_WORKING_DIRECTORY).
set -euo pipefail

cd "${BUILD_WORKING_DIRECTORY:-$PWD}"

CONFIG="fc_config.json"

FC_SOCKET=$(jq -r '.fc_socket' "$CONFIG")
FC_SOCKET_PAT="${FC_SOCKET//./\\.}"

sudo pkill -f "firecracker.*$FC_SOCKET_PAT" || true
timeout 10 bash -c "while sudo pkill -0 -f 'firecracker.*$FC_SOCKET_PAT' 2>/dev/null; do sleep 0.1; done" || true
sudo rm -f "$FC_SOCKET"
echo "firecracker stopped"
