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
//! ### Local build
//!
//! This guide assumes that you have `docker` or any other container engine
//! installed.
//!
//! lockc comes with a `Makefile` which supports building the project entirely
//! in containers without installing any dependencies on your host.
//!
//! To start building, you can simply do:
//!
//! ```bash
//! make
//! ```
//!
//! That build should result in binaries produced in the `out/` directory:
//!
//! ```bash
//! $ ls out/
//! lockcd  lockc-runc-wrapper
//! ```
//!
//! BPF programs can be loaded by executing `lockcd`:
//!
//! ```bash
//! sudo ./out/lockcd
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
//! ### Vagrant
//!
//! There is also a possibility to build the project in a Vagrant VM, which is
//! convenient for testing the Kubernetes integration.
//!
//! You can start creating and provisioning the environment by:
//!
//! ```bash
//! vagrant up
//! ```

#[macro_use]
extern crate lazy_static;

use byteorder::{NativeEndian, WriteBytesExt};
use std::io::prelude::*;
use std::{fs, path};

#[rustfmt::skip]
mod bpf;
use bpf::*;

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
pub enum HashError {
    #[error("could not convert the hash to a byte array")]
    ByteWriteError(#[from] std::io::Error),
}

/// Simple string hash function which allows to use strings as keys for BPF
/// maps even though they use u32 as a key type.
pub fn hash(s: &str) -> Result<u32, HashError> {
    let mut hash: u32 = 0;

    for c in s.chars() {
        let c_u32 = c as u32;
        hash += c_u32;
    }

    Ok(hash)
}

#[derive(thiserror::Error, Debug)]
pub enum LoadProgramError {
    #[error("hash error")]
    HashError(#[from] HashError),

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
        let key = hash(runtime)?;
        let mut key_b = vec![];
        key_b.write_u32::<NativeEndian>(key)?;
        map.update(&key_b, &val, libbpf_rs::MapFlags::empty())?;
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

#[repr(C)]
pub enum ContainerPolicyLevel {
    Restricted,
    Baseline,
    Privileged,
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

#[repr(C, packed)]
struct Process {
    container_id: u32,
}

#[repr(C, packed)]
struct Container {
    container_policy_level: ContainerPolicyLevel,
}

#[derive(thiserror::Error, Debug)]
pub enum ReusedMapsOperationError {
    #[error("libbpf error")]
    LibbpfError(#[from] libbpf_rs::Error),

    #[error("I/O error")]
    IOError(#[from] std::io::Error),

    #[error("could not reuse BPF maps")]
    SkelReusedMapsError(#[from] SkelReusedMapsError),

    #[error("hash error")]
    HashError(#[from] HashError),
}

pub fn add_container(
    container_key: u32,
    pid: u32,
    level: ContainerPolicyLevel,
) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    // let container_key = hash(container_id)?;
    let mut container_key_b = vec![];
    container_key_b.write_u32::<NativeEndian>(container_key)?;

    let container = Container {
        container_policy_level: level,
    };
    let container_b = unsafe { plain::as_bytes(&container) };

    skel.maps_mut().containers().update(
        &container_key_b,
        container_b,
        libbpf_rs::MapFlags::empty(),
    )?;

    let mut process_key = vec![];
    process_key.write_u32::<NativeEndian>(pid)?;

    let process = Process {
        container_id: container_key,
    };
    let process_b = unsafe { plain::as_bytes(&process) };

    skel.maps_mut()
        .processes()
        .update(&process_key, process_b, libbpf_rs::MapFlags::empty())?;

    Ok(())
}

pub fn delete_container(container_key: u32) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    let mut container_key_b = vec![];
    container_key_b.write_u32::<NativeEndian>(container_key)?;

    skel.maps_mut().containers().delete(&container_key_b)?;

    Ok(())
}

pub fn write_policy(
    container_id: &str,
    level: ContainerPolicyLevel,
) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    let container_key = hash(container_id)?;
    let mut container_key_b = vec![];
    container_key_b.write_u32::<NativeEndian>(container_key)?;

    let container = Container {
        container_policy_level: level,
    };
    let container_b = unsafe { plain::as_bytes(&container) };

    skel.maps_mut().containers().update(
        &container_key_b,
        container_b,
        libbpf_rs::MapFlags::empty(),
    )?;

    Ok(())
}

pub fn add_process(container_key: u32, pid: u32) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    let mut process_key = vec![];
    process_key.write_u32::<NativeEndian>(pid)?;

    let process = Process {
        container_id: container_key,
    };
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
