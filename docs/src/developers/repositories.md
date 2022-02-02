# Repositories

lockc currently uses two git repositories:

* **[rancher-sandbox/lockc](https://github.com/rancher-sandbox/lockc)** - the
  main repository containing lockc source code
* **[rancher-sandbox/lockc-helm-charts](https://github.com/rancher-sandbox/lockc-helm-charts)** -
  repository with Helm charts to deploy lockc on Kubernetes

If you are interested in development and contributing to lockc, we recommend to
fork and clone both of them. Both will be needed especially for building a
[development environment based on Terraform](terraform/README.md).

The latter chapters assume that you have **lockc** and **lockc-helm-charts**
cloned in the same parent directory. For example, as
*$HOME/my-repositories/lockc* and *$HOME/my-repositories/lockc-helm-charts*.
