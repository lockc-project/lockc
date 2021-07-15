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
    let skel_builder = EnclaveSkelBuilder::default();
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
pub enum FindEnclaveBpfPathError {
    #[error("I/O error")]
    IOError(#[from] std::io::Error),

    #[error("BPF objects not found")]
    NotFound,
}

fn find_enclave_bpf_path() -> Result<std::path::PathBuf, FindEnclaveBpfPathError> {
    let path_base = std::path::Path::new("/sys")
        .join("fs")
        .join("bpf")
        .join("enclave");

    for entry in fs::read_dir(path_base)? {
        let path = entry?.path();
        if path.is_dir() {
            return Ok(path);
        }
    }

    Err(FindEnclaveBpfPathError::NotFound)
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
    FindEnclaveBpfPathError(#[from] FindEnclaveBpfPathError),
}

pub fn skel_reused_maps<'a>() -> Result<EnclaveSkel<'a>, SkelReusedMapsError> {
    let skel_builder = EnclaveSkelBuilder::default();
    let mut open_skel = skel_builder.open()?;

    let path_base = find_enclave_bpf_path()?;

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
        &container_b,
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
        .update(&process_key, &process_b, libbpf_rs::MapFlags::empty())?;

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
        &container_b,
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
        .update(&process_key, &process_b, libbpf_rs::MapFlags::empty())?;

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
