# libvirt

## Configure libvirt
  
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

## Running VMs

Now it's time to prepare Terraform environment. Terraform files are included in
the main lockc git repository.

```bash
cd contrib/terraform/libvirt
cp terraform.tfvars.json.example terraform.tfvars.json
```

After that, open the `terraform.tfvars.json` file with your favorite text
editor. The only setting which you really need to change is
`authorized_keys`. Please paste your public SSH key there. Otherwise,
connecting to VMs with SSH will be impossible.

By default, the `enable_k8s_containerd` option is `true` which means that
Kubernetes cluster with CRI containerd is going to be deployed on VMs.

If you want to rather use lockc with Docker as a local container engine, set
`enable_k8s_containerd` to `false` and set `enable_docker` to `true`. In that
case, it also makes sense to set `control_planes` to `1` and `workers` to `0`,
since one VM is enough for non-clusterized Docker.

Other customization available through `terraform.tfvars.json` are explained
further in this document.

After setting the options, initialize the environment with:

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
Apply complete! Resources: 31 added, 0 changed, 0 destroyed.

Outputs:

ip_control_planes = {
  "lockc-control-plane-0" = tolist([
    "10.16.0.113",
  ])
  "lockc-control-plane-1" = tolist([
    "10.16.0.137",
  ])
}
ip_workers = {
  "lockc-worker-0" = tolist([
    "10.16.0.174",
  ])
  "lockc-worker-1" = tolist([
    "10.16.0.44",
  ])
}

```

You can simply ssh to them using the `opensuse` user:

```bash
ssh opensuse@10.16.0.255
```

## Kubernetes

If you chose to deploy Kubernetes, there will be the `admin.conf` file
available after successful run of terraform. You can use it by doing:

```bash
export KUBECONFIG=$(pwd)/admin.conf
```

Then we can check whether Kubernetes is running properly:

```bash
$ kubectl get nodes
NAME                    STATUS   ROLES                  AGE   VERSION
lockc-control-plane-0   Ready    control-plane,master   15m   v1.22.3
lockc-control-plane-1   Ready    control-plane,master   14m   v1.22.3
lockc-worker-0          Ready    <none>                 13m   v1.22.3
lockc-worker-1          Ready    <none>                 13m   v1.22.3
$ kubectl get pods -A
NAMESPACE     NAME                                            READY   STATUS    RESTARTS      AGE
kube-system   cilium-6s8m6                                    1/1     Running   0             14m
kube-system   cilium-nnrvx                                    1/1     Running   0             14m
kube-system   cilium-operator-5986db558f-kj9w5                1/1     Running   0             15m
kube-system   cilium-operator-5986db558f-xddgx                1/1     Running   1 (14m ago)   15m
kube-system   cilium-tms8r                                    1/1     Running   1 (14m ago)   15m
kube-system   cilium-x5rtz                                    1/1     Running   0             14m
kube-system   coredns-78fcd69978-cppjl                        1/1     Running   0             14m
kube-system   coredns-78fcd69978-rn58p                        1/1     Running   0             14m
kube-system   etcd-lockc-control-plane-0                      1/1     Running   0             15m
kube-system   etcd-lockc-control-plane-1                      1/1     Running   0             14m
kube-system   kube-apiserver-lockc-control-plane-0            1/1     Running   0             15m
kube-system   kube-apiserver-lockc-control-plane-1            1/1     Running   0             14m
kube-system   kube-controller-manager-lockc-control-plane-0   1/1     Running   1 (14m ago)   15m
kube-system   kube-controller-manager-lockc-control-plane-1   1/1     Running   0             14m
kube-system   kube-proxy-42b5t                                1/1     Running   0             14m
kube-system   kube-proxy-4stzw                                1/1     Running   0             14m
kube-system   kube-proxy-7jtfb                                1/1     Running   0             14m
kube-system   kube-proxy-mhrxq                                1/1     Running   0             15m
kube-system   kube-scheduler-lockc-control-plane-0            1/1     Running   1 (14m ago)   15m
kube-system   kube-scheduler-lockc-control-plane-1            1/1     Running   0             14m
```

Now it's time to build and deploy lockc!

To build lockc container image, we have to go to the main directory in lockc
git repository:

```bash
# Go to the main directory of lockc sources
cd ../../..
export IMAGE_NAME=$(uuidgen)
docker build -t ttl.sh/${IMAGE_NAME}:30m .
docker push ttl.sh/${IMAGE_NAME}:30m
```

Then we need to go to the lockc-helm-charts git repository:

```bash
# Go to the main directory of lockc-helm-charts sources
cd ../lockc-helm-charts
helm install lockc charts/lockc/ --namespace kube-system \
    --set lockcd.image.repository=ttl.sh/${IMAGE_NAME} \
    --set lockcd.image.tag=30m \
    --set lockcd.debug.enabled=true
