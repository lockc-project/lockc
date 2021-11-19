variable "libvirt_uri" {
  description = "libvirt connection URI"
  default     = "qemu:///system"
}

variable "pool" {
  description = "Pool to be used to store all the volumes"
  default     = "default"
}

variable "image_name" {
  description = "Image name in libvirt"
  default     = "lockc-image"
}

variable "image_path" {
  description = "Path or URL to the image"
  default     = "http://download.opensuse.org/distribution/leap/15.3/appliances/openSUSE-Leap-15.3-JeOS.x86_64-OpenStack-Cloud.qcow2"
}

variable "network_name" {
  description = "Network name in libvirt"
  default     = "lockc-network"
}

variable "network_mode" {
  description = "Network mode in libvirt (nat / none / route / bridge)"
  default     = "nat"
}

variable "dns_domain" {
  description = "DNS domain name"
  default     = "lockc.local"
}

variable "stack_name" {
  description = "Identifier to make all your resources unique and avoid clashes with other users of this terraform project"
  default     = ""
}

variable "network_cidr" {
  description = "Network CIDR"
  default     = "10.16.0.0/24"
}

variable "locale" {
  description = "System locales to set on all the nodes"
  default     = "en_US.UTF-8"
}

variable "timezone" {
  description = "Timezone to set on all the nodes"
  default     = "Etc/UTC"
}

variable "authorized_keys" {
  description = "SSH keys to inject into all the nodes"
  type        = list(string)
  default     = []
}

variable "repositories" {
  description = "Zypper repositories to add"
  type        = map(string)
  default     = {
    Kernel_stable_Backport = "https://download.opensuse.org/repositories/Kernel:/stable:/Backport/standard/"
  }
}

variable "packages" {
  description = "List of addditional packagess to install"
  type        = list(string)
  default     = [
    "bpftool",
    "conntrack-tools",
    "ebtables",
    "ethtool",
    "iptables",
    "jq",
    "kernel-default",
    "-kernel-default-base",
    "socat",
    "strace",
    "tmux"
  ]
}

variable "enable_docker" {
  description = "Enable Docker support (as a non-clustered container engine)"
  type        = bool
  default     = false
}

variable "enable_k8s_containerd" {
  description = "Enable Kubernetes with containerd CRI"
  type        = bool
  default     = true
}

variable "username" {
  description = "Default user in VMs"
  default     = "opensuse"
}

variable "control_planes" {
  description = "Number of CP VMs to create"
  default     = 1
}

variable "control_plane_memory" {
  description = "The amount of RAM (MB) for a CP node"
  default     = 2048
}

variable "control_plane_vcpu" {
  description = "The amount of virtual CPUs for a CP node"
  default     = 2
}

variable "control_plane_disk_size" {
  description = "Disk size (in bytes)"
  default     = "25769803776"
}

variable "workers" {
  description = "Number of worker VMs to create"
  default     = 1
}

variable "worker_memory" {
  description = "The amount of RAM (MB) for a worker node"
  default     = 2048
}

variable "worker_vcpu" {
  description = "The amount of virtual CPUs for a worker node"
  default     = 1
}

variable "worker_disk_size" {
  description = "Disk size (in bytes)"
  default     = "25769803776"
}
