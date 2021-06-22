#!/bin/bash

zypper install -y -t pattern kubic_admin
install -D -m 0644 /home/vagrant/enclave/contrib/crio/00-default.conf /etc/crio/crio.conf.d/00-default.conf
systemctl restart crio
systemctl enable kubelet.service
