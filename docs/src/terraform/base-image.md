#### Building the base image

The first step is to build the VM image.

```bash
cd contrib/guestfs
./build
```

If the script ran successfully, `lockc-base.qcow2` file should be present.
It cointains the base VM image which will be used by Terraform.
