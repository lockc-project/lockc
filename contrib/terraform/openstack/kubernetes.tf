resource "null_resource" "deploy_kubernetes" {
  depends_on = [
    null_resource.master_reboot,
    null_resource.worker_reboot,
  ]
  count = 1

  provisioner "local-exec" {
    environment = {
      TR_STACK             = var.stack_name
      TR_USERNAME          = var.username
      TR_LB_IP             = openstack_networking_floatingip_v2.lb_ext.address
      TR_MASTER_IPS        = join(" ", openstack_networking_floatingip_v2.master_ext.*.address)
      TR_WORKER_IPS        = join(" ", openstack_networking_floatingip_v2.worker_ext.*.address)
    }

    command = "bash deploy-kubernetes.sh"
  }
}

