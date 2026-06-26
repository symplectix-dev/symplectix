stack {
  name        = "netns host"
  description = "Host setup for network namespaces"
}

globals {
  ssh_key_path = "../testkey.pub"
}

script "up" {
  description = "Create host bridge"
  job {
    command = ["ansible-playbook", "setup_host.yml", "create_bridge.yml", "-e", "@vars.yml", "--ask-become-pass"]
  }
}

script "down" {
  description = "Remove host bridge"
  job {
    command = ["ansible-playbook", "remove_bridge.yml", "-e", "@vars.yml", "--ask-become-pass"]
  }
}

generate_file "vars.yml" {
  content = tm_yamlencode({
    cpu_arch            = global.cpu_arch
    cpu_vendor          = global.cpu_vendor
    firecracker_version = global.firecracker_version
    fc_data_dir         = global.fc_data_dir
    rootfs_filename     = global.rootfs_filename
    bridge_name         = global.bridge_name
    bridge_ipv4         = global.bridge_ipv4
    bridge_prefix4      = global.bridge_prefix4
    bridge_ipv6         = global.bridge_ipv6
    bridge_prefix6      = global.bridge_prefix6
    vmlinux_url         = global.vmlinux_url
    rootfs_squashfs_url = global.rootfs_squashfs_url
    ssh_public_key      = tm_trimspace(tm_file(global.ssh_key_path))
  })
}
