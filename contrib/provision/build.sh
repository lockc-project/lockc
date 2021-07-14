#!/bin/bash

cd /home/vagrant/enclave

export CLANG=/usr/bin/clang-12
cargo install --path .

sudo install -D -m 0644 contrib/systemd/enclaved.service /etc/systemd/system/enclaved.service
sudo systemctl enable enclaved.service
sudo systemctl start enclaved.service
