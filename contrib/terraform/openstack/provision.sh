#!/bin/bash

# ensure running as root
if [ "$(id -u)" != "0" ]; then
  exec sudo "$0" "$@"
fi

set -eux

# TODO(vadorovsky): Include BPF as an enabled LSM in openSUSE kernel config.
sed -i -e "s/GRUB_CMDLINE_LINUX=.*/GRUB_CMDLINE_LINUX=\"lsm=bpf,integrity\"/" \
    /etc/default/grub
grub2-mkconfig -o /boot/grub2/grub.cfg

# Load br_netfilter
cat >> /etc/modules-load.d/99-k8s.conf << EOF
br_netfilter
EOF

# Network-related sysctls
cat >> /etc/sysctl.d/99-k8s.conf << EOF
net.bridge.bridge-nf-call-ip6tables = 1
net.bridge.bridge-nf-call-iptables = 1
net.ipv4.ip_forward = 1
net.ipv4.conf.all.forwarding = 1
EOF

# Add 9p drivers to dracut
cat >> /etc/dracut.conf.d/90-9p.conf << EOF
# Add 9p 9pnet and 9pnet_virtio modules
add_drivers+=" 9p 9pnet 9pnet_virtio "
EOF

# Rebuild initrd with dracut
mkinitrd

exit 0
