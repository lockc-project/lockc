#!/bin/bash

set -eux

CC=${CC:-"$1"}

cargo libbpf build --clang-path "${CC}"
cargo libbpf gen
