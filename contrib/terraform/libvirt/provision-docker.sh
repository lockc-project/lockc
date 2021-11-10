#!/bin/bash

# ensure running as root
if [ "$(id -u)" != "0" ]; then
  exec sudo "$0" "$@"
fi

error() { (>&2 echo -e "[ ERROR ] $@") ;}

# Install lockc
tar -C / -xzf /home/opensuse/lockc.tar.gz
systemctl enable lockcd

# Install Docker
DOCKER_VERSION=$(curl -s https://api.github.com/repos/moby/moby/releases/latest | jq -r '.tag_name' | sed -e 's/^v//')
curl -L "https://download.docker.com/linux/static/stable/x86_64/docker-${DOCKER_VERSION}.tgz" | sudo tar -C /usr/local/bin -xz --strip-components=1
curl -sSL "https://raw.githubusercontent.com/moby/moby/v${DOCKER_VERSION}/contrib/init/systemd/docker.service" | sed "s:/usr/bin:/usr/local/bin:g" | tee /etc/systemd/system/docker.service
curl -sSL -o /etc/systemd/system/docker.socket "https://raw.githubusercontent.com/moby/moby/v${DOCKER_VERSION}/contrib/init/systemd/docker.socket"

systemctl enable docker
usermod -aG docker opensuse
