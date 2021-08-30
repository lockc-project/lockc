variable "image_name" {
  default     = ""
  description = "Name of the image to use"
}

variable "repositories" {
  type        = map(string)
  default     = {}
  description = "Urls of the repositories to mount via cloud-init"
}

variable "internal_net" {
  default     = ""
  description = "Name of the internal network to be created"
}

variable "internal_subnet" {
  default     = ""
  description = "Name of the internal subnet to be created"
}

variable "internal_router" {
  default     = ""
  description = "Name of the internal router to be created"
}

variable "subnet_cidr" {
  default     = ""
  description = "CIDR of the subnet for the internal network"
}

variable "external_net" {
  default     = ""
  description = "Name of the external network to be used, the one used to allocate floating IPs"
}

variable "master_size" {
  default     = "m1.medium"
  description = "Size of the master nodes"
}

variable "masters" {
  default     = 1
  description = "Number of master nodes"
}

variable "worker_size" {
  default     = "m1.medium"
  description = "Size of the worker nodes"
}

variable "workers" {
  default     = 2
  description = "Number of worker nodes"
}

variable "workers_vol_enabled" {
  default     = false
  description = "Attach persistent volumes to workers"
}

variable "workers_vol_size" {
  default     = 5
  description = "size of the volumes in GB"
}

variable "dnsdomain" {
  default     = ""
  description = "Name of DNS domain"
}

variable "dnsentry" {
  default     = false
  description = "DNS Entry"
}

variable "stack_name" {
  default     = ""
  description = "Identifier to make all your resources unique and avoid clashes with other users of this terraform project"
}

variable "authorized_keys" {
  type        = list(string)
  default     = []
  description = "SSH keys to inject into all the nodes"
}

variable "key_pair" {
  default     = ""
  description = "SSH key stored in openstack to create the nodes with"
}

variable "ntp_servers" {
  type        = list(string)
  default     = []
  description = "List of ntp servers to configure"
}

variable "packages" {
  type = list(string)

  default = [
    "kernel-default",
    "-kernel-default-base",
  ]

  description = "list of additional packages to install"
}

variable "username" {
  default     = "opensuse"
  description = "Default user for the cluster nodes created by cloud-init default configuration for openSUSE"
}

variable "password" {
  default     = "opensuse"
  description = "Default password for the cluster nodes created by cloud-init default configuration for openSUSE"
}

variable "ca_file" {
  default     = ""
  description = "Used to specify the path to your custom CA file"
}

variable "hostname_from_dhcp" {
  default     = true
  description = "Set node's hostname from DHCP server"
}
