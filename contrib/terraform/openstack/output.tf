output "ip_masters" {
  value = zipmap(
    concat(
      openstack_compute_instance_v2.master.*.name, [ for i in range(length(openstack_compute_instance_v2.master), var.masters) : format("master-%d (not provisioned)", i ) ]),
    concat(
      openstack_networking_floatingip_v2.master_ext.*.address,  [ for _ in range(length(openstack_networking_floatingip_v2.master_ext.*.address), var.masters) : "" ])
    )
}

#output "ip_workers" {
#  value = zipmap(
#    concat(
#      openstack_compute_instance_v2.worker.*.name, [ for i in range(length(openstack_compute_instance_v2.worker), var.workers) : format("worker-%d (not provisioned)", i ) ]),
#    concat(
#      openstack_networking_floatingip_v2.worker_ext.*.address,  [ for _ in range(length(openstack_networking_floatingip_v2.worker_ext.*.address), var.workers) : "" ])
#  )
#}

output "ip_internal_load_balancer" {
  value = openstack_lb_loadbalancer_v2.lb.vip_address
}

output "ip_load_balancer" {
  value = zipmap(
    length(openstack_lb_loadbalancer_v2.lb.*) == 1 ? list(openstack_lb_loadbalancer_v2.lb.name) : list("${var.stack_name}-lb"),
    length(openstack_networking_floatingip_v2.lb_ext.*) == 1 ? list(openstack_networking_floatingip_v2.lb_ext.address) :
      list("Could not obtain ip address, please retry terraform refresh to update the output")
  )
}
