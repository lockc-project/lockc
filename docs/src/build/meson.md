### Meson

If you are comfortable with installing all dependencies on your host
system, you need to install the following software:

* meson
* rust, cargo
* llvm, clang
* libbpf
* bpftool

Build can be performed by the following commands:

```bash
CC=clang meson build
cd build
meson compile
```

Installation can be perfomed by:

```bash
meson install
```
