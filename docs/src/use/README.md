## Using lockc locally

lockc can be used on the local system if you want to secure your local
container engine (docker, podman).

BPF programs can be loaded by executing `lockcd`:

First, we need to load BPF programs by running lockcd. That can be done
by the following command, if lockc was built inside a container:

```bash
sudo ./out/lockcd
```

or if lockc was build with Meson:

```bash
sudo ./build/src/lockcd
```

If you have `bpftool` available on your host, you canm check whether lockc
BPF programs are running. The correct output should look similar to:

```bash
sudo bpftool prog
[...]
25910: tracing  name sched_process_f  tag 3a6a6e4defce95ab  gpl
        loaded_at 2021-06-02T16:52:57+0200  uid 0
        xlated 2160B  jited 1137B  memlock 4096B  map_ids 14781,14782,14783
        btf_id 18711
25911: lsm  name clone_audit  tag fc30a5b3e6a4610b  gpl
        loaded_at 2021-06-02T16:52:57+0200  uid 0
        xlated 2280B  jited 1196B  memlock 4096B  map_ids 14781,14782,14783
        btf_id 18711
25912: lsm  name syslog_audit  tag 2cdd93e75fa0e936  gpl
        loaded_at 2021-06-02T16:52:57+0200  uid 0
        xlated 816B  jited 458B  memlock 4096B  map_ids 14783,14782
        btf_id 18711
```

To check if containers get "hardened" by lockc, check if you are able to
see the kernel logs from inside the container wrapped by **lockc-runc-wrapper**.
Example:

```bash
podman --runtime ./out/lockc-runc-wrapper run -ti --rm registry.opensuse.org/opensuse/toolbox:latest
a135dbc3ef08:/ # dmesg
dmesg: read kernel buffer failed: Operation not permitted
```

That error means that lockc works, applied the *baseline* policy on a new
container and prevented containerized processes from accessing kernel logs.

If`dmesg` ran successfully and shows the kernel logs, it means that something
went wrong and lockc is not working properly.

