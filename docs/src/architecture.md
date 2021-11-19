# Architecture

The project consists of 3 parts:

- the set of BPF programs (written in C)
  - programs for monitoring processes, which detects whether new processes
    are running inside any container, which means applying policies on them
  - programs attached to particular LSM hooks, which allow or deny actions
    based on the policy applied to the container (currently all containers have
    the `baseline` policy applied, the mechanism of differentiating between
    policies per container/pod is yet to be implemented)
- **lockcd** - the userspace program (written in Rust)
  - loads the BPF programs into the kernel, pins them in BPFFS
  - monitors runc processes, registers new containers and determines which
    policy should be applied to a container
  - in future, it's going to serve as the configuration manager and log
    collector
