#!/bin/bash

pushd /home/vagrant/enclave
install -D -m 0644 contrib/etc/containerd/config.toml /etc/containerd/config.toml
install -D -m 0644 contrib/etc/docker/daemon.json /etc/docker/daemon.json
install -D -m 0644 contrib/systemd/containerd.service /etc/systemd/system/containerd.service
popd

systemctl enable --now containerd
systemctl enable --now docker

CNI_VERSION="v0.9.1"
sudo mkdir -p /opt/cni/bin
curl -L "https://github.com/containernetworking/plugins/releases/download/${CNI_VERSION}/cni-plugins-linux-amd64-${CNI_VERSION}.tgz" | sudo tar -C /opt/cni/bin -xz

DOWNLOAD_DIR=/usr/local/bin
sudo mkdir -p $DOWNLOAD_DIR

CRI_TOOLS_VERSION="v1.21.0"
wget https://github.com/kubernetes-sigs/cri-tools/releases/download/$CRI_TOOLS_VERSION/crictl-$CRI_TOOLS_VERSION-linux-amd64.tar.gz
sudo tar zxvf crictl-$CRI_TOOLS_VERSION-linux-amd64.tar.gz -C /usr/local/bin
rm -f crictl-$CRI_TOOLS_VERSION-linux-amd64.tar.gz

RELEASE="$(curl -sSL https://dl.k8s.io/release/stable.txt)"
cd $DOWNLOAD_DIR
sudo curl -L --remote-name-all https://storage.googleapis.com/kubernetes-release/release/${RELEASE}/bin/linux/amd64/{kubeadm,kubelet,kubectl}
sudo chmod +x {kubeadm,kubelet,kubectl}

RELEASE_VERSION="v0.9.0"
curl -sSL "https://raw.githubusercontent.com/kubernetes/release/${RELEASE_VERSION}/cmd/kubepkg/templates/latest/deb/kubelet/lib/systemd/system/kubelet.service" | sed "s:/usr/bin:${DOWNLOAD_DIR}:g" | sudo tee /etc/systemd/system/kubelet.service
sudo mkdir -p /etc/systemd/system/kubelet.service.d
curl -sSL "https://raw.githubusercontent.com/kubernetes/release/${RELEASE_VERSION}/cmd/kubepkg/templates/latest/deb/kubeadm/10-kubeadm.conf" | sed "s:/usr/bin:${DOWNLOAD_DIR}:g" | sudo tee /etc/systemd/system/kubelet.service.d/10-kubeadm.conf

systemctl enable --now kubelet
