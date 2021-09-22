#!/bin/bash

set -eux

pushd /usr/local/src/lockc

if [ -d "build/" ]; then
    if which meson &> /dev/null; then
        # lockc was built with Meson
        pushd build
        meson install --no-rebuild
        popd
    else
        # lockc was build with containerized-build.sh script
        ./containerized-build.sh install
    fi
else
    echo "lockc was not build, cannot install" >&2
    exit 1
fi

popd
