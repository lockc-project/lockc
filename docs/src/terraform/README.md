## Terraform

There is also a possibility to run lockc in virtual machines with
Kubernetes.

In order to do that, ensure that you have the following software installed:

* libvirt
* guestfs-tools

Then we can proceed with following steps:

- **[Base image]** - The first step is to build the VM image
- **[Custom kernel]** *Optional* - building the image with a custom kernel
- **[Configure libvirt]** - Configure libvirt environment
- **[Running VMs]** - Starting VMs

[Base image]: base-image.md
[Custom kernel]: custom-kernel.md
[Configure libvirt]: conf-libvirt.md
[Running VMs]: start-vm.md

