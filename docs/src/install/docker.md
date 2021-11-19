# With Docker

This documentation section explains how to install lockc on a single machine
with Docker. In order to do that, we need to install `lockcd` binary and a
systemd unit for it.

## Installation methods

There are two ways to do that.

### Install with cargo

If you want to install lockc on a machine where you have the source code of
lockc, you can do it with cargo. You need to build lockc with Cargo before
that. After building lockc, you can install it with the following command.

```bash
cargo xtask install
```

Do not run this command with sudo! Why?

tl;dr: you will be asked for password when necessary, don't worry!

Explanation: Running cargo with sudo ends with weird consequences like not
seing cargo content from your home directory or leaving some files owned by
root in `target`. When any destination directory is owned by root, sudo will
be launched automatically by `xtask install` just to perform necessary
installation steps.

By default it tries to install lockcd binary in `/usr/local/bin`, but the
destination directory can be changed by the following arguments:

* `--destdir` - the rootfs of your system, default: `/`
* `--prefix` - prefix of the most of installation destinations, default:
  `usr/local`
* `--bindir` - directory for binary files, default: `bin`
* `--unitdir` - directory for systemd units, default: `lib/systemd/system`
* `--sysconfdir` - directory for configuration files, default: `etc`

By default, binaries are installed from the `debug` target profile. If you want
to change it, use the `--profile` argument. `--profile release` is what you
most likely want to use when packaging or installing on the production system.

### Unpack the bintar

Documentation sections about:

* [building with Dapper](../build/dapper.md)
* [building with Cargo](../build/cargo.md)

mention *Building tarball with binary and unit*. To quickly sum it up, you can
build a "bintar" by doing:

```bash
dapper cargo xtask bintar
```

or:

```bash
cargo xtask bintar
```

Both commands will produce a bintar available as `target/[profile]/lockc.tar.gz`
(i.e. `target/debug/lockc.tar.gz`).

That tarball can be copied to any machine and unpacked with the following
command:

```bash
sudo tar -C / -xzf lockc.tar.gz
```

## Verify the installation

After installing lockc, you should be able to enable and start the lockcd
service:

```bash
sudo systemctl enable --now lockcd
```

After starting the service, you can verify that lockc is running by trying to
run a "not containing" container, like:

```bash
$ docker run --rm -it -v /:/rootfs registry.opensuse.org/opensuse/toolbox:latest
docker: Error response from daemon: OCI runtime create failed: container_linux.go:380: starting container process caused: process_linux.go:545: container init caused: rootfs_linux.go:76: mounting "/" to rootfs at "/rootfs" caused: mount through procfd: operation not permitted: unknown.
ERRO[0020] error waiting for container: context canceled
```

Or you can try to run a less insecure container and try to `ls` the contents
of `/sys`:

```bash
$ docker run --rm -it registry.opensuse.org/opensuse/toolbox:latest
9b34d760017f:/ # ls /sys
ls: cannot open directory '/sys': Operation not permitted
9b34d760017f:/ # ls /sys/fs/btrfs
ls: cannot access '/sys/fs/btrfs': No such file or directory
9b34d760017f:/ # ls /sys/fs/cgroup
blkio  cpu,cpuacct  cpuset   freezer  memory  net_cls           net_prio    pids  systemd
cpu    cpuacct      devices  hugetlb  misc    net_cls,net_prio  perf_event  rdma
```

You should be able to see cgroups (which is fine), but other parts of */sys*
should be hidden.

However, running insecure containers as root with `privileged` policy level
should work:

```bash
$ sudo -i
# docker run --label org.lockc.policy=privileged --rm -it -v /:/rootfs registry.opensuse.org/opensuse/toolbox:latest bash
8ea310609fce:/ # 
```
