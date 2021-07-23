#!/bin/bash

# This script builds the base qcow2 image for lockc development environment.

BASE_IMAGE="openSUSE-Tumbleweed-JeOS.qcow2"
LOCKC_IMAGE="lockc-base.qcow2"

if [ ! -f ${BASE_IMAGE} ]; then
    wget -O ${BASE_IMAGE} \
        https://download.opensuse.org/tumbleweed/appliances/openSUSE-Tumbleweed-JeOS.x86_64-OpenStack-Cloud.qcow2
fi

rm -f ${LOCKC_IMAGE}
cp ${BASE_IMAGE} ${LOCKC_IMAGE}
qemu-img resize ${LOCKC_IMAGE} 40G
virt-resize --expand /dev/sda3 ${BASE_IMAGE} ${LOCKC_IMAGE}

virt-customize -v -x -a \
    ${LOCKC_IMAGE} \
    --mkdir /etc/containerd \
    --mkdir /etc/docker \
    --copy-in ../etc/modules-load.d/99-k8s.conf:/etc/modules-load.d/ \
    --copy-in ../etc/sysctl.d/99-k8s.conf:/etc/sysctl.d/ \
    --copy-in ../systemd/lockcd.service:/etc/systemd/system/ \
    --copy-in ../etc/containerd/config.toml:/etc/containerd/ \
    --copy-in ../etc/docker/daemon.json:/etc/docker/ \
    --copy-in ../systemd/containerd.service:/etc/systemd/system/ \
    --run provision.sh
