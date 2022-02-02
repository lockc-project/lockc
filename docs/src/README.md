![lockc](https://rancher-sandbox.github.io/lockc/images/logo-horizontal-lockc.png)

# Introduction

**lockc** is open source software for providing MAC (Mandatory Access Control)
type of security audit for container workloads.

The main reason why **lockc** exists is that **containers do not contain**.
Containers are not as secure and isolated as VMs. By default, they expose
a lot of information about host OS and provide ways to "break out" from the
container. **lockc** aims to provide more isolation to containers and make them
more secure.

The [Containers do not contain](containers-do-not-contain.md) documentation
section explains why we mean by that phrase and what kind of behavior we want
to restrict with **lockc**.

The main technology behind lockc is [eBPF](https://ebpf.io/) - to be more
precise, its ability to attach to [LSM hooks](https://www.kernel.org/doc/html/latest/bpf/bpf_lsm.html)

Please note that currently lockc is an experimental project, not meant for
production environments. Currently we don't publish any official binaries or
packages to use, except of a Rust crate. Currently the most convenient way
to use it is to use the source code and follow the guide.

## Contributing

If you need help or want to talk with contributors, please come chat with
us on `#lockc` channel on the [Rust Cloud Native Discord server](https://discord.gg/799cmsYB4q).

You can find the source code on [GitHub](https://github.com/rancher-sandbox/lockc)
and issues and feature requests can be posted on the
[GitHub issue tracker](https://github.com/rancher-sandbox/lockc/issues).
**lockc** relies on the community to fix bugs and add features: if you'd like
to contribute, please read the [CONTRIBUTING](https://github.com/rancher-sandbox/lockc/blob/master/CONTRIBUTING.md)
guide and consider opening [pull request](https://github.com/rancher-sandbox/lockc/pulls).

## License

**lockc's** userspace part is licensed under [Apache License, version 2.0](https://github.com/rancher-sandbox/lockc/blob/main/LICENSE).

eBPF programs inside [lockc/src/bpf directory](https://github.com/rancher-sandbox/lockc/tree/main/lockc/src/bpf)
are licensed under [GNU General Public License, version 2](https://github.com/rancher-sandbox/lockc/blob/main/lockc/src/bpf/LICENSE).

Documentation is licensed under [Mozilla Public License v2.0](https://www.mozilla.org/MPL/2.0/).
