FROM registry.opensuse.org/opensuse/leap:15.3 as builder
RUN zypper ar -p 90 -r https://download.opensuse.org/repositories/devel:/languages:/rust/openSUSE_Leap_15.3/devel:languages:rust.repo \
    && zypper ar -p 90 -r https://download.opensuse.org/repositories/devel:/tools:/compiler/openSUSE_Leap_15.3/devel:tools:compiler.repo \
    && zypper --gpg-auto-import-keys ref \
    && zypper --non-interactive dup --allow-vendor-change
RUN zypper --non-interactive install -t pattern \
    devel_C_C++ \
    devel_basis \
    && zypper --non-interactive install \
    clang \
    curl \
    libelf-devel \
    libopenssl-devel \
    llvm \
    rustup \
    sudo \
    tar \
    xz \
    zlib-devel \
    && zypper clean
RUN rustup-init -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup component add \
    clippy \
    rustfmt
RUN cargo install \
    libbpf-cargo

FROM builder AS build
WORKDIR /usr/local/src
# Build bpftool from the newest stable kernel sources.
RUN curl -Lso linux.tar.xz \
    $(curl -s https://www.kernel.org/ | grep -A1 "latest_link" | grep -Eo '(http|https)://[^"]+') \
    && tar -xf linux.tar.xz \
    && mv $(find . -maxdepth 1 -type d -name "linux*") linux \
    && cd linux \
    && cd tools/bpf/bpftool \
    && make -j $(nproc)
# Prepare lockc sources and build it.
WORKDIR /usr/local/src/lockc
COPY . ./
RUN cargo build --release

FROM registry.opensuse.org/opensuse/leap:15.3 AS lockcd
# runc links those libraries dynamically
RUN zypper --non-interactive install \
    libseccomp2 \
    libselinux1 \
    && zypper clean
COPY --from=build /usr/local/src/linux/tools/bpf/bpftool/bpftool /usr/sbin/bpftool
COPY --from=build /usr/local/src/lockc/target/release/lockcd /usr/bin/lockcd
ENTRYPOINT ["/usr/bin/lockcd"]
