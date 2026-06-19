stack {
  name        = "netns vms"
  description = "VM lifecycle management"
  after       = ["tag:netns-node"]
}

script "up" {
  description = "Start all VMs and configure networks"
  job {
    commands = [
      ["bash", "-c", "for d in ../node/*/; do sudo bash \"$${d}start.sh\"; done"],
      ["bash", "-c", "ansible-playbook setup_vm_network.yml $(find ../node -mindepth 2 -maxdepth 2 -name host.yml | sort | xargs printf -- '-i %s ')"],
    ]
  }
}

script "down" {
  description = "Stop all VMs"
  job {
    command = ["bash", "-c", "for d in ../node/*/; do sudo bash \"$${d}stop.sh\"; done"]
  }
}
