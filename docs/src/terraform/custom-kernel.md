#### (Optional) building the image with a custom kernel

The `build.sh` script can be also used to create a VM image with a
custom kernel if there is a need for kernel testing. You can optionally
provide a path to your kernel source tree. Please note that the kernel
should be already build on the host with `make`. Our guestfs scripts do
only `make modules_install install` to install the kernel image and
modules inside a VM. Installing the custom kernel is enabled by using
the `CUSTOM_KERNEL` environment variable. Its value has to be set to
`true`. By default, the script assumes that your kernel tree is in
`~/linux` directory. You can provide a custom path by another
environment variable - `KERNEL_SOURCE`. Examples of usage:

```bash
CUSTOM_KERNEL=true ./build.sh
CUSTOM_KERNEL=true KERNEL_SOURCE=${HOME}/my-git-repos/linux ./build.sh
```
If you already used `build.sh` once and you would like to inject a
custom kernel into already build qcow2 image, there is a separate script
- `reinstall-custom-kernel.sh`. It takes an optional `KERNEL_SOURCE`
environment variable. Examples of usage:

```bash
./reinstall-custom-kernel.sh
KERNEL_SOURCE=${HOME}/my-git-repos/linux ./reinstall-custom-kernel.sh
```
