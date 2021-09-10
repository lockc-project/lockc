# syntax=docker/dockerfile:1
FROM docker.io/library/rust:latest AS buildbase
RUN wget https://apt.llvm.org/llvm-snapshot.gpg.key && \
    apt-key add llvm-snapshot.gpg.key && \
    rm -f llvm-snapshot.gpg.key && \
    echo "deb http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-12 main" > /etc/apt/sources.list.d/llvm.list && \
    apt update && \
    apt upgrade -y --no-install-recommends && \
    apt install -y --no-install-recommends \
        clang-12 \
        libelf-dev \
        gcc-multilib \
        lld-12 \
        lldb-12 \
        python3-pip \
        sudo && \
    apt purge --auto-remove && \
    apt clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
RUN pip3 install \
        meson \
        ninja
WORKDIR /usr/local/src
# Build libbpf and bpftool from the newest stable kernel sources.
RUN curl -Lo linux.tar.xz \
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
ARG USER_ID
ARG GROUP_ID
USER ${USER_ID}:${GROUP_ID}
RUN cargo install libbpf-cargo
RUN rustup component add rustfmt
WORKDIR /usr/local/src/lockc

FROM buildbase AS rustfmt
ARG USER_ID
ARG GROUP_ID
USER ${USER_ID}:${GROUP_ID}
CMD ["/usr/local/cargo/bin/cargo", "fmt"]

FROM buildbase AS clippy
ARG USER_ID
ARG GROUP_ID
USER ${USER_ID}:${GROUP_ID}
RUN rustup component add clippy
CMD ["/usr/local/cargo/bin/cargo", "clippy", "--", "-D", "warnings"]

FROM buildbase AS test
ARG USER_ID
ARG GROUP_ID
USER ${USER_ID}:${GROUP_ID}
CMD ["/usr/local/cargo/bin/cargo", "test"]

FROM buildbase AS build
ARG PREFIX
ENV USER_ID=1000
ENV GROUP_ID=100
COPY scripts/entrypoint.sh /
CMD ["/entrypoint.sh"]
