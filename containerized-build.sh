#!/bin/bash

set -e

CRUNTIME=${CRUNTIME:-"env DOCKER_BUILDKIT=1 docker"}

DESTDIR=${DESTDIR:-/}
PREFIX=${PREFIX:-"/usr/local"}
BINDIR=${BINDIR:-"${PREFIX}/bin"}
UNITDIR=${UNITDIR:-"${PREFIX}/lib/systemd/system"}
SYSCONFDIR=${SYSCONFDIR:-"/etc"}

function do_gen() {
    ${CRUNTIME} build \
        --build-arg USER_ID=$(id -u) \
        --build-arg GROUP_ID=$(id -g) \
        --target gen \
        --tag lockc-gen \
	.
    ${CRUNTIME} run \
        --rm -i \
	--user "$(id -u):$(id -g)" \
        -v $(pwd):/usr/local/src/lockc \
        lockc-gen
}

function do_build() {
    ${CRUNTIME} build \
        --build-arg PREFIX=${PREFIX} \
        --target artifact \
        --output type=local,dest=out \
	.
}

function do_install() {
    cp -R out/* ${DESTDIR}
}

function do_fmt() {
    ${CRUNTIME} build \
        --build-arg USER_ID=$(id -u) \
        --build-arg GROUP_ID=$(id -g) \
        --target clippy \
        --tag lockc-clippy \
        .
    ${CRUNTIME} run \
        --rm -i \
        --user "$(id -u):$(id -g)" \
        -v $(pwd):/usr/local/src/lockc \
        lockc-clippy
}

function do_lint() {
    ${CRUNTIME} build \
        --build-arg USER_ID=$(id -u) \
        --build-arg GROUP_ID=$(id -g) \
        --target clippy \
        --tag lockc-clippy \
        .
    ${CRUNTIME} run \
        --rm -i \
        --user "$(id -u):$(id -g)" \
        -v $(pwd):/usr/local/src/lockc \
        lockc-clippy
}

function do_help() {
    echo "Usage: $(basename $0) <subcommand>"
    echo
    echo "Subcommands:"
    echo "    gen        Compile BPF programs and generate BPF CO-RE skeleton"
    echo "    build      Build lockc"
    echo "    install    Install lockc"
    echo "    fmt        Autoformat code"
    echo "    lint       Code analysis"
    echo "    help       Show help information"
}

case "$1" in
    "gen")
        do_gen
        ;;
    "build")
        do_build
        ;;
    "install")
        do_install
        ;;
    "fmt")
        do_fmt
        ;;
    "lint")
        do_lint
        ;;
    "")
        do_build
        ;;
    "help" | "-h" | "--help")
        do_help
        ;;
    *)
        do_help
        exit 1
        ;;
esac
