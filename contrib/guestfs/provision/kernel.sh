#!/bin/bash

# This script is intended to be running INSIDE guestfs.
# It installs a custom kernel in the VM image.

set -eux

# TODO(vadorovsky): This is a wacky workaround for even more wackier problem
# with resolv.conf in guestfs. Seems like sysconfig-netconfig is somehow
# messing up with how guestfs is generating resolv.conf. That problem is
# specific for openSUSE.
# 169.254.2.3 is the host's address in qemu user mode networking.
echo "nameserver 169.254.2.3" > /etc/resolv.conf

# Remove kernel packages
KERNEL_PACKAGES="kernel-default kernel-default-base"
for package in ${KERNEL_PACKAGES}; do
    if rpm -q ${package} &>/dev/null; then
        zypper rm -y ${package}
    fi
done

# Remove all old kernels
rm -f /boot/{config*,initrd*,vmlinux*,vmlinuz*,System.map*}

LINUX_DIR="/usr/src/linux"
pushd "${LINUX_DIR}"
make modules_install install
popd
