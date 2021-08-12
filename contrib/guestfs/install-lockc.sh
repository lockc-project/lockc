#!/bin/bash

set -eux

pushd /home/opensuse/lockc

if [ -d "out/" ]; then
    # lockc was build with containerized-build.sh script
    ./containerized-build.sh install
elif [ -d "build/" ]; then
    # lockc was build with Meson
    pushd build
    meson install
    popd
else
    echo "lockc was not build, cannot install" >&2
    exit 1
fi

popd
