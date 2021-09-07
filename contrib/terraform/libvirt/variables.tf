variable "libvirt_uri" {
  description = "libvirt connection URI"
  default     = "qemu:///system"
}

variable "pool" {
  description = "Pool to be used to store all the volumes"
  default     = "default"
}

variable "image_name" {
  description = "Image name"
  default     = "lockc-image"
}

variable "image_path" {
  description = "Path or URL to the image"
  default     = "../../guestfs/lockc-base.qcow2"
}

variable "network_name" {
  description = "Network name"
  default     = "lockc-network"
}

variable "network_mode" {
  description = "Network mode"
  default     = "nat"
}

variable "dns_domain" {
  description = "DNS domain name"
  default     = "lockc.local"
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

variable "username" {
  description = "Default user on the nodes"
  default     = "opensuse"
}

variable "authorized_keys" {
  description = "SSH keys to inject into all the nodes"
  type        = list(string)
  default     = []
}

variable "kubeadm_token" {
  description = "Token for trust between kubeadm nodes"
  default     = "8c05f4.b5bdd4ceb5ce4d3f"
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
