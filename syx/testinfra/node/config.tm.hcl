script "up" {
  description = "Create network namespace and start VM"
  job {
    commands = [
      ["ansible-playbook", "../create_netns.yml", "-e", "@vars.yml", "--ask-become-pass"],
      ["bazel", "run", "//syx/syxvm:fc_start", "--", "${terramate.root.path.fs.absolute}/syx/testinfra/testkey.pub"],
    ]
  }
}

script "down" {
  description = "Stop VM and remove network namespace"
  job {
    commands = [
      ["bazel", "run", "//syx/syxvm:fc_stop"],
      ["ansible-playbook", "../remove_netns.yml", "-e", "@vars.yml", "--ask-become-pass"],
    ]
  }
}

generate_file "fc_config.json" {
  condition = tm_contains(terramate.stack.tags, "netns-node")
  content = "${tm_jsonencode({
    node_name   = global.node_name
    fc_socket   = global.fc_socket
    fc_data_dir = global.fc_data_dir
    fc_log      = "${global.fc_node_dir}/fc.log"
    fc_vcpu     = global.fc_vcpu
    fc_memory   = global.fc_memory
    fc_mac      = global.fc_mac
    tap_dev     = global.tap_dev
    fc_ipv4     = global.fc_ipv4
    prefix4     = global.tap_prefix4
    fc_ipv6     = global.fc_ipv6
    prefix6     = global.tap_prefix6
    tap_ipv4    = global.tap_ipv4
    tap_ipv6    = global.tap_ipv6
  })}\n"
}

generate_file "host.yml" {
  condition = tm_contains(terramate.stack.tags, "netns-node")
  content = tm_yamlencode({
    all = {
      hosts = {
        (global.node_name) = {
          ansible_host            = global.node_name
          ansible_ssh_common_args = "-o StrictHostKeyChecking=no -o ProxyCommand='sudo ip netns exec ${global.node_name} nc ${global.fc_ipv4} %p'"
          bridge_ipv4             = global.bridge_ipv4
          bridge_ipv6             = global.bridge_ipv6
          node_name               = global.node_name
          tap_prefix6             = global.tap_prefix6
          tap_ipv4                = global.tap_ipv4
          tap_ipv6                = global.tap_ipv6
          fc_ipv6                 = global.fc_ipv6
          fc_peers                = global.fc_peers
        }
      }
    }
  })
}

generate_file "vars.yml" {
  condition = tm_contains(terramate.stack.tags, "netns-node")
  content = tm_yamlencode({
    bridge_name     = global.bridge_name
    bridge_ipv4     = global.bridge_ipv4
    bridge_ipv6     = global.bridge_ipv6
    node_name       = global.node_name
    node_ipv4       = global.node_ipv4
    node_prefix4    = global.node_prefix4
    node_ipv6       = global.node_ipv6
    node_prefix6    = global.node_prefix6
    tap_dev         = global.tap_dev
    tap_ipv4        = global.tap_ipv4
    tap_prefix4     = global.tap_prefix4
    tap_ipv6        = global.tap_ipv6
    tap_prefix6     = global.tap_prefix6
    fc_data_dir     = global.fc_data_dir
    fc_node_dir     = global.fc_node_dir
    fc_socket       = global.fc_socket
    fc_mac          = global.fc_mac
    fc_ipv4         = global.fc_ipv4
    fc_ipv6         = global.fc_ipv6
    fc_ipv6_prefix  = global.fc_ipv6_prefix
    fc_peers        = global.fc_peers
    rootfs_filename = global.rootfs_filename
  })
}
