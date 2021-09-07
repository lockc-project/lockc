#!/bin/bash

CONTAINER_ENGINE=${CONTAINER_ENGINE:-docker}
LOCKC_BUILD_PROFILE=${LOCKC_BUILD_PROFILE:-debug}
LOCKC_REGISTRY=${LOCKC_REGISTRY:-docker.io}
LOCKC_REGISTRY_USER=${LOCKC_REGISTRY_USER:-$(whoami)}
LOCKC_PUSH=${LOCKC_PUSH:-"false"}

IMAGES="lockc-k8s-agent lockc-runc-wrapper lockcd"

for image in ${IMAGES}; do
    echo "Building ${image}"
    ${CONTAINER_ENGINE} build \
        -t ${LOCKC_REGISTRY}/${LOCKC_REGISTRY_USER}/${image} \
        --build-arg PROFILE=${LOCKC_BUILD_PROFILE} \
        --target ${image} \
        .
    echo "Image ${image} build"

    if [[ ${LOCKC_PUSH} == "true" ]]; then
        ${CONTAINER_ENGINE} push ${LOCKC_REGISTRY}/${LOCKC_REGISTRY_USER}/${image}
        echo "Image ${image} pushed"
    fi
done
