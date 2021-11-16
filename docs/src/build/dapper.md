# Dapper

## Building lockc

One option for building lockc is using dapper to perform the build inside
container, without installing needed dependencies on the host system.

This guide assumes that you have `docker` or any other container engine
installed.

The first step is to install dapper, if it's not present. It can be done
either by downloading a binary:

```bash
curl -sL https://releases.rancher.com/dapper/latest/dapper-$(uname -s)-$(uname -m) > /usr/local/bin/dapper
chmod +x /usr/local/bin/dapper
```

Or by using `go`:

```bash
go install github.com/rancher/dapper@latest
```

Dapper should be launched always in the main directory of the project, where
`Dockerfile.dapper` file is present.

Our build container image has no entrypoint, so calling `dapper` without any
argument is spawning a shell inside the container:

```bash
$ dapper
[...]
root@ea133ef3d28e:/source#
```

Usually we will be interested in using `cargo` inside the container spawned by
dapper.
[More information about cargo can be found here.](cargo.md)

The build (of both BPF and userspace part) can be performed by running the
following command:

```bash
dapper cargo build
```

A successful build should result in binaries being present in `target/debug`
directory.

Running tests:

```bash
dapper cargo test
```

Running lints:

```bash
dapper cargo clippy
```

## Building tarball with binary and unit

To make distribution of lockc for Docker users easier, we have a possibility of
building an archive with binary and systemd unit which can be just unpacked in
`/` directory. It can be done by the following command:

```bash
dapper cargo xtask bintar
```

By default it archives lockcd binary in `usr/local/bin`, but the
destination directory can be changed by the following arguments:

* `--prefix` - prefix of the most of installation destinations, default:
  `usr/local`
* `--bindir` - directory for binary files, default: `bin`
* `--unitdir` - directory for systemd units, default: `lib/systemd/system`
* `--sysconfdir` - directory for configuration files, default: `etc`

By default, binaries are installed from the `debug` target profile. If you want
to change it, use the `--profile` argument. `--profile release` is what you
most likely want to use when creating a tarball for releases and production
systems.

The resulting binary should be available as `target/[profile]/lockc.tar.gz`
(i.e. `target/debug/lockc.tar.gz`).
