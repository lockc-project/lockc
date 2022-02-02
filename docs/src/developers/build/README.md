# Building lockc

The first step to try out lockc is to build it. There are several ways to do
that:

* **[Dapper]** - build binaries within container
  * doesn't require any dependencies on the host system (except Docker)
  * ensures that Rust, Cargo and all dependencies are in the newest version
  * ensures the same behavior as on CI, aims to reduce "it worked on machine"
    kind of problems
  * recommended for
    * trying out the project (especially if there is no interest in changing
      the code)
    * final build before submitting changes in the code
* **[Cargo]** - build binaries with Cargo (Rust build system) on the host
  * convenient for local development, IDE/editor integration
* **[Container image]** - build a container image which can be deployed on
  Kubernetes
  * the only method to try lockc on Kubernetes
  * doesn't work for Docker integration, where we rather install lockc as a
    binary on the host, managed by systemd

[Dapper]: dapper.md
[Cargo]: cargo.md
[Container image]: container-image.md
