#!/bin/bash

# This script is intended to be running INSIDE guestfs.
# It provisions the base VM image.

set -eux

# TODO(vadorovsky): This is a wacky workaround for even more wackier problem
# with resolv.conf in guestfs. Seems like sysconfig-netconfig is somehow
# messing up with how guestfs is generating resolv.conf. That problem is
# specific for openSUSE.
# 169.254.2.3 is the host's address in qemu user mode networking.
echo "nameserver 169.254.2.3" > /etc/resolv.conf

zypper ar -p 90 -r \
    https://download.opensuse.org/repositories/Kernel:/stable:/Backport/standard/Kernel:stable:Backport.repo
zypper ar -p 90 -r \
    https://download.opensuse.org/repositories/devel:/languages:/rust/openSUSE_Leap_15.3/devel:languages:rust.repo

# NOTE(vadorovsky): Temporary workaround for non-functioning mirror.
zypper mr -d repo-backports-update

zypper --gpg-auto-import-keys ref
zypper up -y --allow-vendor-change
zypper dup -y --allow-vendor-change

zypper install -y -t pattern \
    devel_basis \
    devel_C_C++

LINUX_DIR="/usr/src/linux"
# Kernel packages when we don't use custom kernel. It means, we install
# kernel-default and uninstall kernel-default-base. We have to do that, since
# the base image is openSUSE JeOS, whose default-base kernel has not enough
# modules to run Kubernetes. So far, JeOS is the only openSUSE flavor which
# has the qcow2 image with cloud-init and is published.
# TODO(vadorovsky): Make a qcow2 image (on OBS) which:
# - is based on openSUSE Leap
# - has cloud-init or ignition installed
# - has the default kernel
# KERNEL_PACKAGES="kernel-default -kernel-default-base"
KERNEL_PACKAGES="kernel-default"

if [[ -d "${LINUX_DIR}" ]]; then
    pushd "${LINUX_DIR}"
    make modules_install install
    popd
    # Don't install any kernel packages when using custom kernel.
    KERNEL_PACKAGES=""
fi

zypper install -y \
    bpftool \
    cargo \
    conntrack-tools \
    cri-tools \
    criu \
    docker \
    ebtables \
    ethtool \
    jq \
    -kernel-default-base \
    libopenssl-devel \
    podman \
    podman-cni-config \
    python3-pip \
    rust \
    socat \
    strace \
    tmux \
    ${KERNEL_PACKAGES} || true

# TODO(vadorovsky): Include BPF as an enabled LSM in openSUSE kernel config.
sed -i -e "s/GRUB_CMDLINE_LINUX=.*/GRUB_CMDLINE_LINUX=\"lsm=bpf,integrity\"/" \
    /etc/default/grub
grub2-mkconfig -o /boot/grub2/grub.cfg

### Add 9p drivers to dracut
cat >> /etc/dracut.conf.d/90-9p.conf << EOF
# Add 9p 9pnet and 9pnet_virtio modules
add_drivers+=" 9p 9pnet 9pnet_virtio "
EOF

### Rebuild initrd with dracut
mkinitrd

CONTAINERD_URL=$(curl -s https://api.github.com/repos/containerd/containerd/releases/latest | jq -r '.assets[] | select(.browser_download_url | contains("cri-containerd-cni") and endswith("linux-amd64.tar.gz")) | .browser_download_url')
curl -L "${CONTAINERD_URL}" | sudo tar --no-overwrite-dir -C / -xz

# mv /etc/containerd/config.toml.rpmorig /etc/containerd/config.toml

systemctl enable containerd
systemctl enable docker

CNI_VERSION=$(curl -s https://api.github.com/repos/containernetworking/plugins/releases/latest | jq -r '.tag_name')
ARCH="amd64"
sudo mkdir -p /opt/cni/bin
curl -L "https://github.com/containernetworking/plugins/releases/download/${CNI_VERSION}/cni-plugins-linux-${ARCH}-${CNI_VERSION}.tgz" | sudo tar -C /opt/cni/bin -xz

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

HELM_VERSION=$(curl -s https://api.github.com/repos/helm/helm/releases/latest | jq -r .tag_name)
curl -L "https://get.helm.sh/helm-${HELM_VERSION}-linux-amd64.tar.gz" | tar -xz
install -D -m 0755 linux-amd64/helm /usr/bin/helm
rm -rf linux-amd64

systemctl enable kubelet

curl -sL https://releases.rancher.com/dapper/latest/dapper-$(uname -s)-$(uname -m) > /usr/local/bin/dapper
chmod +x /usr/local/bin/dapper

curl -L --remote-name-all https://github.com/cilium/cilium-cli/releases/latest/download/cilium-linux-amd64.tar.gz{,.sha256sum}
sha256sum --check cilium-linux-amd64.tar.gz.sha256sum
sudo tar xzvfC cilium-linux-amd64.tar.gz /usr/local/bin
rm cilium-linux-amd64.tar.gz{,.sha256sum}
