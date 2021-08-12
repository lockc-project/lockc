//! lockc is open source software for providing MAC (Mandatory Access Control)
//! type of security audit for container workloads.
//!
//! The main technology behind lockc is [eBPF](https://ebpf.io/) - to be more
//! precise, its ability to attach to [LSM hooks](https://www.kernel.org/doc/html/latest/bpf/bpf_lsm.html)
//!
//! Please note that currently lockc is an experimental project, not meant for
//! production environments. Currently we don't publish any official binaries or
//! packages to use, except of a Rust crate. Currently the most convenient way
//! to use it is to use the source code and follow the guide.
//!
//! If you need help or want to talk with contributors, please come chat with
//! us on `#lockc` channel on the [Rust Cloud Native Discord server](https://discord.gg/799cmsYB4q).
//!
//! ## Architecture
//!
//! The project consists of 3 parts:
//!
//! * the set of BPF programs (written in C)
//!   * programs for monitoring processes, which detects whether new processes
//!     are running inside any container, which means applying policies on them
//!   * programs attached to particular LSM hooks, which allow or deny actions
//!     based on the policy applied to the container (currently all containers have
//!     the `baseline` policy applied, the mechanism of differentiating between
//!     policies per container/pod is yet to be implemented)
//! * **lockcd** - the userspace program (written in Rust)
//!   * loads the BPF programs into the kernel, pins them in BPFFS
//!   * in future, it's going to serve as the configuration manager and log
//!     collector
//! * **lockc-runc-wrapper** - a wraper for runc which registers new containers
//!   and determines which policy should be applied on a container
//!
//! ## Getting started
//!
//! ### Building lockc
//!
//! The first step to try out lockc is to build it. There are two ways to do
//! that.
//!
//! #### Containerized build
//!
//! One option for building lockc is using the `containerized-build.sh` script
//! to perform the build inside container, without installing needed
//! dependencies on the host system.
//!
//! This guide assumes that you have `docker` or any other container engine
//! installed.
//!
//! The build can be performed by running the following command:
//!
//! ```bash
//! ./containerized-build.sh build
//! ```
//!
//! or simply:
//!
//! ```bash
//! ./containerized-build.sh
//! ```
//!
//! `build` is the default subcommand of `containerized-build`. There are
//! several other subcommands:
//!
//! ```bash
//! $ ./containerized-build.sh help
//! Usage: containerized-build.sh <subcommand>
//!
//! Subcommands:
//!     gen        Compile BPF programs and generate BPF CO-RE skeleton
//!     build      Build lockc
//!     install    Install lockc
//!     fmt        Autoformat code
//!     lint       Code analysis
//!     help       Show help information
//! ```
//!
//! For following this guide, using the `build` subcommand is enough.
//!
//! `./containerized-build.sh install` can be used to install
//! lockc in your host system, which by default means directories like
//! `/usr/bin`, `/etc`. Target directories can be customized by `DESTDIR`,
//! `PREFIX`, `BINDIR`, `UNITDIR` and `SYSCONFDIR` environment variables.
//!
//! `build` should result in binaries produced in the `out/` directory:
//!
//! ```bash
//! $ ls out/
//! lockcd  lockc-runc-wrapper
//! ```
//!
//! #### Meson
//!
//! If you are comfortable with installing all dependencies on your host
//! system, you need to install the following software:
//!
//! * meson
//! * rust, cargo
//! * llvm, clang
//! * libbpf
//! * bpftool
//!
//! Build can be performed by the following commands:
//!
//! ```bash
//! CC=clang meson build
//! cd build
//! meson compile
//! ```
//!
//! Installation can be perfomed by:
//!
//! ```bash
//! meson install
//! ```
//!
//! ### Using lockc locally
//!
//! lockc can be used on the local system if you want to secure your local
//! container engine (docker, podman).
//!
//! BPF programs can be loaded by executing `lockcd`:
//!
//! First, we need to load BPF programs by running lockcd. That can be done
//! by the following command, if lockc was built inside a container:
//!
//! ```bash
//! sudo ./out/lockcd
//! ```
//!
//! or if lockc was build with Meson:
//!
//! ```bash
//! sudo ./build/src/lockcd
//! ```
//!
//! If you have `bpftool` available on your host, you canm check whether lockc
//! BPF programs are running. The correct output should look similar to:
//!
//! ```bash
//! $ sudo bpftool prog
//! [...]
//! 25910: tracing  name sched_process_f  tag 3a6a6e4defce95ab  gpl
//!         loaded_at 2021-06-02T16:52:57+0200  uid 0
//!         xlated 2160B  jited 1137B  memlock 4096B  map_ids 14781,14782,14783
//!         btf_id 18711
//! 25911: lsm  name clone_audit  tag fc30a5b3e6a4610b  gpl
//!         loaded_at 2021-06-02T16:52:57+0200  uid 0
//!         xlated 2280B  jited 1196B  memlock 4096B  map_ids 14781,14782,14783
//!         btf_id 18711
//! 25912: lsm  name syslog_audit  tag 2cdd93e75fa0e936  gpl
//!         loaded_at 2021-06-02T16:52:57+0200  uid 0
//!         xlated 816B  jited 458B  memlock 4096B  map_ids 14783,14782
//!         btf_id 18711
//! ```
//!
//! To check if containers get "hardened" by lockc, check if you are able to
//! see the kernel logs from inside the container wrapped by **lockc-runc-wrapper**.
//! Example:
//!
//! ```bash
//! $ podman --runtime ./out/lockc-runc-wrapper run -ti --rm registry.opensuse.org/opensuse/toolbox:latest
//! a135dbc3ef08:/ # dmesg
//! dmesg: read kernel buffer failed: Operation not permitted
//! ```
//!
//! That error means that lockc works, applied the *baseline* policy on a new
//! container and prevented containerized processes from accessing kernel logs.
//!
//! If `dmesg` ran successfully and shows the kernel logs, it means that something
//! went wrong and lockc is not working properly.
//!
//! ### Terraform
//!
//! There is also a possibility to run lockc in virtual machines with
//! Kubernetes.
//!
//! In order to do that, ensure that you have the following software installed:
//!
//! * libvirt
//! * guestfs-tools
//!
//! #### Building the base image
//!
//! The first step is to build the VM image.
//!
//! ```bash
//! cd contrib/guestfs
//! ./build
//! ```
//!
//! If the script ran successfully, `lockc-base.qcow2` file should be present.
//! It cointains the base VM image which will be used by Terraform.
//!
//! #### (Optional) building the image with a custom kernel
//!
//! The `build.sh` script can be also used to create a VM image with a
//! custom kernel if there is a need for kernel testing. You can optionally
//! provide a path to your kernel source tree. Please note that the kernel
//! should be already build on the host with `make`. Our guestfs scripts do
//! only `make modules_install install` to install the kernel image and
//! modules inside a VM. Installing the custom kernel is enabled by using
//! the `CUSTOM_KERNEL` environment variable. Its value has to be set to
//! `true`. By default, the script assumes that your kernel tree is in
//! `~/linux` directory. You can provide a custom path by another
//! environment variable - `KERNEL_SOURCE`. Examples of usage:
//!
//! ```bash
//! CUSTOM_KERNEL=true ./build.sh
//! CUSTOM_KERNEL=true KERNEL_SOURCE=${HOME}/my-git-repos/linux ./build.sh
//! ```
//!
//! If you already used `build.sh` once and you would like to inject a
//! custom kernel into already build qcow2 image, there is a separate script
//! - `reinstall-custom-kernel.sh`. It takes an optional `KERNEL_SOURCE`
//! environment variable. Examples of usage:
//!
//! ```bash
//! ./reinstall-custom-kernel.sh
//! KERNEL_SOURCE=${HOME}/my-git-repos/linux ./reinstall-custom-kernel.sh
//! ```
//!
//! #### Configure libvirt
//!
//! VMs which we are going to run are using 9p to mount the source tree. To
//! ensure that those mounts are going to work correctly, open the
//! `/etc/libvirt/qemu.conf` file and ensure that the following options
//! are present there:
//!
//! ```bash
//! user = "root"
//! group = "root"
//! dynamic_ownership = 0
//! ```
//! If you had to edit the configuration, save the file and restart libvirt:
//!
//! ```bash
//! sudo systemctl restart libvirtd
//! ```
//!
//! #### Running VMs
//!
//! Now it's time to prepare Terraform environment.
//!
//! ```bash
//! cd contrib/terraform/libvirt
//! cp terraform.tfvars.json.example terraform.tfvars.json
//! ```
//!
//! After that, open the `terraform.tfvars.json` file with your favorite text
//! editor. The only setting which you really need to change is
//! `authorized_keys`. Please paste your public SSH key there. Otherwise,
//! connecting to VMs with SSH will be impossible.
//!
//! Initialize the environment with:
//!
//! ```bash
//! terraform init
//! ```
//!
//! And then start the VMs:
//!
//! ```bash
//! terraform apply
//! ```
//!
//! If Terraform finished successfully, you should see the output with IP
//! addresses of virtual machines, like:
//!
//! ```bash
//! ip_control_planes = {
//!   "lockc-control-plane-0" = "10.16.0.225"
//! }
//!
//! You can simply ssh to them using the `opensuse` user:
//!
//! ```bash
//! ssh opensuse@10.16.0.255
//! ```
//! Inside the VM we can check whether Kubernetes is running:
//!
//! ```bash
//! # kubectl get pods -A
//! NAMESPACE     NAME                                            READY
//! STATUS    RESTARTS   AGE
//! kube-system   coredns-78fcd69978-lvshz                        0/1
//! Running   0          7s
//! kube-system   coredns-78fcd69978-q874s                        0/1
//! Running   0          7s
//! kube-system   etcd-lockc-control-plane-0                      1/1
//! Running   0          11s
//! kube-system   kube-apiserver-lockc-control-plane-0            1/1
//! Running   0          10s
//! kube-system   kube-controller-manager-lockc-control-plane-0   1/1
//! Running   0          11s
//! kube-system   kube-proxy-p7nrd                                1/1
//! Running   0          7s
//! kube-system   kube-scheduler-lockc-control-plane-0            1/1
//! Running   0          11s
//! ```
//!
//! And whether lockc is running. The main service can be checked by:
//!
//! ```bash
//! systemctl status lockcd
//! ```
//!
//! We can check also whether lockc's BPF programs are running:
//!
//! ```bash
//! # bpftool prog list
//! 35: tracing  name sched_process_f  tag b3c2c2a08effc879  gpl
//!       loaded_at 2021-08-10T12:23:55+0000  uid 0
//!       xlated 1528B  jited 869B  memlock 4096B  map_ids 3,2
//!       btf_id 95
//! 36: lsm  name clone_audit  tag 33a5e8a5da485fd4  gpl
//!       loaded_at 2021-08-10T12:23:55+0000  uid 0
//!       xlated 1600B  jited 899B  memlock 4096B  map_ids 3,2
//!       btf_id 95
//! 37: lsm  name syslog_audit  tag 80d655f557922055  gpl
//!       loaded_at 2021-08-10T12:23:55+0000  uid 0
//!       xlated 1264B  jited 714B  memlock 4096B  map_ids 3,2
//!       btf_id 95
//! [...]
//! ```
//!
//! And whether it registers containers. Directories inside
//! `/sys/fs/bpf/lockc` represent timestamps of lockcd launch, so it will be
//! different than in the following example.
//!
//! ```bash
//! # bpftool map dump pinned /sys/fs/bpf/lockc/1628598193/map_containers
//! [{
//!         "key": 4506,
//!         "value": {
//!             "policy_level": "POLICY_LEVEL_PRIVILEGED"
//!         }
//! [...]
//! ```

