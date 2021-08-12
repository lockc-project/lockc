resource "libvirt_volume" "control_plane" {
  name           = "lockc-control-plane-volume-${count.index}"
  base_volume_id = libvirt_volume.lockc_image.id
  count          = var.control_planes
}


data "template_file" "control_plane_commands" {
  template = file("cloud-init/control-plane.tpl")
}

data "template_file" "control_plane_cloud_init" {
  template = file("cloud-init/common.tpl")
  count    = var.control_planes

  vars = {
    hostname        = "lockc-control-plane-${count.index}"
    locale          = var.locale
    timezone        = var.timezone
    authorized_keys = join("\n", formatlist("      - %s", var.authorized_keys))
    commands        = join("\n", data.template_file.control_plane_commands.*.rendered)
  }
}

resource "libvirt_cloudinit_disk" "control_plane" {
  count     = var.control_planes
  name      = "lockc-control-plane-cloudinit-disk-${count.index}"
  pool      = var.pool
  user_data = data.template_file.control_plane_cloud_init[count.index].rendered
}

resource "libvirt_domain" "control_plane" {
  count      = var.control_planes
  name       = "lockc-control-plane-${count.index}"
  memory     = var.control_plane_memory
  vcpu       = var.control_plane_vcpu
  cloudinit  = element(libvirt_cloudinit_disk.control_plane.*.id, count.index)

  cpu = {
    mode = "host-passthrough"
  }

  disk {
    volume_id = element(libvirt_volume.control_plane.*.id, count.index)
  }

  # Mount the source code.
  filesystem {
    source     = "${path.cwd}/../../.."
    target     = "lockc"
    readonly   = false
  }

  network_interface {
    network_name   = var.network_name
    hostname       = "lockc-control-plane-${count.index}"
    wait_for_lease = true
  }

  graphics {
    type        = "vnc"
    listen_type = "address"
  }
}