```

Then wait until the `lockcd` DaemonSet is ready:

```
$ kubectl -n kube-system get ds lockcd
NAME     DESIRED   CURRENT   READY   UP-TO-DATE   AVAILABLE   NODE SELECTOR   AGE
lockcd   4         4         4       4            4           <none>          42s
```

You can check further whether lockc is working properly by using example
deployments. Some of them (not violating policies) should deploy successfully.
The other ones (violating policies) should fail.

Let's start with creating namespaces which enforce particular policy levels:

```bash
$ cd examples/kubernetes
$ kubectl apply -f namespaces.yaml 
namespace/restricted created
namespace/baseline created
namespace/privileged created
```

Then let's deploy examples which should run successfully:

```bash
$ kubectl apply -f deployments-should-succeed.yaml 
deployment.apps/nginx-default-success created
deployment.apps/nginx-restricted-success created
deployment.apps/nginx-baseline-success created
deployment.apps/bpf-privileged-success created
$ kubectl get pods -A | grep success
baseline      nginx-baseline-success-8f5dd55f5-8cn9v          1/1     Running   0             17s
default       nginx-default-success-54df89d4ff-582dz          1/1     Running   0             17s
privileged    bpf-privileged-success-5f6b5975b6-hw6h5         1/1     Running   0             17s
restricted    nginx-restricted-success-57b757d5df-btr2s       1/1     Running   0             17s
```

And then examples which should fail:

```bash
$ kubectl apply -f deployments-should-fail.yaml 
deployment.apps/nginx-restricted-fail created
deployment.apps/bpf-default-fail created
deployment.apps/bpf-restricted-fail created
deployment.apps/bpf-baseline-fail created
$ kubectl get pods -A | grep fail
baseline      bpf-baseline-fail-756b89d76f-gzqzq              0/1     CrashLoopBackOff    5 (17s ago)   3m22s
default       bpf-default-fail-7d767947c7-dvjmt               0/1     RunContainerError   5 (9s ago)    3m22s
restricted    bpf-restricted-fail-7789944bc5-777tq            0/1     CrashLoopBackOff    5 (21s ago)   3m22s
restricted    nginx-restricted-fail-74fb56bb7-vllkx           0/1     ContainerCreating   0             3m22s
```

We can dig deeper to check for the reason of failure:

```bash
$ kubectl describe pod bpf-default-fail-7d767947c7-dvjmt
[...]
      Message:      failed to create containerd task: failed to create shim: OCI runtime create failed: container_linux.go:380: starting container process caused: process_linux.go:545: container init caused: rootfs_linux.go:76: mounting "/sys/fs/bpf" to rootfs at "/sys/fs/bpf" caused: mount through procfd: operation not permitted: unknown
[...]
```

After connecting to one of the VMs we can use bpftool to check lockc BPF
programs:

```bash
$ ssh opensuse@10.16.0.174
$ sudo -i
# bpftool prog list
[...]
567: tracing  name sched_process_f  tag fbda76511566c0af  gpl
	loaded_at 2021-11-17T14:49:02+0000  uid 0
	xlated 1528B  jited 869B  memlock 4096B  map_ids 108,107
	btf_id 125
568: lsm  name clone_audit  tag 5cca020a1dcf0cdf  gpl
	loaded_at 2021-11-17T14:49:02+0000  uid 0
	xlated 1600B  jited 899B  memlock 4096B  map_ids 108,107
	btf_id 125
569: tracing  name do_exit  tag 9ebd9eeef8cdbc3a  gpl
	loaded_at 2021-11-17T14:49:02+0000  uid 0
	xlated 488B  jited 285B  memlock 4096B  map_ids 108
	btf_id 125
570: lsm  name syslog_audit  tag 36b3cfb6f1722b24  gpl
	loaded_at 2021-11-17T14:49:02+0000  uid 0
	xlated 1112B  jited 629B  memlock 4096B  map_ids 108,107
	btf_id 125
571: lsm  name mount_audit  tag f972bdc0b55c76aa  gpl
	loaded_at 2021-11-17T14:49:02+0000  uid 0
	xlated 2832B  jited 1620B  memlock 4096B  map_ids 108,107,109,110
	btf_id 125
