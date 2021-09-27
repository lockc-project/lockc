# lockc

[![Crate](https://img.shields.io/crates/v/lockc)](https://crates.io/crates/lockc)
[![Book](https://img.shields.io/website?url=https%3A%2F%2Francher-sandbox.github.io%2Flockc%2F)](https://rancher-sandbox.github.io/lockc/)
[![Discord](https://img.shields.io/discord/874314181191565453)](https://discord.gg/799cmsYB4q)
[![Docs](https://docs.rs/lockc/badge.svg)](https://docs.rs/lockc/)
[![Build Status](https://github.com/rancher-sandbox/lockc/actions/workflows/rust.yml/badge.svg)](https://github.com/rancher-sandbox/lockc/actions/workflows/rust.yml)

**lockc** is open source sofware for providing MAC (Mandatory Access Control)
type of security audit for container workloads.

The main reason why **lockc** exists is that **containers do not contain**.
Containers are not as secure and isolated as VMs. By default, they expose
a lot of information about host OS and provide ways to "break out" from the
container. **lockc** aims to provide more isolation to containers and make them
more secure.

The [Containers do not contain](https://rancher-sandbox.github.io/lockc/containers-do-not-contain.html)
documentation section explains what we mean by that phrase and what kind of
behavior we want to restrict with **lockc**.

The main technology behind lockc is [eBPF](https://ebpf.io/) - to be more
precise, its ability to attach to [LSM hooks](https://www.kernel.org/doc/html/latest/bpf/bpf_lsm.html)

Please note that currently lockc is an experimental project, not meant for
production environment and without any official binaries or packages to use -
currently the only way to use it is building from sources.

See [the full documentation here](https://rancher-sandbox.github.io/lockc/).
And [the code documentation here](https://docs.rs/lockc/).

If you need help or want to talk with contributors, plese come chat with us
on `#lockc` channel on the [Rust Cloud Native Discord server](https://discord.gg/799cmsYB4q).

**lockc's** userspace part is licensed under [Apache License, version 2.0](https://github.com/rancher-sandbox/lockc/blob/main/LICENSE).

eBPF programs inside [src/bpf directory](https://github.com/rancher-sandbox/lockc/tree/main/src/bpf)
are licensed under [GNU General Public License, version 2](https://github.com/rancher-sandbox/lockc/blob/main/src/bpf/LICENSE).
