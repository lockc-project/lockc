#cloud-config

hostname: ${hostname}
locale: ${locale} # en_US.UTF-8
timezone: ${timezone} # Etc/UTC

mounts:
  - [ lockc, /usr/local/src/lockc, 9p, "trans=virtio,version=9p2000.L,rw", "0", "0" ]

users:
  - name: ${username}
    groups: users, docker
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys:
${authorized_keys}

runcmd:
  - pushd /usr/local/src/lockc
  - install-lockc.sh
  - popd
${commands}
