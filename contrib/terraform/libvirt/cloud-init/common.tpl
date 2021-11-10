#cloud-config

hostname: ${hostname}
locale: ${locale} # en_US.UTF-8
timezone: ${timezone} # Etc/UTC

users:
  - name: opensuse
    groups: users, docker
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys:
${authorized_keys}

zypper:
  repos:
${repositories}
  config:
    gpgcheck: "off"
    solver.onlyRequires: "true"
    download.use_deltarpm: "true"

runcmd:
  # Set node's hostname from DHCP server
  - sed -i -e '/^DHCLIENT_SET_HOSTNAME/s/^.*$/DHCLIENT_SET_HOSTNAME=\"yes\"/' /etc/sysconfig/network/dhcp
  - systemctl restart wicked
  # Refresh repos and upgrade
  - zypper ref
  - zypper dup -y --allow-vendor-change --replacefiles
${commands}
