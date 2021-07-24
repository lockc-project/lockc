#!/bin/bash

# This script builds the base qcow2 image for lockc development environment.

set -e

LEAP_VERSION="15.3"
BASE_IMAGE="openSUSE-Leap-${LEAP_VERSION}-JeOS.qcow2"
LOCKC_IMAGE="lockc-base.qcow2"

CUSTOM_KERNEL=${CUSTOM_KERNEL:-false}
KERNEL_SOURCE=${KERNEL_SOURCE:-${HOME}/linux}

if [[ ! -f ${BASE_IMAGE} ]]; then
    wget -O ${BASE_IMAGE} \
        http://download.opensuse.org/distribution/leap/15.3/appliances/openSUSE-Leap-15.3-JeOS.x86_64-OpenStack-Cloud.qcow2
fi

rm -f ${LOCKC_IMAGE}
cp ${BASE_IMAGE} ${LOCKC_IMAGE}
qemu-img resize ${LOCKC_IMAGE} 40G
virt-resize --expand /dev/sda3 ${BASE_IMAGE} ${LOCKC_IMAGE}

EXTRA_FLAGS=""

if [[ "${CUSTOM_KERNEL}" == "true" ]]; then
    echo "Building with a custom kernel source tree ${KERNEL_SOURCE}"

    EXTRA_FLAGS+="--copy-in ${KERNEL_SOURCE}:/usr/src/"
fi

set -eux

virt-customize -a \
    ${LOCKC_IMAGE} \
    --mkdir /etc/containerd \
    --mkdir /etc/docker \
    --copy-in ../etc/modules-load.d/99-k8s.conf:/etc/modules-load.d/ \
    --copy-in ../etc/sysctl.d/99-k8s.conf:/etc/sysctl.d/ \
    --copy-in ../systemd/containerd.service:/etc/systemd/system/ \
    --copy-in install-lockc.sh:/usr/sbin/ \
    ${EXTRA_FLAGS} \
    --run provision/provision.sh
