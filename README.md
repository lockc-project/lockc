# enclave

[![Build Status](https://github.com/rancher-sandbox/enclave/actions/workflows/rust.yml/badge.svg)](https://github.com/rancher-sandbox/enclave/actions/workflows/rust.yml)

Enclave is open source sofware for providing MAC (Mandatory Access Control)
type of security audit for container workloads.

The main technology behind enclave is [eBPF](https://ebpf.io/) - to be more
precise, its ability to attach to [LSM hooks](https://www.kernel.org/doc/html/latest/bpf/bpf_lsm.html)

License for eBPF programs: GPLv2

License for userspace part: Apache-2.0

Please note that currently enclave is an experimental project, not meant for
production environment and without any official binaries or packages to use -
currently the only way to use it is building from sources.

## Architecture

The project consists of two parts:

* the set of BPF programs (written in C)
  * programs for monitoring processes, which detects the container runtime init
    (which is considered to be a new "container") and all its children
  * programs attached to particular LSM hooks, which allow or deny actions
    based on the policy applied to the container (currently all containers have
    the `baseline` policy applied, the mechanism of differentiating between
    policies per container/pod is yet to be implemented)
* the userspace program (written in Rust)
  * loads the BPF programs into the kernel, pins them in BPFFS
  * in future, it's going to serve as the configuration manager

## Getting started

This guide assumes that you have `rustc` and `cargo` installed.

Please build the project with the following command:

```bash
cargo build --release
```

Then run it as root:

```bash
sudo ./target/release/enclave
```

To check if the command loaded BPF programs successfully, use `bpftool`:

```bash
sudo bpftool prog
```

The output of that command should contain information similar to:

```bash
$ sudo bpftool prog
[...]
25910: tracing  name sched_process_f  tag 3a6a6e4defce95ab  gpl
        loaded_at 2021-06-02T16:52:57+0200  uid 0
        xlated 2160B  jited 1137B  memlock 4096B  map_ids 14781,14782,14783
        btf_id 18711
25911: lsm  name clone_audit  tag fc30a5b3e6a4610b  gpl
        loaded_at 2021-06-02T16:52:57+0200  uid 0
        xlated 2280B  jited 1196B  memlock 4096B  map_ids 14781,14782,14783
        btf_id 18711
25912: lsm  name syslog_audit  tag 2cdd93e75fa0e936  gpl
        loaded_at 2021-06-02T16:52:57+0200  uid 0
        xlated 816B  jited 458B  memlock 4096B  map_ids 14783,14782
        btf_id 18711
```

To check if containers get "hardened" by enclave, check if you are able to see
the kernel logs from inside the container. Example:

```bash
$ podman run -ti --rm registry.opensuse.org/opensuse/toolbox:latest
a135dbc3ef08:/ # dmesg
dmesg: read kernel buffer failed: Operation not permitted
```

(For now, all containers get the `baseline` policy. The mechanism of
differentiating between policies per container/pod is yet to be implemented.)

That error means that enclave works and prevented containerized processes from
accessing kernel logs.

If `dmesg` ran successfully and shows the kernel logs, it means that something
went wrong and enclave is not working properly.
