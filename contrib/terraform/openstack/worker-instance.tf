data "template_file" "worker_repositories" {
  template = file("cloud-init/repository.tpl")
  count    = length(var.repositories)

  vars = {
    repository_url  = element(values(var.repositories), count.index)
    repository_name = element(keys(var.repositories), count.index)
  }
}

data "template_file" "worker_commands" {
  template = file("cloud-init/commands.tpl")
  count    = join("", var.packages) == "" ? 0 : 1

  vars = {
    packages = join(", ", var.packages)
  }
}

data "template_file" "worker-cloud-init" {
  template = file("cloud-init/common.tpl")
  count    = var.workers

  vars = {
    authorized_keys    = join("\n", formatlist("  - %s", var.authorized_keys))
    repositories       = join("\n", data.template_file.worker_repositories.*.rendered)
    commands           = join("\n", data.template_file.worker_commands.*.rendered)
    username           = var.username
    ntp_servers        = join("\n", formatlist("    - %s", var.ntp_servers))
    hostname           = "${var.stack_name}-k8s-worker${count.index}"
    hostname_from_dhcp = var.hostname_from_dhcp
  }
}

resource "openstack_blockstorage_volume_v2" "worker_vol" {
  count = var.workers_vol_enabled ? var.workers : 0
  size  = var.workers_vol_size
  name  = "vol_${element(openstack_compute_instance_v2.worker.*.name, count.index)}"
}

resource "openstack_compute_volume_attach_v2" "worker_vol_attach" {
  count       = var.workers_vol_enabled ? var.workers : 0
  instance_id = element(openstack_compute_instance_v2.worker.*.id, count.index)
  volume_id = element(
    openstack_blockstorage_volume_v2.worker_vol.*.id,
    count.index,
  )
}

resource "openstack_compute_instance_v2" "worker" {
  count      = var.workers
  name       = "${var.stack_name}-k8s-worker${count.index}"
  image_name = var.image_name
  key_pair   = var.key_pair

  depends_on = [
    openstack_networking_network_v2.network,
    openstack_networking_subnet_v2.subnet,
  ]

  flavor_name = var.worker_size

  network {
    name = var.internal_net
  }

  security_groups = [
    openstack_networking_secgroup_v2.common.id,
  ]

  user_data = data.template_file.worker-cloud-init[count.index].rendered
}

resource "openstack_networking_floatingip_v2" "worker_ext" {
  count = var.workers
  pool  = var.external_net
}

resource "openstack_compute_floatingip_associate_v2" "worker_ext_ip" {
  depends_on = [openstack_compute_instance_v2.worker]
  count      = var.workers
  floating_ip = element(
    openstack_networking_floatingip_v2.worker_ext.*.address,
    count.index,
  )
  instance_id = element(openstack_compute_instance_v2.worker.*.id, count.index)
}

resource "null_resource" "worker_wait_cloudinit" {
  depends_on = [
    openstack_compute_instance_v2.worker,
    openstack_compute_floatingip_associate_v2.worker_ext_ip,
  ]
  count = var.workers

  connection {
    host = element(
      openstack_compute_floatingip_associate_v2.worker_ext_ip.*.floating_ip,
      count.index,
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

resource "null_resource" "worker_reboot" {
  depends_on = [null_resource.worker_wait_cloudinit]
  count      = var.workers

  provisioner "local-exec" {
    environment = {
      user = var.username
      host = element(
        openstack_compute_floatingip_associate_v2.worker_ext_ip.*.floating_ip,
        count.index,
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
        [ $delay -gt 30 ] && exit 1
    done
fi
EOT
  }
}