#[macro_use]
extern crate lazy_static;

use byteorder::{NativeEndian, WriteBytesExt};
use std::io::prelude::*;
use std::{fs, path};

#[rustfmt::skip]
mod bpf;
use bpf::*;

pub mod bpfstructs;
mod settings;

lazy_static! {
    static ref SETTINGS: settings::Settings = settings::Settings::new().unwrap();
}

#[derive(thiserror::Error, Debug)]
pub enum CheckBpfLsmError {
    #[error("regex compilation error")]
    RegexError(#[from] regex::Error),

    #[error("I/O error")]
    IOError(#[from] std::io::Error),

    #[error("BPF LSM is not enabled")]
    BpfLsmDisabledError,
}

/// Checks whether BPF LSM is enabled in the system.
pub fn check_bpf_lsm_enabled() -> Result<(), CheckBpfLsmError> {
    let rx = regex::Regex::new(r"bpf")?;
    let path = path::Path::new("/sys")
        .join("kernel")
        .join("security")
        .join("lsm");
    let mut file = fs::File::open(path)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;

    match rx.is_match(&content) {
        true => Ok(()),
        false => Err(CheckBpfLsmError::BpfLsmDisabledError),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LoadProgramError {
    #[error("could not create a new BPF struct instance")]
    NewBpfstructError(#[from] bpfstructs::NewBpfstructError),

    #[error("could not convert the hash to a byte array")]
    ByteWriteError(#[from] std::io::Error),

    #[error("libbpf error")]
    LibbpfError(#[from] libbpf_rs::Error),
}

/// Registers the names of supported container runtime init processes in a BPF
/// map. Based on that information, BPF programs will track those processes and
/// their children.
pub fn init_runtimes(map: &mut libbpf_rs::Map) -> Result<(), LoadProgramError> {
    let runtimes = &SETTINGS.runtimes;
    let val: [u8; 4] = [0, 0, 0, 0];

    for runtime in runtimes.iter() {
        // let mut comm = vec![0; bpfstructs.CONTAINER_ID_MAX_LIMIT];
        // comm.clone_from_slice(std::ffi::CString::new(runtime.as_str())?.as_bytes_with_nul());
        // let key = bpfstructs::runtime_key {
        //     comm: std::ffi::CString::new(runtime.as_str())?
        //         .as_bytes_with_nul()
        //         .try_into()?,
        // };
        let key = bpfstructs::runtime_key::new(runtime.as_str())?;
        let key_b = unsafe { plain::as_bytes(&key) };
        map.update(key_b, &val, libbpf_rs::MapFlags::empty())?;
    }

    Ok(())
}

/// Performs the following BPF-related operations:
/// - loading BPF programs
/// - pinning BPF maps in BPFFS
/// - pinning BPF programs in BPFFS
/// - attaching BPF programs, creating links
/// - pinning links in BPFFS
///
/// All entities pinned in BPFFS have the dedicated directory signed with a
/// timestamp. The reason behind it is to be able to still keep running
/// previous instances of BPF programs while we are in the process of loading
/// new programs. This is done to ensure that **some** instance of BPF programs
/// is always running and that containers are secured.
///
/// TODO: The concept described above still has one hole - the contents of old
/// BPF maps is not migrated in any way. We need to come up with some sane copy
/// mechanism.
pub fn load_programs(path_base_ts: path::PathBuf) -> Result<(), LoadProgramError> {
    let skel_builder = LockcSkelBuilder::default();
    let open_skel = skel_builder.open()?;
    let mut skel = open_skel.load()?;

    let mut path_map_runtimes = path_base_ts.join("map_runtimes");
    skel.maps_mut().runtimes().pin(&mut path_map_runtimes)?;

    init_runtimes(skel.maps_mut().runtimes())?;

    let path_map_containers = path_base_ts.join("map_containers");
    skel.maps_mut().containers().pin(path_map_containers)?;

    let path_map_processes = path_base_ts.join("map_processes");
    skel.maps_mut().processes().pin(path_map_processes)?;

    let path_program_fork = path_base_ts.join("prog_fork");
    skel.progs_mut()
        .sched_process_fork()
        .pin(path_program_fork)?;

    let path_program_clone = path_base_ts.join("prog_clone_audit");
    skel.progs_mut().clone_audit().pin(path_program_clone)?;

    let path_program_syslog = path_base_ts.join("prog_syslog_audit");
    skel.progs_mut().syslog_audit().pin(path_program_syslog)?;

    let mut link_fork = skel.progs_mut().sched_process_fork().attach()?;
    let path_link_fork = path_base_ts.join("link_fork");
    link_fork.pin(path_link_fork)?;

    let mut link_clone = skel.progs_mut().clone_audit().attach_lsm()?;
    let path_link_clone = path_base_ts.join("link_clone_audit");
    link_clone.pin(path_link_clone)?;

    let mut link_syslog = skel.progs_mut().syslog_audit().attach_lsm()?;
    let path_link_syslog = path_base_ts.join("link_syslog_audit");
    link_syslog.pin(path_link_syslog)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum FindLockcBpfPathError {
    #[error("I/O error")]
    IOError(#[from] std::io::Error),

    #[error("BPF objects not found")]
    NotFound,
}

fn find_lockc_bpf_path() -> Result<std::path::PathBuf, FindLockcBpfPathError> {
    let path_base = std::path::Path::new("/sys")
        .join("fs")
        .join("bpf")
        .join("lockc");

    for entry in fs::read_dir(path_base)? {
        let path = entry?.path();
        if path.is_dir() {
            return Ok(path);
        }
    }

    Err(FindLockcBpfPathError::NotFound)
}

#[derive(thiserror::Error, Debug)]
pub enum SkelReusedMapsError {
    #[error("libbpf error")]
    LibbpfError(#[from] libbpf_rs::Error),

    #[error("could not find the BPF objects path")]
    FindLockcBpfPathError(#[from] FindLockcBpfPathError),
}

pub fn skel_reused_maps<'a>() -> Result<LockcSkel<'a>, SkelReusedMapsError> {
    let skel_builder = LockcSkelBuilder::default();
    let mut open_skel = skel_builder.open()?;

    let path_base = find_lockc_bpf_path()?;

    let path_map_containers = path_base.join("map_containers");
    open_skel
        .maps_mut()
        .containers()
        .reuse_pinned_map(path_map_containers)?;

    let path_map_processes = path_base.join("map_processes");
    open_skel
        .maps_mut()
        .processes()
        .reuse_pinned_map(path_map_processes)?;

    let skel = open_skel.load()?;

    Ok(skel)
}

#[derive(thiserror::Error, Debug)]
pub enum ReusedMapsOperationError {
    #[error("libbpf error")]
    LibbpfError(#[from] libbpf_rs::Error),

    #[error("could not convert from slice")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),

    #[error("infallible error")]
    InfallibleError(#[from] std::convert::Infallible),

    #[error("FFI nul error")]
    NulError(#[from] std::ffi::NulError),

    #[error("I/O error")]
    IOError(#[from] std::io::Error),

    #[error("could not reuse BPF maps")]
    SkelReusedMapsError(#[from] SkelReusedMapsError),
}

pub fn add_container(
    container_id: &str,
    pid: u32,
    level: bpfstructs::container_policy_level,
) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    // let container_key = bpfstructs::container_key {
    //     container_id: std::ffi::CString::new(container_id)?
    //         .as_bytes_with_nul()
    //         .try_into()?,
    // };
    let container_key = bpfstructs::container_key::new(container_id)?;
    let container_key_b = unsafe { plain::as_bytes(&container_key) };

    let container = bpfstructs::container {
        policy_level: level,
    };
    let container_b = unsafe { plain::as_bytes(&container) };

    skel.maps_mut().containers().update(
        container_key_b,
        container_b,
        libbpf_rs::MapFlags::empty(),
    )?;

    let mut process_key = vec![];
    process_key.write_u32::<NativeEndian>(pid)?;

    skel.maps_mut().processes().update(
        &process_key,
        container_key_b,
        libbpf_rs::MapFlags::empty(),
    )?;

    Ok(())
}

pub fn delete_container(container_id: &str) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    // let container_key = bpfstructs::container_key {
    //     container_id: std::ffi::CString::new(container_id)?
    //         .as_bytes_with_nul()
    //         .try_into()?,
    // };
    let container_key = bpfstructs::container_key::new(container_id)?;
    let container_key_b = unsafe { plain::as_bytes(&container_key) };

    skel.maps_mut().containers().delete(container_key_b)?;

    Ok(())
}

pub fn write_policy(
    container_id: &str,
    level: bpfstructs::container_policy_level,
) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    // let container_key = bpfstructs::container_key {
    //     container_id: std::ffi::CString::new(container_id)?
    //         .as_bytes_with_nul()
    //         .try_into()?,
    // };
    let container_key = bpfstructs::container_key::new(container_id)?;
    let container_key_b = unsafe { plain::as_bytes(&container_key) };

    let container = bpfstructs::container {
        policy_level: level,
    };
    let container_b = unsafe { plain::as_bytes(&container) };

    skel.maps_mut().containers().update(
        container_key_b,
        container_b,
        libbpf_rs::MapFlags::empty(),
    )?;

    Ok(())
}

pub fn add_process(container_id: &str, pid: u32) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    let mut process_key = vec![];
    process_key.write_u32::<NativeEndian>(pid)?;

    // let process = bpfstructs::container_key {
    //     container_id: std::ffi::CString::new(container_id)?
    //         .as_bytes_with_nul()
    //         .try_into()?,
    // };
    let process = bpfstructs::container_key::new(container_id)?;
    let process_b = unsafe { plain::as_bytes(&process) };

    skel.maps_mut()
        .processes()
        .update(&process_key, process_b, libbpf_rs::MapFlags::empty())?;

    Ok(())
}

pub fn delete_process(pid: u32) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    let mut process_key = vec![];
    process_key.write_u32::<NativeEndian>(pid)?;

    skel.maps_mut().processes().delete(&process_key)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum CleanupError {
    #[error("regex compilation error")]
    RegexError(#[from] regex::Error),

    #[error("I/O error")]
    IOError(#[from] std::io::Error),

    #[error("could not convert path to string")]
    PathToStrConvError,
}

/// Removes all old BPF entities (programs, maps, links) from BPFFS, to stop
/// the execution of old BPF programs. All directories with timestamp lower
/// than the current one get removed.
pub fn cleanup(path_base: path::PathBuf, dirname: &str) -> Result<(), CleanupError> {
    let rx = regex::Regex::new(dirname.to_string().as_str())?;

    for entry in fs::read_dir(path_base)? {
        let path = entry?.path();
        let path_s = path.to_str().ok_or(CleanupError::PathToStrConvError)?;

        if !rx.is_match(path_s) {
            fs::remove_dir_all(path)?;
        }
    }

    Ok(())
}
