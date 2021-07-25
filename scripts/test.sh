#!/bin/bash

set -eux

export CARGO_TARGET_DIR="$1/target/"
export CARGO_HOME="$CARGO_TARGET_DIR/cargo-home"

cargo test
