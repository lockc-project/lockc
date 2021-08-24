## Terraform

There is also a possibility to run lockc in virtual machines with
Kubernetes.

In order to do that, ensure that you have the following software installed:

* libvirt
* guestfs-tools

Then we can proceed with following steps:

- **[Base image]** - The first step is to build the VM image
- **[Custom kernel]** *Optional* - building the image with a custom kernel
- **[Use libvirt]** - Configure and start VMs in libvirt environment
- **[Use OpenStack]** - Starting VMs in OpenStack environment

[Base image]: base-image.md
[Custom kernel]: custom-kernel.md
[Use libvirt]: libvirt.md
[Use OpenStack]: openstack.md

