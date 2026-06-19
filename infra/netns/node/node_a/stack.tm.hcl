stack {
  tags  = ["netns-node"]
  after = ["/infra/netns/host"]
}

globals {
  node_name    = "${terramate.stack.name}"
  node_ipv4    = "10.0.0.2"
  node_prefix4 = 24
  node_ipv6    = "fd10::2"
  node_prefix6 = 64

  fc_node_dir = "/data/firecracker/${terramate.stack.name}"
  fc_socket   = "/tmp/${terramate.stack.name}-fc.sock"

  # Tap device inside the namespace
  tap_dev     = "tap0"
  tap_ipv4    = "172.16.0.0"
  tap_prefix4 = 31
  tap_ipv6    = "fdfc:a::0"
  tap_prefix6 = 127

  fc_mac         = "06:00:AC:10:00:01"
  fc_ipv4        = "172.16.0.1"
  fc_ipv6        = "fdfc:a::1"
  fc_ipv6_prefix = "fdfc:a::"
  fc_peers       = ["fdfc:b::1"]
}
