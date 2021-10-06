## Building lockc

The first step to try out lockc is to build it. There are two ways to do
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

[Dapper]: dapper.md
[Cargo]: cargo.md
