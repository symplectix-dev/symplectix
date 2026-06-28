# testinfra

Virtual network infrastructure for testing Firecracker microVMs.

## Architecture

vwan netns acts as a mesh hub. Each zone has a bridge inside vwan, and
nodes connect to it directly. VM-to-VM traffic (fdfc::) stays within
vwan; external traffic exits via the host.

### TODO: Adding an ingress gateway

syxgw handles DNAT from virtual public IPs to fdfc:: VM addresses,
and advertises the anycast prefix via BGP. It connects to vwan (br-nrt)
like a node, and additionally peers with each node in the zone.

```
[vwan netns]
  br-nrt
  |    |    |
nrt-1 nrt-2 nrt-router    <- syxgw: DNAT (public IP -> fdfc::) + BGP
```

Add as a stack under `zone/nrt/router/` with `after` set to the node
stacks. Nodes themselves only need to forward fdfc:: packets to tap;
the public IP mapping is managed entirely by syxgw.

### TODO: Adding an egress gateway

On an IPv6-only server, nodes have no IPv4. VMs that need to reach IPv4
destinations must route through a designated natgw node that holds an IPv4
address and performs masquerade.

```
[vwan netns]
  br-nrt
  |    |    |
nrt-1 nrt-2 nrt-natgw    <- IPv4 masquerade for outbound VM traffic
```

### TODO: Remove IPv4 DNAT from node_ipv4 to VM

Currently the fc nftables table DNATs traffic destined for node_ipv4
(e.g. 10.0.0.2) to the VM's tap IP. This creates an unintended path where
any host on the internal network can reach a VM via the node's IP.

node_ipv4 should be used only for node-to-node and node-to-vwan routing,
not as a reachable address for VM traffic. Remove the DNAT prerouting rule
and rely solely on fdfc:: for VM-to-VM communication.

### TODO: MMDS tap

Add a dedicated tap for MMDS so that the metadata endpoint (169.254.169.254)
is served on a separate interface from VM traffic.

The MMDS tap has only a link-local address (169.254.169.254 on the tap side,
no IP on the host side, no routing). This ensures metadata packets never
enter the data path. With a shared tap, packets destined for 169.254.169.254
could accidentally reach the host side and be forwarded or leak outside the
node netns.

### TODO: Adding a TC router (inter-zone traffic control)

To inject latency or limit bandwidth between zones (e.g. to simulate
WAN conditions), insert a router between zone bridges in vwan netns.
TC rules (netem, tbf) are applied on the router's veth interfaces.

```
[vwan netns]
  br-nrt --- tc-router --- br-kix    <- netem/tbf on tc-router veths
```

For per-node shaping (not inter-zone), TC can be applied directly on
the node's veth in vwan netns without a dedicated router:

```sh
ip netns exec vwan tc qdisc add dev nrt-1 root netem delay 10ms
```

The TC router, if needed, is a stack under `vwan/tc-router/` brought
up after all zone stacks.

## IPv6 prefix scheme

```
fd10:<zone>::
fdfc:<net>:<zone+node>:<vm-id>/112
```

- `<net>`       = network identifier: `abad:face`
- `<zone>`      = airport code hex: NRT=`6e72:74`, KIX=`6b69:78`
- `<zone+node>` = zone hex + node index byte: nrt-1=`7400`, nrt-2=`7401`
- `<vm-id>`     = 32-bit VM identifier (hi:lo), assigned at VM creation time;
                  tap (::1) and vm (::2)

```
vwan netns
  br-nrt:          10.0.0.1/24      fd10:6e72:74::1/64
---
nrt-1  vwan:       10.0.0.2/24      fd10:6e72:74::2/64
nrt-1  tap0:       172.16.0.0/31    fdfc:abad:face:6e72:7400:0:0:1/112
vm     eth0:       172.16.0.1/31    fdfc:abad:face:6e72:7400:0:0:2/112
---
nrt-2  vwan:       10.0.0.3/24      fd10:6e72:74::3/64
nrt-2  tap0:       172.16.0.0/31    fdfc:abad:face:6e72:7401:0:0:1/112
vm     eth0:       172.16.0.1/31    fdfc:abad:face:6e72:7401:0:0:2/112
```

## Setup

```sh
ansible-playbook netns.yml
```

Network namespaces, veth pairs, service templates, and Firecracker
processes are ephemeral and lost on reboot.

## SSH into microVM

```sh
ssh -i testbed/testkey \
    -o StrictHostKeyChecking=no \
    -o UserKnownHostsFile=/dev/null \
    -o "ProxyCommand=sudo ip netns exec nrt-1 nc 172.16.0.1 22" \
    root@nrt-1
```