572: lsm  name open_audit  tag 9617c657c693030f  gpl
	loaded_at 2021-11-17T14:49:02+0000  uid 0
	xlated 2936B  jited 1678B  memlock 4096B  map_ids 108,107,113,114,111,112
	btf_id 125
573: kprobe  name add_container  tag 1bb54a2bc3b00693  gpl
	loaded_at 2021-11-17T14:49:02+0000  uid 0
	xlated 824B  jited 455B  memlock 4096B  map_ids 107,108
	btf_id 125
574: kprobe  name delete_containe  tag a6b9816741d0c7b3  gpl
	loaded_at 2021-11-17T14:49:02+0000  uid 0
	xlated 1104B  jited 639B  memlock 4096B  map_ids 107,108
	btf_id 125
575: kprobe  name add_process  tag ec39a14f8b9b9ccf  gpl
	loaded_at 2021-11-17T14:49:02+0000  uid 0
	xlated 480B  jited 272B  memlock 4096B  map_ids 108
	btf_id 125

[...]
```

And whether it registers containers. Directories inside
`/sys/fs/bpf/lockc` represent timestamps of lockcd launch, so it will be
different than in the following example.

```bash
# bpftool map dump pinned /sys/fs/bpf/lockc/1637160542/map_containers 
[{
        "key": 4257,
        "value": {
            "policy_level": "POLICY_LEVEL_BASELINE"
        }
    },{
        "key": 4780,
        "value": {
            "policy_level": "POLICY_LEVEL_BASELINE"
        }
    },{
        "key": 4664,
        "value": {
            "policy_level": "POLICY_LEVEL_RESTRICTED"
        }
    },{
        "key": 4589,
        "value": {
            "policy_level": "POLICY_LEVEL_BASELINE"
        }
    },{
        "key": 4422,
        "value": {
            "policy_level": "POLICY_LEVEL_RESTRICTED"
        }
    }
]
```

## Docker

Terraform output in case of Docker should look like:

```bash
Apply complete! Resources: 9 added, 0 changed, 0 destroyed.

Outputs:

ip_control_planes = {
  "lockc-control-plane-0" = tolist([
    "10.16.0.152",
  ])
}
ip_workers = {}
```

We can simply ssh to the VM:

```bash
ssh opensuse@10.16.0.152
```

And then try to run some insecure container:

```bash
$ docker run --rm -it -v /:/rootfs registry.opensuse.org/opensuse/toolbox:latest bash
docker: Error response from daemon: OCI runtime create failed: container_linux.go:380: starting container process caused: process_linux.go:545: container init caused: rootfs_linux.go:76: mounting "/" to rootfs at "/rootfs" caused: mount through procfd: operation not permitted: unknown.
```

lockc should prevent that container from running. However, running such
container as root with `privileged` policy level should be fine:

```bash
$ sudo -i
# docker run --label org.lockc.policy=privileged --rm -it -v /:/rootfs registry.opensuse.org/opensuse/toolbox:latest bash
8ea310609fce:/ # 
```

## Variables

Variables available in `terraform.tfvars.json` are:

* `libvirt_uri` - libvirt connection URI
* `pool` - pool to be used to store all the volumes
* `image_name` - image name in libvirt
* `image_path` - path or URL to the image
* `network_name` - network name in libvirt
* `network_mode` - network mode in libvirt (`nat` / `none` / `route` / `bridge`)
* `dns_domain` - DNS domain name
* `stack_name` - identifier to make all your resources unique and avoid clashes with other users of this terraform project
* `network_cidr` - network CIDR
* `locale` - system locales to set onm all the nodes
* `timezone` - timezone to set on all the nodes
* `authorized_keys` - SSH keys to inject into all the nodes
* `repositories` - zypper repositories to add
* `packages` - list of additional packages to install#
* `enable_docker` - enable Docker support (as a non-clustered container engine
* `enable_k8s_containerd` - enable Kubernetes with containerd CRI
* `username` - default user in VMs
* `control_planes` - number of control plane VMs to create
* `control_plane_memory` - the amount of RAM (MB) for a control plane node
* `control_plane_vcpu` - the amount of virtual CPUs for a control plane node
* `control_plane_disk_size` - disk size (in bytes)
* `workers` - number of worker VMs to create
* `worker_memory` - the amount of RAM (MB) for a worker node
* `worker_vcpu` - the amount of virtual CPUs for a worker node
* `worker_disk_size` - disk size (in bytes)
