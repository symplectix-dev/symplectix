script "up" {
  description = "Create network namespace"
  job {
    command = ["ansible-playbook", "../create_netns.yml", "-e", "@vars.yml", "--ask-become-pass"]
  }
}

script "down" {
  description = "Remove network namespace"
  job {
    command = ["ansible-playbook", "../remove_netns.yml", "-e", "@vars.yml", "--ask-become-pass"]
  }
}

generate_file "host.yml" {
  condition = tm_contains(terramate.stack.tags, "netns-node")
  content = tm_yamlencode({
    all = {
      hosts = {
        (global.node_name) = {
          ansible_host            = global.node_name
          ansible_ssh_common_args = "-o StrictHostKeyChecking=no -o ProxyCommand='sudo ip netns exec ${global.node_name} nc ${global.fc_ipv4} %p'"
          node_name               = global.node_name
          fc_ipv6                 = global.fc_ipv6
          tap_prefix6             = global.tap_prefix6
          tap_ipv4                = global.tap_ipv4
          tap_ipv6                = global.tap_ipv6
          bridge_ipv4             = global.bridge_ipv4
          bridge_ipv6             = global.bridge_ipv6
          fc_peers                = global.fc_peers
        }
      }
    }
  })
}

generate_file "vars.yml" {
  condition = tm_contains(terramate.stack.tags, "netns-node")
  content = tm_yamlencode({
    node_name       = global.node_name
    node_ipv4       = global.node_ipv4
    node_prefix4    = global.node_prefix4
    node_ipv6       = global.node_ipv6
    node_prefix6    = global.node_prefix6
    fc_node_dir     = global.fc_node_dir
    fc_socket       = global.fc_socket
    tap_dev         = global.tap_dev
    tap_ipv4        = global.tap_ipv4
    tap_prefix4     = global.tap_prefix4
    tap_ipv6        = global.tap_ipv6
    tap_prefix6     = global.tap_prefix6
    fc_mac          = global.fc_mac
    fc_ipv4         = global.fc_ipv4
    fc_ipv6         = global.fc_ipv6
    fc_ipv6_prefix  = global.fc_ipv6_prefix
    fc_peers        = global.fc_peers
    fc_data_dir     = global.fc_data_dir
    rootfs_filename = global.rootfs_filename
    bridge_name     = global.bridge_name
    bridge_ipv4     = global.bridge_ipv4
    bridge_ipv6     = global.bridge_ipv6
  })
}

generate_file "start.sh" {
  condition = tm_contains(terramate.stack.tags, "netns-node")
  content   = <<-EOT
    #!/usr/bin/env bash
    set -euo pipefail

    NODE="${global.node_name}"
    FC_SOCKET="${global.fc_socket}"
    FC_CONFIG="$(dirname "$0")/firecracker_config.json"

    if [[ ! -f "$FC_CONFIG" ]]; then
      echo "error: not found: $FC_CONFIG" >&2
      exit 1
    fi

    FC_SOCKET_PAT="$${FC_SOCKET//./\\.}"
    if pkill -0 -f "firecracker.*$FC_SOCKET_PAT" 2>/dev/null; then
      echo "firecracker is already running (socket=$FC_SOCKET)"
      exit 0
    fi
    FC_LOG="${global.fc_node_dir}/fc.log"
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
  EOT
}

generate_file "stop.sh" {
  condition = tm_contains(terramate.stack.tags, "netns-node")
  content   = <<-EOT
    #!/usr/bin/env bash
    set -euo pipefail

    FC_SOCKET="${global.fc_socket}"
    FC_SOCKET_PAT="$${FC_SOCKET//./\\.}"

    pkill -f "firecracker.*$FC_SOCKET_PAT" || true
    timeout 10 bash -c "while pkill -0 -f 'firecracker.*$FC_SOCKET_PAT' 2>/dev/null; do sleep 0.1; done" || true
    rm -f "$FC_SOCKET"
    echo "firecracker stopped"
  EOT
}

generate_file "firecracker_config.json" {
  condition = tm_contains(terramate.stack.tags, "netns-node")
  content   = <<-EOT
    {
      "boot-source": {
        "kernel_image_path": "${global.fc_data_dir}/vmlinux",
        "boot_args": "${global.fc_boot_args}"
      },
      "drives": [
        {
          "drive_id": "rootfs",
          "path_on_host": "${global.fc_node_dir}/${global.rootfs_filename}",
          "is_root_device": true,
          "is_read_only": false
        }
      ],
      "machine-config": {
        "vcpu_count": ${global.fc_vcpu},
        "mem_size_mib": ${global.fc_memory}
      },
      "network-interfaces": [
        {
          "iface_id": "eth0",
          "guest_mac": "${global.fc_mac}",
          "host_dev_name": "${global.tap_dev}"
        }
      ]
    }
  EOT
}
