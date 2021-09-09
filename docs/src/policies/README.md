# Policies

lockc provides three policy levels for containers:

* **baseline** - meant for regular applications
* **restricted** - meant for applications for which we need to be more cautious
  and secure them more stricly
* **privileged** - meant for part of the infrastructure which can have full
  access to host resources - i.e. CNI plugins in Kubernetes

The default policy level is **baseline**. The policy level can be changed by
the `pod-security.kubernetes.io/enforce` label on the **namespace** which
the container is running in. We make an exception for the *kube-system*
namespace for which the **privileged** policy is applied by default.

For now there is no possibility to apply policy levels on local container
engines (Docker, containerd, podman), but such feature is planned in the
future.
