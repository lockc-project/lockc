#!/bin/bash

# ensure running as root
if [ "$(id -u)" != "0" ]; then
  exec sudo "$0" "$@"
fi

HELM_VERSION=$(curl -s https://api.github.com/repos/helm/helm/releases/latest | jq -r '.tag_name')
curl -sSL "https://get.helm.sh/helm-${HELM_VERSION}-linux-amd64.tar.gz" | sudo tar -C /usr/local/bin --strip-components=1 -xzf - linux-amd64/helm
