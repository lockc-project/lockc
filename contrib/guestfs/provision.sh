#!/bin/bash

# This script is intended to be running INSIDE guestfs.

set -eux

# TODO(vadorovsky): This is a wacky workaround for even more wackier problem
# with resolv.conf in guestfs. Seems like sysconfig-netconfig is somehow
# messing up with how guestfs is generating resolv.conf. That problem is
# specific for openSUSE.
# 169.254.2.3 is the host's address in qemu user mode networking.
echo "nameserver 169.254.2.3" > /etc/resolv.conf

zypper ref
zypper up
zypper dup

zypper install -y -t pattern \
    devel_basis \
    devel_C_C++

zypper install -y \
    bpftool \
    cargo \
    clang \
    conntrack-tools \
    containerd \
    cni-plugins \
    cri-tools \
    docker \
    ebtables \
    ethtool \
    jq \
    kernel-default \
    -kernel-default-base \
    libbpf-devel \
    libopenssl-devel \
    llvm \
    podman \
    podman-cni-config \
    rust \
    rustfmt \
    socat \
    strace \
    tmux \
    wget

# containerd will be configured to use /opt/cni/bin as a directory for CNI
# plugins. Sadly, it doesn't allow to define multiple dirs like cri-o. Let's
# just symlink all the plugins from libexecdir.
# TODO(vadorovsky): maybe contribute to containerd to support multiple CNI bin
# dirs?
mkdir -p /opt/cni/bin
ln -s /usr/libexec/cni/* /opt/cni/bin

# TODO(vadorovsky): Include BPF as an enabled LSM in openSUSE kernel config.
sed -i -e "s/GRUB_CMDLINE_LINUX=.*/GRUB_CMDLINE_LINUX=\"lsm=bpf,integrity\"/" \
    /etc/default/grub
grub2-mkconfig -o /boot/grub2/grub.cfg

systemctl enable containerd
systemctl enable docker

# Use vanilla kubeadm instead of Kubic kubeadm.
DOWNLOAD_DIR=/usr/local/bin
mkdir -p $DOWNLOAD_DIR
RELEASE="$(curl -sSL https://dl.k8s.io/release/stable.txt)"
cd $DOWNLOAD_DIR
curl -L --remote-name-all https://storage.googleapis.com/kubernetes-release/release/${RELEASE}/bin/linux/amd64/{kubeadm,kubelet,kubectl}
chmod +x {kubeadm,kubelet,kubectl}

RELEASE_VERSION=$(curl -s https://api.github.com/repos/kubernetes/release/releases/latest | jq -r '.name')
curl -sSL "https://raw.githubusercontent.com/kubernetes/release/${RELEASE_VERSION}/cmd/kubepkg/templates/latest/deb/kubelet/lib/systemd/system/kubelet.service" | sed "s:/usr/bin:${DOWNLOAD_DIR}:g" | tee /etc/systemd/system/kubelet.service
mkdir -p /etc/systemd/system/kubelet.service.d
curl -sSL "https://raw.githubusercontent.com/kubernetes/release/${RELEASE_VERSION}/cmd/kubepkg/templates/latest/deb/kubeadm/10-kubeadm.conf" | sed "s:/usr/bin:${DOWNLOAD_DIR}:g" | tee /etc/systemd/system/kubelet.service.d/10-kubeadm.conf

systemctl enable kubelet
