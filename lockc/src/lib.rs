//! This is an auto-generated code documentation. For more detailed documentation
//! (with all information about usage, deployment and architecture) please check
//! out [the book](https://rancher-sandbox.github.io/lockc/).

#[macro_use]
extern crate lazy_static;

use bpfstructs::BpfStruct;
use byteorder::{NativeEndian, WriteBytesExt};
use std::{convert::TryInto, fs, io, io::prelude::*, path};

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
    IOError(#[from] io::Error),

    #[error("BPF LSM is not enabled")]
    BpfLsmDisabledError,
}

/// Checks whether BPF LSM is enabled in the system.
pub fn check_bpf_lsm_enabled<P: AsRef<path::Path>>(
    sys_lsm_path: P,
) -> Result<(), CheckBpfLsmError> {
    let rx = regex::Regex::new(r"bpf")?;
    let mut file = fs::File::open(sys_lsm_path)?;
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
    ByteWriteError(#[from] io::Error),
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
pub enum InitRuntimesError {
    #[error("hash error")]
    HashError(#[from] HashError),

    #[error("could not convert the hash to a byte array")]
    ByteWriteError(#[from] io::Error),

    #[error("libbpf error")]
    LibbpfError(#[from] libbpf_rs::Error),
}

/// Registers the names of supported container runtime init processes in a BPF
/// map. Based on that information, BPF programs will track those processes and
/// their children.
pub fn init_runtimes(map: &mut libbpf_rs::Map) -> Result<(), InitRuntimesError> {
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

#[derive(thiserror::Error, Debug)]
pub enum InitAllowedPathsError {
    #[error("could not create a new BPF struct instance")]
    NewBpfstructError(#[from] bpfstructs::NewBpfstructError),

    #[error("BPF map operation error")]
    MapOperationError(#[from] bpfstructs::MapOperationError),
}

/// Registers the allowed directories for restricted and baseline containers in
/// BPF maps. Based on that information, mount_audit BPF prrogram will make a
/// decision whether to allow a bind mount for a given container.
pub fn init_allowed_paths(mut maps: LockcMapsMut) -> Result<(), InitAllowedPathsError> {
    for (i, allowed_path_s) in SETTINGS.allowed_paths_restricted.iter().enumerate() {
        bpfstructs::allowed_path::new(allowed_path_s)?
            .map_update(maps.allowed_paths_restricted(), i.try_into().unwrap())?;
    }

    for (i, allowed_path_s) in SETTINGS.allowed_paths_baseline.iter().enumerate() {
        bpfstructs::allowed_path::new(allowed_path_s)?
            .map_update(maps.allowed_paths_baseline(), i.try_into().unwrap())?;
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum LoadProgramError {
    #[error("hash error")]
    HashError(#[from] HashError),

    #[error("init runtimes error")]
    InitRuntimesError(#[from] InitRuntimesError),

    #[error("init allowed paths error")]
    InitAllowedPathsError(#[from] InitAllowedPathsError),

    #[error("could not convert the hash to a byte array")]
    ByteWriteError(#[from] io::Error),

    #[error("libbpf error")]
    LibbpfError(#[from] libbpf_rs::Error),

    #[error("could not align the byte data")]
    ByteAlignError,
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
pub fn load_programs<P: AsRef<path::Path>>(path_base_ts_r: P) -> Result<(), LoadProgramError> {
    let path_base_ts = path_base_ts_r.as_ref();
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

    let path_map_allowed_paths_restricted = path_base_ts.join("map_allowed_paths_restricted");
    skel.maps_mut()
        .allowed_paths_restricted()
        .pin(path_map_allowed_paths_restricted)?;

    let path_map_allowed_paths_baseline = path_base_ts.join("map_allowed_paths_baseline");
    skel.maps_mut()
        .allowed_paths_baseline()
        .pin(path_map_allowed_paths_baseline)?;

    init_allowed_paths(skel.maps_mut())?;

    let path_program_fork = path_base_ts.join("prog_fork");
    skel.progs_mut()
        .sched_process_fork()
        .pin(path_program_fork)?;

    let path_program_clone = path_base_ts.join("prog_clone_audit");
    skel.progs_mut().clone_audit().pin(path_program_clone)?;

    let path_program_syslog = path_base_ts.join("prog_syslog_audit");
    skel.progs_mut().syslog_audit().pin(path_program_syslog)?;

    let path_program_mount = path_base_ts.join("prog_mount_audit");
    skel.progs_mut().mount_audit().pin(path_program_mount)?;

    let mut link_fork = skel.progs_mut().sched_process_fork().attach()?;
    let path_link_fork = path_base_ts.join("link_fork");
    link_fork.pin(path_link_fork)?;

    let mut link_clone = skel.progs_mut().clone_audit().attach_lsm()?;
    let path_link_clone = path_base_ts.join("link_clone_audit");
    link_clone.pin(path_link_clone)?;

    let mut link_syslog = skel.progs_mut().syslog_audit().attach_lsm()?;
    let path_link_syslog = path_base_ts.join("link_syslog_audit");
    link_syslog.pin(path_link_syslog)?;

    let mut link_mount = skel.progs_mut().mount_audit().attach_lsm()?;
    let path_link_mount = path_base_ts.join("link_mount_audit");
    link_mount.pin(path_link_mount)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum FindLockcBpfPathError {
    #[error("I/O error")]
    IOError(#[from] io::Error),

    #[error("BPF objects not found")]
    NotFound,
}

/// Find the directory with BPF objects of the currently running lockc
/// BPF programs.
fn find_lockc_bpf_path<P: AsRef<path::Path>>(
    path_base: P,
) -> Result<path::PathBuf, FindLockcBpfPathError> {
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

/// Returns a new BPF skeleton with reused containers and processes maps. Meant
/// to be used by lockc-runc-wrapper to interact with those maps.
pub fn skel_reused_maps<'a>() -> Result<LockcSkel<'a>, SkelReusedMapsError> {
    let skel_builder = LockcSkelBuilder::default();
    let mut open_skel = skel_builder.open()?;

    let path_base = path::Path::new("/sys").join("fs").join("bpf").join("lockc");
    let bpf_path = find_lockc_bpf_path(path_base)?;

    let path_map_containers = bpf_path.join("map_containers");
    open_skel
        .maps_mut()
        .containers()
        .reuse_pinned_map(path_map_containers)?;

    let path_map_processes = bpf_path.join("map_processes");
    open_skel
        .maps_mut()
        .processes()
        .reuse_pinned_map(path_map_processes)?;

    let skel = open_skel.load()?;

    Ok(skel)
}

#[derive(thiserror::Error, Debug)]
pub enum ReusedMapsOperationError {
    #[error("BPF map operation error")]
    MapOperationError(#[from] bpfstructs::MapOperationError),

    #[error("hash error")]
    HashError(#[from] HashError),

    #[error("could not reuse BPF maps")]
    SkelReusedMapsError(#[from] SkelReusedMapsError),
}

/// Adds a new container and its first associated process into BPF maps.
pub fn add_container(
    container_key: u32,
    pid: u32,
    level: bpfstructs::container_policy_level,
) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    bpfstructs::container {
        policy_level: level,
    }
    .map_update(skel.maps_mut().containers(), container_key)?;

    bpfstructs::process {
        container_id: container_key,
    }
    .map_update(skel.maps_mut().processes(), pid)?;

    Ok(())
}

/// Deletes the given container from BPF map.
pub fn delete_container(container_key: u32) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;
    bpfstructs::map_delete(skel.maps_mut().containers(), container_key)?;

    Ok(())
}

/// Writes the given policy to the container info in BPF map.
pub fn write_policy(
    container_id: &str,
    level: bpfstructs::container_policy_level,
) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    let container_key = hash(container_id)?;
    bpfstructs::container {
        policy_level: level,
    }
    .map_update(skel.maps_mut().containers(), container_key)?;

    Ok(())
}

/// Adds the given process as a container's member in the BPF map. After this
/// action, LSM BPF programs are going to enforce policies on that process.
pub fn add_process(container_key: u32, pid: u32) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;

    bpfstructs::process {
        container_id: container_key,
    }
    .map_update(skel.maps_mut().processes(), pid)?;

    Ok(())
}

/// Removes the given process from BPF map.
pub fn delete_process(pid: u32) -> Result<(), ReusedMapsOperationError> {
    let mut skel = skel_reused_maps()?;
    bpfstructs::map_delete(skel.maps_mut().processes(), pid)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum CleanupError {
    #[error("regex compilation error")]
    RegexError(#[from] regex::Error),

    #[error("I/O error")]
    IOError(#[from] io::Error),

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

#[cfg(test)]
mod tests {
    use std::panic;

    use tempfile::tempdir;

    use super::*;

    static PATH_BASE: &str = "/sys/fs/bpf/lockc/test";

    /// Represents the real base path for lockc's test BPF objects (programs,
    /// maps, links).
    struct PathBase;

    impl PathBase {
        fn new() -> PathBase {
            match fs::remove_dir_all(PATH_BASE) {
                Ok(_) => {}
                Err(e) => match e.kind() {
                    io::ErrorKind::NotFound => {}
                    _ => panic::panic_any(e),
                },
            }
            fs::create_dir_all(PATH_BASE).unwrap();
            PathBase {}
        }
    }

    impl Drop for PathBase {
        /// Cleans up the base path for lockc's test BPF objects.
        fn drop(&mut self) {
            fs::remove_dir_all(PATH_BASE).unwrap();
        }
    }

    #[test]
    fn check_bpf_lsm_enabled_when_correct() {
        let dir = tempdir().unwrap();
        let sys_lsm_path = dir.path().join("lsm");
        let mut f = fs::File::create(sys_lsm_path.clone()).unwrap();
        f.write_all(b"lockdown,capability,bpf").unwrap();
        assert!(check_bpf_lsm_enabled(sys_lsm_path).is_ok());
    }

    #[test]
    fn check_bpf_lsm_enabled_should_return_error() {
        let dir = tempdir().unwrap();
        let sys_lsm_path = dir.path().join("lsm");
        let mut f = fs::File::create(sys_lsm_path.clone()).unwrap();
        f.write_all(b"lockdown,capability,selinux").unwrap();
        let res = check_bpf_lsm_enabled(sys_lsm_path);
        assert!(res.is_err());
        assert!(matches!(
            res.unwrap_err(),
            CheckBpfLsmError::BpfLsmDisabledError
        ));
    }

    #[test]
    fn hash_should_return_hash_when_correct() {
        let test_string = "Test string for hash function";
        assert!(hash(test_string).is_ok());
        let returned_hash = hash(test_string).unwrap();
        let correct_hash: u32 = 2824;
        assert_eq!(returned_hash, correct_hash);
    }

    // It doesn't work on Github actions, see
    // https://github.com/rancher-sandbox/lockc/issues/65
    #[test]
    #[ignore]
    fn test_load_programs() {
        let _cleanup = PathBase::new();
        assert!(load_programs(PATH_BASE).is_ok());
    }

    #[test]
    fn find_lockc_bpf_path_when_correct() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("test");
        fs::create_dir_all(subdir.clone()).unwrap();
        assert_eq!(find_lockc_bpf_path(dir.path()).unwrap(), subdir);
    }
}
