globals {
  cpu_vendor = "intel"
  # Host bridge connecting all nodes
  bridge_name    = "br-fc"
  bridge_ipv4    = "10.0.0.1"
  bridge_prefix4 = 24
  bridge_ipv6    = "fd10::1"
  bridge_prefix6 = 64

  fc_data_dir  = "/data/firecracker"

  # Firecracker + kernel/rootfs (Firecracker CI artifacts)
  cpu_arch            = "x86_64"
  firecracker_version = "1.16.0"
  kernel_version      = "6.1.174"
  firecracker_ci_base = "https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/20260617-a093a70f7979-0"
  vmlinux_url         = "${global.firecracker_ci_base}/${global.cpu_arch}/vmlinux-${global.kernel_version}"
  rootfs_squashfs_url = "${global.firecracker_ci_base}/${global.cpu_arch}/ubuntu-24.04.squashfs"
  rootfs_filename     = "rootfs.ext4"

  # Boot args: no nested KVM workarounds needed for direct KVM
  fc_boot_args = "console=ttyS0 root=/dev/vda rootfstype=ext4 reboot=k panic=1 rw"
  fc_vcpu      = 2
  fc_memory    = 1024
}
