![lockc](https://raw.githubusercontent.com/lockc-project/assets/main/logo-horizontal-lockc.png)

[![Crate](https://img.shields.io/crates/v/lockc)](https://crates.io/crates/lockc)
[![Book](https://img.shields.io/website?url=https%3A%2F%2Flockc-project.github.io%2Flockc%2F)](https://lockc-project.github.io/lockc/)
[![Discord](https://img.shields.io/discord/874314181191565453?label=discord&logo=discord)](https://discord.gg/799cmsYB4q)
[![Docs](https://docs.rs/lockc/badge.svg)](https://docs.rs/lockc/)
[![Build Status](https://github.com/lockc-project/lockc/actions/workflows/rust.yml/badge.svg)](https://github.com/lockc-project/lockc/actions/workflows/rust.yml)

**lockc** is open source sofware for providing MAC (Mandatory Access Control)
type of security audit for container workloads.

The main reason why **lockc** exists is that **containers do not contain**.
Containers are not as secure and isolated as VMs. By default, they expose
a lot of information about host OS and provide ways to "break out" from the
container. **lockc** aims to provide more isolation to containers and make them
more secure.

The [Containers do not contain](https://lockc-project.github.io/book/containers-do-not-contain.html)
documentation section explains what we mean by that phrase and what kind of
behavior we want to restrict with **lockc**.

The main technology behind lockc is [eBPF](https://ebpf.io/) - to be more
precise, its ability to attach to [LSM hooks](https://docs.kernel.org/bpf/prog_lsm.html)

Please note that currently lockc is an experimental project, not meant for
production environment and without any official binaries or packages to use -
currently the only way to use it is building from sources.

See [the full documentation here](https://lockc-project.github.io/).
And [the code documentation here](https://docs.rs/lockc/).

If you need help or want to talk with contributors, plese come chat with us
on `#lockc` channel on the [Rust Cloud Native Discord server](https://discord.gg/799cmsYB4q).

**lockc's** userspace part is licensed under [Apache License, version 2.0](https://github.com/lockc-project/lockc/blob/main/LICENSE).

eBPF programs inside [lockc/src/bpf directory](https://github.com/lockc-project/lockc/tree/main/lock-ebpf)
are licensed under [GNU General Public License, version 2](https://github.com/lockc-project/lockc/blob/main/lockc-ebpf/LICENSE).
