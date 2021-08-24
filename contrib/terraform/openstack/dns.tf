resource "openstack_dns_zone_v2" "ag" {
  count       = var.dnsentry ? 1 : 0
  name        = "${var.dnsdomain}."
  email       = "email@example.com"
  description = "DNS zone"
  ttl         = 60
  type        = "PRIMARY"
}

resource "openstack_dns_recordset_v2" "master" {
  count   = var.dnsentry ? var.masters : 0
  zone_id = openstack_dns_zone_v2.ag[0].id
  name = format(
    "%v.%v.",
    element(openstack_compute_instance_v2.master.*.name, count.index),
    var.dnsdomain,
  )
  description = "master nodes A recordset"
  ttl         = 5
  type        = "A"
  records = [element(
    openstack_networking_floatingip_v2.master_ext.*.address,
    count.index,
  )]
  depends_on = [
    openstack_compute_instance_v2.master,
    openstack_compute_floatingip_associate_v2.master_ext_ip,
  ]
}

