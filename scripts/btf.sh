#!/bin/bash

set -eux

OUTPUT="$1/src/bpf/vmlinux.h"

sudo bpftool btf dump file \
    /sys/kernel/btf/vmlinux format c > \
    "${OUTPUT}" ||
    curl -L \
    https://raw.githubusercontent.com/libbpf/libbpf-bootstrap/master/vmlinux/vmlinux_508.h \
    --output "${OUTPUT}"
