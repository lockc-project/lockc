resource "libvirt_volume" "control_plane" {
  name           = "lockc-control-plane-volume-${count.index}"
  base_volume_id = libvirt_volume.lockc_image.id
  size           = var.control_plane_disk_size
  count          = var.control_planes
}

data "template_file" "control_plane_repositories" {
  template = file("cloud-init/repository.tpl")
  count    = length(var.repositories)

  vars = {
    repository_url  = element(values(var.repositories), count.index)
    repository_name = element(keys(var.repositories), count.index)
  }
}

data "template_file" "control_plane_commands" {
  template = file("cloud-init/commands.tpl")
  count    = length(var.packages)

  vars = {
    packages = join(", ", var.packages)
  }
}

data "template_file" "control_plane_cloud_init" {
  template = file("cloud-init/common.tpl")
  count    = var.control_planes

  vars = {
    hostname        = "lockc-control-plane-${count.index}"
    locale          = var.locale
    timezone        = var.timezone
    authorized_keys = join("\n", formatlist("      - %s", var.authorized_keys))
    repositories    = join("\n", data.template_file.control_plane_repositories.*.rendered)
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

  cpu {
    mode = "host-passthrough"
  }

  disk {
    volume_id = element(libvirt_volume.control_plane.*.id, count.index)
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

resource "null_resource" "control_plane_wait_cloudinit" {
  depends_on = [libvirt_domain.control_plane]
  count      = var.control_planes

  connection {
    host = element(
      libvirt_domain.control_plane.*.network_interface.0.addresses.0,
      count.index
    )
    user = var.username
    type = "ssh"
  }

  provisioner "remote-exec" {
    inline = [
      "cloud-init status --wait > /dev/null",
    ]
  }
}

resource "null_resource" "control_plane_provision" {
  depends_on = [null_resource.control_plane_wait_cloudinit]
  count      = var.control_planes

  connection {
    host = element(
      libvirt_domain.control_plane.*.network_interface.0.addresses.0,
      count.index
    )
    user = var.username
    type = "ssh"
  }

  provisioner "remote-exec" {
    script = "provision.sh"
  }
}

resource "null_resource" "control_plane_provision_docker" {
  depends_on = [null_resource.control_plane_provision]
  count      = var.enable_docker ? var.control_planes : 0

  connection {
    host = element(
      libvirt_domain.control_plane.*.network_interface.0.addresses.0,
      count.index
    )
    user = var.username
    type = "ssh"
  }

  provisioner "file" {
    source      = "../../../target/debug/lockc.tar.gz"
    destination = "/home/opensuse/lockc.tar.gz"
  }

  provisioner "remote-exec" {
    script = "provision-docker.sh"
  }
}

resource "null_resource" "control_plane_provision_k8s_containerd" {
  depends_on = [null_resource.control_plane_provision]
  count      = var.enable_k8s_containerd ? var.control_planes : 0

  connection {
    host = element(
      libvirt_domain.control_plane.*.network_interface.0.addresses.0,
      count.index
    )
    user = var.username
    type = "ssh"
  }

  provisioner "remote-exec" {
    script = "provision-k8s-containerd.sh"
  }

  provisioner "remote-exec" {
    script = "provision-k8s-containerd-cp.sh"
  }
}

resource "null_resource" "control_plane_reboot" {
  depends_on = [
    null_resource.control_plane_provision_docker,
    null_resource.control_plane_provision_k8s_containerd,
  ]
  count      = var.control_planes

  provisioner "local-exec" {
    environment = {
      user = var.username
      host = element(
        libvirt_domain.control_plane.*.network_interface.0.addresses.0,
        count.index
      )
    }

    command = <<EOT
export sshopts="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -oConnectionAttempts=60"
if ! ssh $sshopts $user@$host 'sudo needs-restarting -r'; then
    ssh $sshopts $user@$host sudo reboot || :
    export delay=5
    # wait for node reboot completed
    while ! ssh $sshopts $user@$host 'sudo needs-restarting -r'; do
        sleep $delay
        delay=$((delay+1))
        [ $delay -gt 60 ] && exit 1
        ssh $sshopts $user@$host 'sudo needs-restarting -r'
    done
fi
EOT
  }
}
