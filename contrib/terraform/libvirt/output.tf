output "ip_control_planes" {
  value = zipmap(
    libvirt_domain.control_plane.*.network_interface.0.hostname,
    libvirt_domain.control_plane.*.network_interface.0.addresses,
  )
}
