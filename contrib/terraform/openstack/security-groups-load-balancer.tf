resource "openstack_networking_secgroup_v2" "load_balancer" {
  name        = "${var.stack_name}-k8s_lb_secgroup"
  description = "Common security group for K8s load balancer"
}

resource "openstack_networking_secgroup_rule_v2" "lb_api_server" {
  direction         = "ingress"
  ethertype         = "IPv4"
  protocol          = "tcp"
  port_range_min    = 6443
  port_range_max    = 6443
  remote_ip_prefix  = "0.0.0.0/0"
  security_group_id = openstack_networking_secgroup_v2.load_balancer.id
}
