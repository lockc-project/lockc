#!/bin/bash

# Workaround for https://github.com/hashicorp/vagrant/issues/1659

cat <<EOF >> /etc/sudoers

vagrant ALL=(ALL) NOPASSWD:ALL
Defaults:vagrant !requiretty
EOF
