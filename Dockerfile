# syntax=docker/dockerfile:1
FROM docker.io/library/rust:latest AS buildbase
RUN wget https://apt.llvm.org/llvm-snapshot.gpg.key && \
    apt-key add llvm-snapshot.gpg.key && \
    rm -f llvm-snapshot.gpg.key && \
    echo "deb http://apt.llvm.org/buster/ llvm-toolchain-buster-12 main" > /etc/apt/sources.list.d/llvm.list && \
    apt update && \
    apt upgrade -y --no-install-recommends && \
    apt install -y --no-install-recommends \
        clang-12 \
        libelf-dev \
        gcc-multilib \
        lld-12 \
        lldb-12 && \
    apt purge --auto-remove && \
    apt clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
WORKDIR /usr/local/src
# Build libbpf and bpftool from the newest stable kernel sources.
ARG KERNEL_TAG=v5.13.1
RUN git clone --depth 1 -b \
        ${KERNEL_TAG} \
        git://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git && \
    cd linux && \
    cd tools/lib/bpf && \
    make -j $(nproc) && \
    make install prefix=/usr && \
    cd ../../bpf/bpftool && \
    make -j $(nproc) && \
    make install prefix=/usr && \
    cd ../../../.. && \
    rm -rf linux
ARG USER_ID
ARG GROUP_ID
USER ${USER_ID}:${GROUP_ID}
RUN cargo install libbpf-cargo
RUN rustup component add rustfmt
WORKDIR /usr/local/src/enclave

FROM buildbase AS gen
ARG USER_ID
ARG GROUP_ID
USER ${USER_ID}:${GROUP_ID}
CMD ["/usr/bin/make", "gen", "CLANG=/usr/bin/clang-12", "CONTAINERIZED_BUILD=0"]

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

FROM buildbase AS build
COPY . .
RUN make build install \
    CLANG=/usr/bin/clang-12 \
    CONTAINERIZED_BUILD=0

FROM scratch AS artifact
COPY --from=build /usr/local/bin/enclaved /enclaved
COPY --from=build /usr/local/bin/enclave-runc-wrapper /enclave-runc-wrapper
