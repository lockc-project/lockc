# lockc

[![Crate](https://img.shields.io/crates/v/lockc)](https://crates.io/crates/lockc)
[![Docs](https://docs.rs/lockc/badge.svg)](https://docs.rs/lockc/)
[![Build Status](https://github.com/rancher-sandbox/lockc/actions/workflows/rust.yml/badge.svg)](https://github.com/rancher-sandbox/lockc/actions/workflows/rust.yml)

**lockc** is open source sofware for providing MAC (Mandatory Access Control)
type of security audit for container workloads.

The main technology behind lockc is [eBPF](https://ebpf.io/) - to be more
precise, its ability to attach to [LSM hooks](https://www.kernel.org/doc/html/latest/bpf/bpf_lsm.html)

License for eBPF programs: GPLv2

License for userspace part: Apache-2.0

Please note that currently lockc is an experimental project, not meant for
production environment and without any official binaries or packages to use -
currently the only way to use it is building from sources.

See [the full documentation here](https://docs.rs/lockc/).
