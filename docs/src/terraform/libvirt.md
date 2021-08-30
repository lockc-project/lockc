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

#### Running VMs

Now it's time to prepare Terraform environment.

```bash
cd contrib/terraform/libvirt
cp terraform.tfvars.json.example terraform.tfvars.json
```

After that, open the `terraform.tfvars.json` file with your favorite text
editor. The only setting which you really need to change is
`authorized_keys`. Please paste your public SSH key there. Otherwise,
connecting to VMs with SSH will be impossible.

Initialize the environment with:

```bash
terraform init
```

And then start the VMs:

```bash
terraform apply
```

 Terraform finished successfully, you should see the output with IP
addresses of virtual machines, like:

```bash
ip_control_planes = {
  "lockc-control-plane-0" = "10.16.0.225"
}
```

You can simply ssh to them using the `opensuse` user:

```bash
ssh opensuse@10.16.0.255
```
Inside the VM we can check whether Kubernetes is running:

```bash
# kubectl get pods -A
NAMESPACE     NAME                                            READY
STATUS    RESTARTS   AGE
kube-system   coredns-78fcd69978-lvshz                        0/1
Running   0          7s
kube-system   coredns-78fcd69978-q874s                        0/1
Running   0          7s
kube-system   etcd-lockc-control-plane-0                      1/1
Running   0          11s
kube-system   kube-apiserver-lockc-control-plane-0            1/1
Running   0          10s
kube-system   kube-controller-manager-lockc-control-plane-0   1/1
Running   0          11s
kube-system   kube-proxy-p7nrd                                1/1
Running   0          7s
kube-system   kube-scheduler-lockc-control-plane-0            1/1
Running   0          11s
```

And whether lockc is running. The main service can be checked by:

```bash
systemctl status lockcd
```

We can check also whether lockc's BPF programs are running:

```bash
# bpftool prog list
35: tracing  name sched_process_f  tag b3c2c2a08effc879  gpl
      loaded_at 2021-08-10T12:23:55+0000  uid 0
      xlated 1528B  jited 869B  memlock 4096B  map_ids 3,2
      btf_id 95
36: lsm  name clone_audit  tag 33a5e8a5da485fd4  gpl
      loaded_at 2021-08-10T12:23:55+0000  uid 0
      xlated 1600B  jited 899B  memlock 4096B  map_ids 3,2
      btf_id 95
37: lsm  name syslog_audit  tag 80d655f557922055  gpl
      loaded_at 2021-08-10T12:23:55+0000  uid 0
      xlated 1264B  jited 714B  memlock 4096B  map_ids 3,2
      btf_id 95
[...]
```

And whether it registers containers. Directories inside
`/sys/fs/bpf/lockc` represent timestamps of lockcd launch, so it will be
different than in the following example.

```bash
# bpftool map dump pinned /sys/fs/bpf/lockc/1628598193/map_containers
[{
        "key": 4506,
        "value": {
            "policy_level": "POLICY_LEVEL_PRIVILEGED"
        }
[...]
```
