## Mount policies

### Kubernetes

The following demo shows mount policies being enforced on Kubernetes pods.

YAML files can be found [here](https://github.com/rancher-sandbox/lockc/tree/main/examples/kubernetes).

The policy violations in [deployments-should-fail.yaml](https://github.com/rancher-sandbox/lockc/tree/main/examples/kubernetes/deployments-should-fail.yaml)
file are:

- *nginx-restricted-fail* deployment trying to make a host mount while having a
  **restricted** policy
- *bpf-default-fail* and *bpf-baseline-fail* deployment trying to mount
  `/sys/fs/bpf` while having a **baseline** policy
- *bpf-restricted-fail* trying to mount `/sys/fs/bpf` while having a
  **restricted** policy.

[![asciicast](https://asciinema.org/a/sUxMMB5BKkJzlF1jP6k8Bxab3.svg)](https://asciinema.org/a/sUxMMB5BKkJzlF1jP6k8Bxab3)
