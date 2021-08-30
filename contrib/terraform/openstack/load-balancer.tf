resource "openstack_lb_loadbalancer_v2" "lb" {
  name          = "${var.stack_name}-lb"
  vip_subnet_id = openstack_networking_subnet_v2.subnet.id

  security_group_ids = [
    openstack_networking_secgroup_v2.load_balancer.id,
  ]
}

resource "openstack_lb_listener_v2" "kube_api_listener" {
  protocol        = "TCP"
  protocol_port   = "6443"
  loadbalancer_id = openstack_lb_loadbalancer_v2.lb.id
  name            = "${var.stack_name}-kube-api-listener"
}

resource "openstack_lb_pool_v2" "kube_api_pool" {
  name        = "${var.stack_name}-kube-api-pool"
  protocol    = "TCP"
  lb_method   = "ROUND_ROBIN"
  listener_id = openstack_lb_listener_v2.kube_api_listener.id
}

resource "openstack_lb_member_v2" "kube_api_member" {
  count   = var.masters
  pool_id = openstack_lb_pool_v2.kube_api_pool.id
  address = element(
    openstack_compute_instance_v2.master.*.access_ip_v4,
    count.index,
  )
  subnet_id     = openstack_networking_subnet_v2.subnet.id
  protocol_port = 6443
}

resource "openstack_networking_floatingip_v2" "lb_ext" {
  pool    = var.external_net
  port_id = openstack_lb_loadbalancer_v2.lb.vip_port_id
}

resource "openstack_lb_monitor_v2" "kube_api_monitor" {
  pool_id        = openstack_lb_pool_v2.kube_api_pool.id
  type           = "HTTPS"
  url_path       = "/healthz"
  expected_codes = 200
  delay          = 3
  timeout        = 1
  max_retries    = 1
}
