#!/bin/bash

set -eux

export OUTPUT1="$2"
export OUTPUT2="$3"
export CARGO_TARGET_DIR="$4"/target
export CARGO_HOME="${CARGO_TARGET_DIR}"/cargo-home
export PROFILE="$5"

EXTRA_FLAGS="--release"
CARGO_TARGET_SUBDIR="release"

if [[ "${PROFILE}" == "dev" ]]; then
    EXTRA_FLAGS=""
    CARGO_TARGET_SUBDIR="debug"
fi

cargo build --manifest-path $1/Cargo.toml ${EXTRA_FLAGS} -p lockc

for out in "${OUTPUT1}" "${OUTPUT2}"; do
    cp "${CARGO_TARGET_DIR}"/${CARGO_TARGET_SUBDIR}/$(basename "${out}") "${out}"
done
