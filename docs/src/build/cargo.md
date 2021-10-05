### Cargo

If you are comfortable with installing all dependencies on your host system,
you need to install the following software:

* LLVM
* libbpf, bpftool
* Rust, Cargo

#### LLVM

We need a recent version of LLVM (at least 12) to build BPF programs.

LLVM has an official [apt repository](https://apt.llvm.org/) with recent
stable versions.

Distributions with up to date software repositories like Arch, Fedora, openSUSE
Tumbleweed are shipping recent versions of LLVM.

In more stable and not up to date distributions (CentOS, openSUSE Leap, RHEL,
SLES), using some kind of development repository might be an option. For
example, openSUSE Leap users can use the following devel repo:

```bash
zypper ar -r -p 90 https://download.opensuse.org/repositories/devel:/tools:/compiler/openSUSE_Leap_15.3/devel:tools:compiler.repo
zypper ref
zypper up --allow-vendor-change
zypper in clang llvm
```

If there is no packaging of recent LLVM versions for your distribution, there
is also an option to [download binaries](https://releases.llvm.org/download.html).

#### libbpf, bpftool

libbpf is the official C library for writing, loading and managing BPF programs
and entities. bpftool is the official CLI for interacting with BPF subsystem.

Distributions with up to date software (Arch, Fedora, openSUSE Tumbleweed)
usually provide packaging for both.

Especially for more stable and less up to date distributions, but even
generally, we would recommend to build both dependencies from source. Both of
them are the part of the Linux kernel source.

The easiest way to get the kernel source is to download a tarball available on
[kernel.org](https://www.kernel.org/). Then build and install tools from it
(the version might vary from this snippet):

```bash
tar -xvf linux-5.14.9.tar.xz
cd linux-5.14.9
cd tools/lib/bpf
make -j $(nproc)
make install prefix=/usr
cd ../../bpf/bpftool
make -j $(nproc)
make install prefix=/usr
```

If you are interested in tracking the history of Linux kernel source and/or are
comfortable using git for it, you can clone one of the git trees:

* [stable tree](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/) -
  stable releases and release candidates, this is where the tarball comes from
* [mainline tree](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/) -
  patches accepted by Linus, release candidates
* [bpf-next tree](https://git.kernel.org/pub/scm/linux/kernel/git/bpf/bpf-next.git/) -
  development of BPF features, before being mainlined
* [bpf tree](https://git.kernel.org/pub/scm/linux/kernel/git/bpf/bpf.git/) -
  BPF bugfixes which are backported to the stable tree

Assuming you want to use the stable tree:

```bash
git clone git://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git
cd linux
git tag -l # List available tags
git checkout v5.14.9 # Check out to whatever is the newest
cd tools/lib/bpf
make -j $(nproc)
make install prefix=/usr
cd ../../bpf/bpftool
make -j $(nproc)
make install prefix=/usr
```

#### Installing Rust

Our recommended way of installing Rust is using **rustup**.
[Their website](https://rustup.rs/) contains installation instruction.

After installing rustup, let's install lint tools:

```bash
rustup component add clippy rustfmt
```

And then cargo-libbpf, needed for building the BPF part:

```bash
cargo install libbpf-cargo
```

#### Building lockc

After installing all needed dependencies, it's time to build lockc.

The build of the project can be done with:

```bash
cargo build
```

Running tests:

```bash
cargo test
```

Running lints:

```bash
cargo clippy
```
