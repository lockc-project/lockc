#!/bin/bash

set -eux

pushd /usr/local/src/lockc

if [ -d "target/" ]; then
    cargo xtask install
else
    echo "lockc was not build, cannot install" >&2
    exit 1
fi

popd
