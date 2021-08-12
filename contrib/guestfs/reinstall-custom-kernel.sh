#!/bin/bash

LOCKC_IMAGE="lockc-base.qcow2"
KERNEL_SOURCE=${KERNEL_SOURCE:-${HOME}/linux}

if [ ! -f ${LOCKC_IMAGE} ]; then
    echo "lockc-base image not present, please run build.sh first" >&2
    exit 1
fi

virt-customize -a \
    ${LOCKC_IMAGE} \
    --copy-in ${KERNEL_SOURCE}:/usr/src \
    --run provision/kernel.sh
