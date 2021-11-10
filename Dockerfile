FROM docker.io/library/rust:latest as builder
RUN wget https://apt.llvm.org/llvm-snapshot.gpg.key && \
    apt-key add llvm-snapshot.gpg.key && \
    rm -f llvm-snapshot.gpg.key && \
    echo "deb http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-13 main" > /etc/apt/sources.list.d/llvm.list && \
    apt update && \
    apt upgrade -y --no-install-recommends && \
    apt install -y --no-install-recommends \
        clang-13 \
        libelf-dev \
        gcc-multilib \
        lld-13 \
        lldb-13 \
        python3-pip \
        sudo && \
    apt purge --auto-remove && \
    apt clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
# Build libbpf and bpftool from the newest stable kernel sources.
RUN curl -Lso linux.tar.xz \
        $(curl -s https://www.kernel.org/ | grep -A1 "latest_link" | grep -Eo '(http|https)://[^"]+') && \
    tar -xf linux.tar.xz && \
    cd $(find . -maxdepth 1 -type d -name "linux*") && \
    cd tools/lib/bpf && \
    make -j $(nproc) && \
    make install prefix=/usr && \
    cd ../../bpf/bpftool && \
    make -j $(nproc) && \
    make install prefix=/usr && \
    cd ../../../.. && \
    rm -rf linux*
RUN cargo install libbpf-cargo
RUN rustup component add \
        clippy \
        rustfmt

FROM builder AS build
WORKDIR /usr/local/src/lockc
COPY . ./
ENV CLANG /usr/bin/clang-13
RUN cargo build

FROM registry.opensuse.org/opensuse/leap-microdnf:15.3 AS lockcd
# runc links those libraries dynamically
RUN microdnf install -y --nodocs \
        libseccomp2 \
        libselinux1 \
    && microdnf clean all
ARG PROFILE=debug
COPY --from=build /usr/local/src/lockc/target/$PROFILE/lockcd /usr/bin/lockcd
ENTRYPOINT ["/usr/bin/lockcd"]
