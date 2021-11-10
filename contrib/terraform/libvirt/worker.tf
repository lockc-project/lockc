resource "libvirt_volume" "worker" {
  name           = "lockc-worker-volume-${count.index}"
  base_volume_id = libvirt_volume.lockc_image.id
  size           = var.worker_disk_size
  count          = var.workers
}

data "template_file" "worker_repositories" {
  template = file("cloud-init/repository.tpl")
  count    = length(var.repositories)

  vars = {
    repository_url = element(values(var.repositories), count.index)
    repository_name = element(keys(var.repositories), count.index)
  }
}

data "template_file" "worker_commands" {
  template = file("cloud-init/commands.tpl")
  count    = length(var.packages)

  vars = {
    packages = join(", ", var.packages)
  }
}

data "template_file" "worker_cloud_init" {
  template = file("cloud-init/common.tpl")
  count    = var.workers

  vars = {
    hostname        = "lockc-worker-${count.index}"
    locale          = var.locale
    timezone        = var.timezone
    authorized_keys = join("\n", formatlist("      - %s", var.authorized_keys))
    repositories    = join("\n", data.template_file.worker_repositories.*.rendered)
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
  count      = var.workers
  name       = "lockc-worker-plane-${count.index}"
  memory     = var.worker_memory
  vcpu       = var.worker_vcpu
  cloudinit  = element(libvirt_cloudinit_disk.worker.*.id, count.index)

  cpu {
    mode = "host-passthrough"
  }

  disk {
    volume_id = element(libvirt_volume.worker.*.id, count.index)
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

resource "null_resource" "worker_wait_cloudinit" {
  depends_on = [libvirt_domain.worker]
  count      = var.workers

  connection {
    host = element(
      libvirt_domain.worker.*.network_interface.0.addresses.0,
      count.index
    )
    user = "opensuse"
    type = "ssh"
  }

  provisioner "remote-exec" {
    inline = [
      "cloud-init status --wait > /dev/null",
    ]
  }
}

resource "null_resource" "worker_provision" {
  depends_on = [null_resource.worker_wait_cloudinit]
  count      = var.workers

  connection {
    host = element(
      libvirt_domain.worker.*.network_interface.0.addresses.0,
      count.index
    )
    user = "opensuse"
    type = "ssh"
  }

  provisioner "remote-exec" {
    script = "provision.sh"
  }
}

resource "null_resource" "worker_provision_docker" {
  depends_on = [null_resource.worker_provision]
  count      = var.enable_docker ? var.workers: 0

  connection {
    host = element(
      libvirt_domain.worker.*.network_interface.0.addresses.0,
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

resource "null_resource" "worker_provision_k8s_containerd" {
  depends_on = [null_resource.worker_provision]
  count      = var.enable_k8s_containerd ? var.workers : 0

  connection {
    host = element(
      libvirt_domain.worker.*.network_interface.0.addresses.0,
      count.index
    )
    user = var.username
    type = "ssh"
  }

  provisioner "file" {
    source =      "../../../target/debug/lockc.tar.gz"
    destination = "/home/opensuse/lockc.tar.gz"
  }

  provisioner "remote-exec" {
    script = "provision-k8s-containerd.sh"
  }
}

resource "null_resource" "worker_reboot" {
  depends_on = [
    null_resource.worker_provision_docker,
    null_resource.worker_provision_k8s_containerd
  ]
  count      = var.workers

  provisioner "local-exec" {
    environment = {
      user = "opensuse"
      host = element(
        libvirt_domain.worker.*.network_interface.0.addresses.0,
        count.index
      )
    }

    command = <<EOT
export sshopts="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -oConnectionAttempts=60"
if ! ssh $sshopts $user@$host 'sudo needs-restarting -r'; then
    ssh $sshopts $user@$host sudo reboot || :
    export delay=5
    # # wait for node reboot completed
    # # lol, doesn't work
    # while ! ssh $sshopts $user@$host 'sudo needs-restarting -r'; do
    #     sleep $delay
    #     delay=$((delay+1))
    #     [ $delay -gt 60 ] && exit 1
    # done
fi
EOT
  }
}
