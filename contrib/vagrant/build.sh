#!/bin/bash

cd /home/vagrant/enclave

cargo install --path .
# cargo install --path . --target-dir /usr/local/bin

sudo install -D -m 0644 contrib/systemd/enclave.service /etc/systemd/system/enclave.service
sudo systemctl enable enclave.service
sudo systemctl start enclave.service
