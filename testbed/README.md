# testinfra

Virtual network infrastructure for testing.

## Architecture

vext netns acts as a mesh hub. Each zone has a bridge inside vext, and
nodes connect to it directly. VM-to-VM traffic (fdfc::) stays within
vext; external traffic exits via the host.

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
vext netns
  br-nrt:          10.0.0.1/24      fd10:6e72:74::1/64
---
nrt-1  vext:       10.0.0.2/24      fd10:6e72:74::2/64
nrt-1  tap0:       172.16.0.0/31    fdfc:abad:face:6e72:7400:0:0:1/112
vm     eth0:       172.16.0.1/31    fdfc:abad:face:6e72:7400:0:0:2/112
---
nrt-2  vext:       10.0.0.3/24      fd10:6e72:74::3/64
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

## TODOs

### A dedicated MMDS tap

Add a dedicated tap for MMDS so that the metadata endpoint (169.254.169.254)
is served on a separate interface from VM traffic.

The MMDS tap has only a link-local address (169.254.169.254 on the tap side,
no IP on the host side, no routing). This ensures metadata packets never
enter the data path. With a shared tap, packets destined for 169.254.169.254
could accidentally reach the host side and be forwarded or leak outside the
node netns.

### Remove IPv4 DNAT from node_ipv4 to VM

Currently the fc nftables table DNATs traffic destined for node_ipv4
(e.g. 10.0.0.2) to the VM's tap IP. This creates an unintended path where
any host on the internal network can reach a VM via the node's IP.

node_ipv4 should be used only for node-to-node and node-to-vext routing,
not as a reachable address for VM traffic. Remove the DNAT prerouting rule
and rely solely on fdfc:: for VM-to-VM communication.
