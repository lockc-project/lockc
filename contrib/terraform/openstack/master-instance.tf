data "template_file" "master_repositories" {
  template = file("cloud-init/repository.tpl")
  count    = length(var.repositories)

  vars = {
    repository_url  = element(values(var.repositories), count.index)
    repository_name = element(keys(var.repositories), count.index)
  }
}

data "template_file" "master_commands" {
  template = file("cloud-init/commands.tpl")
  count    = join("", var.packages) == "" ? 0 : 1

  vars = {
    packages = join(", ", var.packages)
  }
}

data "template_file" "master-cloud-init" {
  template = file("cloud-init/common.tpl")
  count    = var.masters

  vars = {
    authorized_keys    = join("\n", formatlist("  - %s", var.authorized_keys))
    repositories       = join("\n", data.template_file.master_repositories.*.rendered)
    commands           = join("\n", data.template_file.master_commands.*.rendered)
    username           = var.username
    ntp_servers        = join("\n", formatlist("    - %s", var.ntp_servers))
    hostname           = "${var.stack_name}-k8s-master${count.index}"
    hostname_from_dhcp = var.hostname_from_dhcp
  }
}

resource "openstack_compute_instance_v2" "master" {
  count      = var.masters
  name       = "${var.stack_name}-k8s-master${count.index}"
  image_name = var.image_name
  key_pair   = var.key_pair

  depends_on = [
    openstack_networking_network_v2.network,
    openstack_networking_subnet_v2.subnet,
  ]

  flavor_name = var.master_size

  network {
    name = var.internal_net
  }

  security_groups = [
    openstack_networking_secgroup_v2.common.id,
    openstack_networking_secgroup_v2.master_nodes.id,
  ]

  user_data = data.template_file.master-cloud-init[count.index].rendered
}

resource "openstack_networking_floatingip_v2" "master_ext" {
  count = var.masters
  pool  = var.external_net
}

resource "openstack_compute_floatingip_associate_v2" "master_ext_ip" {
  depends_on = [openstack_compute_instance_v2.master]
  count      = var.masters
  floating_ip = element(
    openstack_networking_floatingip_v2.master_ext.*.address,
    count.index,
  )
  instance_id = element(openstack_compute_instance_v2.master.*.id, count.index)
}

resource "null_resource" "master_wait_cloudinit" {
  depends_on = [
    openstack_compute_instance_v2.master,
    openstack_compute_floatingip_associate_v2.master_ext_ip
  ]
  count = var.masters

  connection {
    host = element(
      openstack_compute_floatingip_associate_v2.master_ext_ip.*.floating_ip,
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

resource "null_resource" "master_reboot" {
  depends_on = [null_resource.master_wait_cloudinit]
  count      = var.masters

  provisioner "local-exec" {
    environment = {
      user = var.username
      host = element(
        openstack_compute_floatingip_associate_v2.master_ext_ip.*.floating_ip,
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
