### Containerized build

One option for building lockc is using the `containerized-build.sh` script
to perform the build inside container, without installing needed
dependencies on the host system.

This guide assumes that you have `docker` or any other container engine
installed.

The build can be performed by running the following command:

```bash
./containerized-build.sh build
```

or simply:

```bash
./containerized-build.sh
```

`build` is the default subcommand of `containerized-build`. There are
several other subcommands:

```bash
$ ./containerized-build.sh help
Usage: containerized-build.sh <subcommand>

Subcommands:
    gen        Compile BPF programs and generate BPF CO-RE skeleton
    build      Build lockc
    install    Install lockc
    fmt        Autoformat code
    lint       Code analysis
    help       Show help information
```

For following this guide, using the `build` subcommand is enough.

`./containerized-build.sh install` can be used to install
lockc in your host system, which by default means directories like
`/usr/bin`, `/etc`. Target directories can be customized by `DESTDIR`,
`PREFIX`, `BINDIR`, `UNITDIR` and `SYSCONFDIR` environment variables.

`build` should result in binaries produced in the `build/` directory:

```bash
$ ls build/
lockcd  lockc-runc-wrapper
```


