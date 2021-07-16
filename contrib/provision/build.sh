#!/bin/bash

cd /home/vagrant/lockc

export CLANG=/usr/bin/clang-12
cargo install --path .

sudo install -D -m 0644 contrib/systemd/lockcd.service /etc/systemd/system/lockcd.service
sudo systemctl enable lockcd.service
sudo systemctl start lockcd.service
