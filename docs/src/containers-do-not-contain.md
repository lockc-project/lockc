# Containers do not contain

Many people assume that containers:

- provide the same or similar isolation to virtual machines
- protects the host system
- sandboxes applications

While all the points except the first one are partially true, some parts of the
host filesystems are still exposed by default to containers and there are ways to
gain full access.

This section highlights and explains problematic exploitation possibilities
that **lockc** aims to fix via policies.

Please note that as **lockc** is still in early development stage, it doesn't
protect against all examples provided at this time. However, covering them all
is in the roadmap.

The goal of **lockc** is to eventually prevent any of those examples to be done
by a regular user. Following some examples as root by explicitly choosing the
*privileged* policy level in lockc is going to be still allowed. However, it is
is discouraged to use the *priviliged* level for containers which are not part
of Kubernetes infra (CNI plugins, operators, network meshes etc.). We might
still consider restricting some of behaviors even for *privileged* (i.e. it's
probably hard to justify `chroot` inside containers under any ciricumstance).

## Not everything is namespaced

Despite the fact that containers come with their own rootfs, some parts of the
filesystem are **not namespaced**, which means that the content of some
directories is **exactly the same as on the host OS**. Examples:

- Kernel filesystems under */sys*
- many sysctls under */proc/sys*

For non-privileged containers, the content of those directories is read-only.
However, privileged containers can write to them. In both cases, we think that
even exposing many of those directories without write access is unnecessary
for regular containers.

To show some more concrete examples, access to those directories can allow to:

- Check and change GPU settings

```bash
❯ docker run --rm -it opensuse/tumbleweed:latest bash
f4891490a2f3:/ # cat /sys/class/drm/card0/device/power_dpm_force_performance_level
auto
f4891490a2f3:/ # exit
❯ docker run --rm --privileged -it opensuse/tumbleweed:latest bash
bad479286479:/ # echo high > /sys/class/drm/card0/device/power_dpm_force_performance_level
bad479286479:/ # cat /sys/class/drm/card0/device/power_dpm_force_performance_level
high
bad479286479:/ # exit
❯ cat /sys/class/drm/card0/device/power_dpm_force_performance_level
high
```

- look at the host OS filesystem metadata

```bash
❯ docker run --rm -it opensuse/tumbleweed:latest bash
0d35122d08f9:~ # ls /sys/fs/btrfs/a8222a26-d11e-4276-9c38-9df2812cead2/
allocation  bdi  bg_reclaim_threshold  checksum  clone_alignment  devices  devinfo  exclusive_operation  features  generation  label  metadata_uuid  nodesize  qgroups  quota_override  read_policy  sectorsize
```

- use fdisk in a privileged container

```bash
❯ docker run --rm -it --privileged registry.opensuse.org/opensuse/toolbox:latest bash
8b71e0119552:/ # fdisk -l
Disk /dev/nvme0n1: 1.82 TiB, 2000398934016 bytes, 3907029168 sectors
Disk model: Samsung SSD 970 EVO Plus 2TB
Units: sectors of 1 * 512 = 512 bytes
Sector size (logical/physical): 512 bytes / 512 bytes
I/O size (minimum/optimal): 512 bytes / 512 bytes
Disklabel type: gpt
Disk identifier: 8EEBDAB8-F965-4BA0-918A-2671BC67117C

Device           Start        End    Sectors  Size Type
/dev/nvme0n1p1    2048    1026047    1024000  500M EFI System
/dev/nvme0n1p2 1026048 3907029134 3906003087  1.8T Linux filesystem
```

## Host mounts

Container engines allow to bind mount any directory from the host. When using
local, non-clusterized container engines (docker, podman etc.) there are no
restrictions about what can be mounted. In case of Docker, anyone who has an
access to the socket (usually a member of `docker` group) can mount anything.

That gives every member of the `docker` group an access to the host OS as root:

```bash
❯ docker run --rm --privileged -it -v /:/rootfs opensuse/tumbleweed:latest bash
efa4f6e0529a:/ # chroot /rootfs
sh-4.4#
```

The `chroot` works without `--privileged` as well:

```bash
❯ docker run --rm -it -v /:/rootfs opensuse/tumbleweed:latest bash
abb67212044d:/ # chroot /rootfs
sh-4.4#
```

The other approach is to mount a Docker socket. The image used here is `docker`
which is the official image with Docker binaries installed. After starting the
first container, we are able to list containers running on the host. Then, we
are able to run another container - from inside the first one - which is
mounting directories from the host

```bash
❯ docker run --rm -it -v /var/run/docker.sock:/var/run/docker.sock docker sh
/ # docker ps
CONTAINER ID   IMAGE     COMMAND                  CREATED         STATUS         PORTS     NAMES
066811b60d69   docker    "docker-entrypoint.s…"   5 seconds ago   Up 5 seconds             suspicious_liskov
/ # docker run --rm --privileged -it opensuse/tumbleweed:latest bash
fcb94c1d3af6:/ # exit
/ # docker run --rm --privileged -it -v /:/rootfs opensuse/tumbleweed:latest bash
54b08e30fd9e:/ # chroot /rootfs
sh-4.4# cat /etc/os-release
NAME="openSUSE Leap"
VERSION="15.3"
ID="opensuse-leap"
ID_LIKE="suse opensuse"
VERSION_ID="15.3"
PRETTY_NAME="openSUSE Leap 15.3"
ANSI_COLOR="0;32"
CPE_NAME="cpe:/o:opensuse:leap:15.3"
BUG_REPORT_URL="https://bugs.opensuse.org"
HOME_URL="https://www.opensuse.org/"
```

Notice the difference between Linux distibution versions. The second container
image we used is *openSUSE Tumbleweed*, but the host is running
*openSUSE Leap 15.3*.
