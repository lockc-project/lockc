resource "openstack_networking_network_v2" "network" {
  name           = var.internal_net
  admin_state_up = "true"
}

resource "openstack_networking_subnet_v2" "subnet" {
  name       = var.internal_subnet == "" ? "${var.internal_net}-subnet" : var.internal_subnet
  network_id = openstack_networking_network_v2.network.id
  cidr       = var.subnet_cidr
  ip_version = 4
}

data "openstack_networking_network_v2" "external_network" {
  name = var.external_net
}

resource "openstack_networking_router_v2" "router" {
  name                = var.internal_router == "" ? "${var.internal_net}-router" : var.internal_router
  external_network_id = data.openstack_networking_network_v2.external_network.id
}

resource "openstack_networking_router_interface_v2" "router_interface" {
  router_id = openstack_networking_router_v2.router.id
  subnet_id = openstack_networking_subnet_v2.subnet.id
}

