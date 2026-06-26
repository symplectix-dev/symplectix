#!/bin/sh
set -e

mount -t proc proc /proc
mount -t sysfs sys /sys
mount -t devtmpfs dev /dev

# Reach MMDS: need a source IP for the socket, use link-local.
ip -4 link set eth0 up
ip -4 addr add 169.254.0.2/16 dev eth0

# Fetch metadata from MMDS V1, JSON format.
META=$(wget -q -O - --header "Accept: application/json" http://169.254.169.254/)

# Remove the temporary link-local address used to reach MMDS.
ip -4 addr del 169.254.0.2/16 dev eth0

FC_IPV4=$(echo "$META" | jq -r '.fc_ipv4')
PREFIX4=$(echo "$META"  | jq -r '.prefix4')
FC_IPV6=$(echo "$META"  | jq -r '.fc_ipv6')
PREFIX6=$(echo "$META"  | jq -r '.prefix6')
TAP_IPV4=$(echo "$META" | jq -r '.tap_ipv4')
TAP_IPV6=$(echo "$META" | jq -r '.tap_ipv6')
SSH_KEY=$(echo "$META"  | jq -r '.ssh_public_key')

ip -4 addr add "$FC_IPV4/$PREFIX4" dev eth0
ip -6 addr add "$FC_IPV6/$PREFIX6" dev eth0
ip -4 route add default via "$TAP_IPV4"
ip -6 route add default via "$TAP_IPV6"

# squashfs (read-only base) is the first block device.
mkdir -p /mnt/lower /mnt/rw /mnt/root
mount -t squashfs -o ro /dev/vda /mnt/lower

# overlayfs: tmpfs upper so writes are per-boot and in RAM.
mount -t tmpfs tmpfs /mnt/rw
mkdir -p /mnt/rw/upper /mnt/rw/work
mount -t overlay overlay \
  -o lowerdir=/mnt/lower,upperdir=/mnt/rw/upper,workdir=/mnt/rw/work \
  /mnt/root

# Write through the overlay mount, not directly into `upper/`.
mkdir -p /mnt/root/root/.ssh
printf '%s\n' "$SSH_KEY" > /mnt/root/root/.ssh/authorized_keys
chmod 700 /mnt/root/root/.ssh
chmod 600 /mnt/root/root/.ssh/authorized_keys

# Mask fcnet-* services so will skip them.
mkdir -p /mnt/root/etc/systemd/system
ln -sf /dev/null /mnt/root/etc/systemd/system/fcnet-setup.service
ln -sf /dev/null /mnt/root/etc/systemd/system/fcnet.service

exec switch_root /mnt/root /sbin/init
