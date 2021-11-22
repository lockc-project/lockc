#cloud-config

# set hostname
hostname: ${hostname}

# set locale
locale: ${locale} # en_US.UTF-8

# set timezone
timezone: ${timezone} # Etc/UTC

# Inject the public keys
ssh_authorized_keys:
${authorized_keys}

ntp:
  enabled: true
  ntp_client: chrony
  config:
    confpath: /etc/chrony.conf
  servers:
${ntp_servers}

# need to disable gpg checks because the cloud image has an untrusted repo
zypper:
  repos:
${repositories}
  config:
    gpgcheck: "off"
    solver.onlyRequires: "true"
    download.use_deltarpm: "true"

runcmd:
  # workaround for bsc#1119397 . If this is not called, /etc/resolv.conf is empty
  - netconfig -f update
  # Workaround for bsc#1138557 . Disable root and password SSH login
  - sed -i -e '/^PermitRootLogin/s/^.*$/PermitRootLogin no/' /etc/ssh/sshd_config
  - sed -i -e '/^#ChallengeResponseAuthentication/s/^.*$/ChallengeResponseAuthentication no/' /etc/ssh/sshd_config
  - sed -i -e '/^#PasswordAuthentication/s/^.*$/PasswordAuthentication no/' /etc/ssh/sshd_config
  - sshd -t || echo "ssh syntax failure"
  - systemctl restart sshd
  # Set node's hostname from DHCP server
  - sed -i -e '/^DHCLIENT_SET_HOSTNAME/s/^.*$/DHCLIENT_SET_HOSTNAME=\"yes\"/' /etc/sysconfig/network/dhcp
  - systemctl restart wicked
  # Refresh repos and upgrade
  - zypper ref
  - zypper dup -y --allow-vendor-change --replacefiles
${commands}

final_message: "The system is finally up, after $UPTIME seconds"
