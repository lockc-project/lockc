resource "libvirt_volume" "worker" {
  name           = "lockc-worker-volume-${count.index}"
  base_volume_id = libvirt_volume.lockc_image.id
  count          = var.workers
}

data "template_file" "worker_commands" {
  template = file("cloud-init/worker.tpl")

  vars = {
    control_plane_ip = length(libvirt_domain.control_plane.0.network_interface.0.addresses) == 0 ? "" : libvirt_domain.control_plane.0.network_interface.0.addresses.0
    kubeadm_token    = var.kubeadm_token
  }
}

data "template_file" "worker_cloud_init" {
  template = file("cloud-init/common.tpl")
  count    = var.workers

  vars = {
    hostname        = "lockc-worker-${count.index}"
    locale          = var.locale
    timezone        = var.timezone
    username        = var.username
    authorized_keys = join("\n", formatlist("      - %s", var.authorized_keys))
    commands        = join("\n", data.template_file.worker_commands.*.rendered)
  }
}

resource "libvirt_cloudinit_disk" "worker" {
  count     = var.workers
  name      = "lockc-worker-cloudinit-disk-${count.index}"
  pool      = var.pool
  user_data = data.template_file.worker_cloud_init[count.index].rendered
}

resource "libvirt_domain" "worker" {
  count     = var.workers
  name      = "lockc-worker-${count.index}"
  memory    = var.control_plane_memory
  vcpu      = var.control_plane_vcpu
  cloudinit = element(libvirt_cloudinit_disk.worker.*.id, count.index)

  cpu {
    mode = "host-passthrough"
  }

  disk {
    volume_id = element(libvirt_volume.worker.*.id, count.index)
  }

  # Mount the source code.
  filesystem {
    source   = "${path.cwd}/../../.."
    target   = "lockc"
    readonly = false
  }

  network_interface {
    network_name   = var.network_name
    hostname       = "lockc-worker-${count.index}"
    wait_for_lease = true
  }

  graphics {
    type        = "vnc"
    listen_type = "address"
  }
}
