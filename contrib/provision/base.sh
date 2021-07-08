#!/bin/bash

zypper install -y -t pattern \
       devel_basis \
       devel_C_C++
zypper install -y \
       bpftool \
       cargo \
       clang \
       conntrack-tools \
       containerd \
       docker \
       ebtables \
       ethtool \
       libbpf-devel \
       libopenssl-devel \
       llvm \
       podman \
       podman-cni-config \
       rust \
       rustfmt \
       socat \
       tmux \
       wget

pushd /home/vagrant/enclave
install -D -m 0644 contrib/etc/modules-load.d/99-k8s.conf /etc/modules-load.d/99-k8s.conf
install -D -m 0644 contrib/etc/sysctl.d/99-k8s.conf /etc/sysctl.d/99-k8s.conf
popd

sed -i -e "s/GRUB_CMDLINE_LINUX=.*/GRUB_CMDLINE_LINUX=\"lsm=bpf,integrity\"/" \
    /etc/default/grub
grub2-mkconfig -o /boot/grub2/grub.cfg
