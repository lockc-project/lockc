#!/bin/bash

set -eux

export OUTPUT1="$2"
export OUTPUT2="$3"
export CARGO_TARGET_DIR="$4"/target
export CARGO_HOME="${CARGO_TARGET_DIR}"/cargo-home

cargo build --manifest-path $1/Cargo.toml --release -p lockc

for out in "${OUTPUT1}" "${OUTPUT2}"; do
    cp "${CARGO_TARGET_DIR}"/release/$(basename "${out}") "${out}"
done
