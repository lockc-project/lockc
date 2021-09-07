#!/bin/bash

set -eux

pushd /usr/local/src/lockc

if [ -d "build/" ]; then
    # lockc was built with Meson
    pushd build
    meson install --no-rebuild
    popd
else
    echo "lockc was not build, cannot install" >&2
    exit 1
fi

popd
