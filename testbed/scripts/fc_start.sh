#!/usr/bin/env bash
# Start a Firecracker VM inside a network namespace.
#
set -euo pipefail

cd "${BUILD_WORKING_DIRECTORY:-$PWD}"

CONFIG="fc_config.json"

jq_get() { jq -r ".$1" "$CONFIG"; }

NODE=$(jq_get node_name)
FC_SOCKET=$(jq_get fc_socket)
FC_DATA_DIR=$(jq_get fc_data_dir)
FC_LOG=$(jq_get fc_log)
FC_VCPU=$(jq_get fc_vcpu)
FC_MEMORY=$(jq_get fc_memory)
FC_MAC=$(jq_get fc_mac)
TAP_DEV=$(jq_get tap_dev)
FC_IPV4=$(jq_get fc_ipv4)
PREFIX4=$(jq_get prefix4)
FC_IPV6=$(jq_get fc_ipv6)
PREFIX6=$(jq_get prefix6)
TAP_IPV4=$(jq_get tap_ipv4)
TAP_IPV6=$(jq_get tap_ipv6)
SSH_KEY_FILE="$FC_DATA_DIR/key/testkey.pub"

FC_SOCKET_PAT="${FC_SOCKET//./\\.}"
if sudo pkill -0 -f "firecracker.*$FC_SOCKET_PAT" 2>/dev/null; then
  echo "firecracker is already running (socket=$FC_SOCKET)"
  exit 0
fi

sudo rm -f "$FC_SOCKET"
sudo ip netns exec "$NODE" /usr/local/bin/firecracker \
  --api-sock "$FC_SOCKET" 2>&1 | sudo tee -a "$FC_LOG" >/dev/null &
FC_PID=$!

for _ in $(seq 1 30); do
  [[ -S $FC_SOCKET ]] && break
  if ! sudo kill -0 "$FC_PID" 2>/dev/null; then
    rm -f "$FC_SOCKET"
    echo "error: firecracker exited, check $FC_LOG" >&2
    exit 1
  fi
  sleep 0.5
done
if [[ ! -S $FC_SOCKET ]]; then
  sudo kill "$FC_PID" 2>/dev/null || true
  rm -f "$FC_SOCKET"
  echo "error: firecracker socket not created after 15s, check $FC_LOG" >&2
  exit 1
fi

fc_api() {
  sudo curl -sf --unix-socket "$FC_SOCKET" -H "Content-Type: application/json" "$@"
}

fc_api -X PUT http://localhost/machine-config -d \
  "{\"vcpu_count\": $FC_VCPU, \"mem_size_mib\": $FC_MEMORY}"

fc_api -X PUT http://localhost/boot-source -d \
  "{\"kernel_image_path\": \"$FC_DATA_DIR/boot/vmlinux\", \"initrd_path\": \"$FC_DATA_DIR/boot/initrd.cpio.gz\", \"boot_args\": \"console=ttyS0 reboot=k panic=1\"}"

fc_api -X PUT http://localhost/drives/rootfs -d \
  "{\"drive_id\": \"rootfs\", \"path_on_host\": \"$FC_DATA_DIR/boot/rootfs.squashfs\", \"is_root_device\": false, \"is_read_only\": true}"

fc_api -X PUT http://localhost/network-interfaces/eth0 -d \
  "{\"iface_id\": \"eth0\", \"guest_mac\": \"$FC_MAC\", \"host_dev_name\": \"$TAP_DEV\"}"

fc_api -X PUT http://localhost/mmds/config -d \
  '{"version": "V1", "network_interfaces": ["eth0"]}'

SSH_KEY="$(tr -d '\n' <"$SSH_KEY_FILE")"
fc_api -X PUT http://localhost/mmds -d \
  "$(jq -n \
    --arg fc_ipv4 "$FC_IPV4" \
    --argjson prefix4 "$PREFIX4" \
    --arg fc_ipv6 "$FC_IPV6" \
    --argjson prefix6 "$PREFIX6" \
    --arg tap_ipv4 "$TAP_IPV4" \
    --arg tap_ipv6 "$TAP_IPV6" \
    --arg ssh_public_key "$SSH_KEY" \
    '{fc_ipv4:$fc_ipv4,prefix4:$prefix4,fc_ipv6:$fc_ipv6,prefix6:$prefix6,tap_ipv4:$tap_ipv4,tap_ipv6:$tap_ipv6,ssh_public_key:$ssh_public_key}')"

fc_api -X PUT http://localhost/actions -d \
  '{"action_type": "InstanceStart"}'

echo "firecracker started in $NODE (pid=$FC_PID, socket=$FC_SOCKET)"
