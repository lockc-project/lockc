#!/bin/bash

cd /home/vagrant/enclave

export CLANG=/usr/bin/clang-12
cargo install --path .

sudo install -D -m 0644 contrib/systemd/enclave.service /etc/systemd/system/enclave.service
sudo systemctl enable enclave.service
sudo systemctl start enclave.service
