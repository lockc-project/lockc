#### Configure libvirt

VMs which we are going to run are using 9p to mount the source tree. To
ensure that those mounts are going to work correctly, open the
`/etc/libvirt/qemu.conf` file and ensure that the following options
are present there:

```bash
user = "root"
group = "root"
dynamic_ownership = 0
```
If you had to edit the configuration, save the file and restart libvirt:

```bash
sudo systemctl restart libvirtd
```
