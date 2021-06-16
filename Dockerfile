# syntax=docker/dockerfile:1
FROM docker.io/library/rust:latest AS build
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
RUN git clone --depth 1 -b \
        v$(curl -s https://www.kernel.org/ | grep -A1 'stable:' | grep -oP '(?<=strong>).*(?=</strong.*)') \
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
WORKDIR /usr/local/src/enclave
COPY . .
RUN rustup component add rustfmt
ENV CLANG=/usr/bin/clang-12
RUN cargo install --path .

FROM build AS rustfmt
CMD ["/usr/local/cargo/bin/cargo", "fmt"]

FROM build AS clippy
RUN rustup component add clippy
CMD ["/usr/local/cargo/bin/cargo", "clippy", "--", "-D", "warnings"]

FROM scratch AS artifact
COPY --from=build /usr/local/cargo/bin/enclave /enclave
