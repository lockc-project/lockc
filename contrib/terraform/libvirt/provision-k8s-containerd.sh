#!/bin/bash

# ensure running as root
if [ "$(id -u)" != "0" ]; then
  exec sudo "$0" "$@"
fi

#CONTAINERD_URL=$(curl -s https://api.github.com/repos/containerd/containerd/releases/latest | jq -r '.assets[] | select(.browser_download_url | contains("cri-containerd-cni") and endswith("linux-amd64.tar.gz")) | .browser_download_url')
# https://github.com/rancher-sandbox/lockc/issues/178
# Using latest containerd v1.6.0 will cause following issue 
# runc: symbol lookup error: runc: undefined symbol: seccomp_notify_respond
curl -L "${CONTAINERD_URL}" | sudo tar --no-overwrite-dir -C / -xz

systemctl enable containerd

CNI_VERSION=$(curl -s https://api.github.com/repos/containernetworking/plugins/releases/latest | jq -r '.tag_name')
ARCH="amd64"
mkdir -p /opt/cni/bin
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

systemctl enable kubelet
