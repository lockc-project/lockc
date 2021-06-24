#!/bin/bash

# Workaround for https://github.com/hashicorp/vagrant/issues/1659

echo "" >> /etc/sudoers
echo "vagrant ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers
