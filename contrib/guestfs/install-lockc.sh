#!/bin/bash

set -eux

pushd /usr/local/src/lockc

if [ -d "target/" ]; then
    # Calling xtask binary directly, because doing `cargo xtask ...` on VM has
    # sometimes weird consequences - sometimes cargo tries to rebuild xtask
    # (even though it was built on host already) and then linker complains
    # (obviously because of mismatch between objects which were compiled on the
    # host and those getting compiled inside VM).
    ./target/debug/xtask install
else
    echo "lockc was not build, cannot install" >&2
    exit 1
fi

popd
