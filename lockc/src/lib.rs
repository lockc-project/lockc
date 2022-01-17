//! This is an auto-generated code documentation. For more detailed documentation
//! (with all information about usage, deployment and architecture) please check
//! out [the book](https://rancher-sandbox.github.io/lockc/).

#[macro_use]
extern crate lazy_static;

use std::{
    fs,
    io::{self, prelude::*},
    num, path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time,
};

use byteorder::{NativeEndian, WriteBytesExt};
use sysctl::Sysctl;

use bpfstructs::BpfStruct;
use lockc_uprobes::{add_container, add_process, delete_container};
use uprobe_ext::FindSymbolUprobeExt;

#[rustfmt::skip]
mod bpf;
use bpf::*;

pub mod bpfstructs;
pub mod runc;
mod settings;
mod uprobe_ext;

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
    for (i, allowed_path_s) in SETTINGS.allowed_paths_mount_restricted.iter().enumerate() {
        bpfstructs::accessed_path::new(allowed_path_s)?
            .map_update(maps.ap_mnt_restr(), i.try_into().unwrap())?;
    }

    for (i, allowed_path_s) in SETTINGS.allowed_paths_mount_baseline.iter().enumerate() {
        bpfstructs::accessed_path::new(allowed_path_s)?
            .map_update(maps.ap_mnt_base(), i.try_into().unwrap())?;
    }

    for (i, allowed_path_s) in SETTINGS.allowed_paths_access_restricted.iter().enumerate() {
        bpfstructs::accessed_path::new(allowed_path_s)?
            .map_update(maps.ap_acc_restr(), i.try_into().unwrap())?;
    }

    for (i, allowed_path_s) in SETTINGS.allowed_paths_access_baseline.iter().enumerate() {
        bpfstructs::accessed_path::new(allowed_path_s)?
            .map_update(maps.ap_acc_base(), i.try_into().unwrap())?;
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum GetPidMaxError {
    #[error(transparent)]
    ParseInt(#[from] num::ParseIntError),

    #[error(transparent)]
    Sysctl(#[from] sysctl::SysctlError),
}

/// Gets the max PID number configured in the system.
fn get_pid_max() -> Result<u32, GetPidMaxError> {
    let pid_max_s = sysctl::Ctl::new("kernel.pid_max")?.value_string()?;
    let pid_max = pid_max_s.parse::<u32>()?;
    Ok(pid_max)
}

pub struct BpfContext<'a> {
    pub skel: LockcSkel<'a>,
}

#[derive(thiserror::Error, Debug)]
pub enum NewBpfContextError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Libbpf(#[from] libbpf_rs::Error),

    #[error(transparent)]
    AttachUprobeAddr(#[from] uprobe_ext::AttachUprobeAddrError),

    #[error(transparent)]
    GetPidMax(#[from] GetPidMaxError),

    #[error(transparent)]
    InitAllowedPaths(#[from] InitAllowedPathsError),

    #[error(transparent)]
    InitRuntimes(#[from] InitRuntimesError),
}

impl<'a> BpfContext<'a> {
    /// Performs the following BPF-related operations:
    /// - loading BPF programs
    /// - trying to reuse pinned BPF maps from BPFFS (if there was a previous
    ///   lockc instance running in the system)
    /// - resizing PID-related BPF maps
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
    pub fn new<P: AsRef<path::Path>>(path_base_r: P) -> Result<Self, NewBpfContextError> {
        let path_base = path_base_r.as_ref();
        let skel_builder = LockcSkelBuilder::default();
        let mut open_skel = skel_builder.open()?;

        let path_map_runtimes = path_base.join("map_runtimes");
        if path_map_runtimes.exists() {
            open_skel
                .maps_mut()
                .runtimes()
                .reuse_pinned_map(path_map_runtimes.clone())?;
        }

        let pid_max = get_pid_max()?;

        let path_map_containers = path_base.join("map_containers");
        if path_map_containers.exists() {
            open_skel
                .maps_mut()
                .containers()
                .reuse_pinned_map(path_map_containers.clone())?;
        } else {
            open_skel.maps_mut().containers().set_max_entries(pid_max)?;
        }

        let path_map_processes = path_base.join("map_processes");
        if path_map_processes.exists() {
            open_skel
                .maps_mut()
                .processes()
                .reuse_pinned_map(path_map_processes.clone())?;
        } else {
            open_skel.maps_mut().processes().set_max_entries(pid_max)?;
        }

        let path_map_ap_mnt_restr = path_base.join("map_ap_mnt_restr");
        if path_map_ap_mnt_restr.exists() {
            open_skel
                .maps_mut()
                .ap_mnt_restr()
                .reuse_pinned_map(path_map_ap_mnt_restr.clone())?;
        }

        let path_map_ap_mnt_base = path_base.join("map_ap_mnt_base");
        if path_map_ap_mnt_base.exists() {
            open_skel
                .maps_mut()
                .ap_mnt_base()
                .reuse_pinned_map(path_map_ap_mnt_base.clone())?;
        }

        let path_map_ap_acc_restr = path_base.join("map_ap_acc_restr");
        if path_map_ap_acc_restr.exists() {
            open_skel
                .maps_mut()
                .ap_acc_restr()
                .reuse_pinned_map(path_map_ap_acc_restr.clone())?;
        }

        let path_map_ap_acc_base = path_base.join("map_ap_acc_base");
        if path_map_ap_acc_base.exists() {
            open_skel
                .maps_mut()
                .ap_acc_base()
                .reuse_pinned_map(path_map_ap_acc_base.clone())?;
        }

        let path_map_dp_acc_restr = path_base.join("map_dp_acc_restr");
        if path_map_dp_acc_restr.exists() {
            open_skel
                .maps_mut()
                .dp_acc_restr()
                .reuse_pinned_map(path_map_dp_acc_restr.clone())?;
        }

        let path_map_dp_acc_base = path_base.join("map_dp_acc_base");
        if path_map_dp_acc_base.exists() {
            open_skel
                .maps_mut()
                .dp_acc_base()
                .reuse_pinned_map(path_map_dp_acc_base.clone())?;
        }

        let mut skel = open_skel.load()?;

        if !path_map_runtimes.exists() {
            skel.maps_mut().runtimes().pin(path_map_runtimes)?;
        }
        init_runtimes(skel.maps_mut().runtimes())?;
        if !path_map_containers.exists() {
            skel.maps_mut().containers().pin(path_map_containers)?;
        }
        if !path_map_processes.exists() {
            skel.maps_mut().processes().pin(path_map_processes)?;
        }
        if !path_map_ap_mnt_restr.exists() {
            skel.maps_mut().ap_mnt_restr().pin(path_map_ap_mnt_restr)?;
        }
        if !path_map_ap_mnt_base.exists() {
            skel.maps_mut().ap_mnt_base().pin(path_map_ap_mnt_base)?;
        }
        if !path_map_ap_acc_restr.exists() {
            skel.maps_mut().ap_acc_restr().pin(path_map_ap_acc_restr)?;
        }
        if !path_map_ap_acc_base.exists() {
            skel.maps_mut().ap_acc_base().pin(path_map_ap_acc_base)?;
        }
        if !path_map_dp_acc_restr.exists() {
            skel.maps_mut().dp_acc_restr().pin(path_map_dp_acc_restr)?;
        }
        if !path_map_dp_acc_base.exists() {
            skel.maps_mut().dp_acc_base().pin(path_map_dp_acc_base)?;
        }
        init_allowed_paths(skel.maps_mut())?;

        let path_program_fork = path_base.join("prog_fork");
        if path_program_fork.exists() {
            fs::remove_file(path_program_fork.clone())?;
        }
        skel.progs_mut()
            .sched_process_fork()
            .pin(path_program_fork)?;

        let path_program_clone = path_base.join("prog_clone_audit");
        if path_program_clone.exists() {
            fs::remove_file(path_program_clone.clone())?;
        }
        skel.progs_mut().clone_audit().pin(path_program_clone)?;

        let path_program_syslog = path_base.join("prog_syslog_audit");
        if path_program_syslog.exists() {
            fs::remove_file(path_program_syslog.clone())?;
        }
        skel.progs_mut().syslog_audit().pin(path_program_syslog)?;

        let path_program_mount = path_base.join("prog_mount_audit");
        if path_program_mount.exists() {
            fs::remove_file(path_program_mount.clone())?;
        }
        skel.progs_mut().mount_audit().pin(path_program_mount)?;

        let path_program_open = path_base.join("prog_open_audit");
        if path_program_open.exists() {
            fs::remove_file(path_program_open.clone())?;
        }
        skel.progs_mut().open_audit().pin(path_program_open)?;

        let path_program_add_container = path_base.join("prog_add_container");
        if path_program_add_container.exists() {
            fs::remove_file(path_program_add_container.clone())?;
        }
        skel.progs_mut()
            .add_container()
            .pin(path_program_add_container)?;

        let path_program_delete_container = path_base.join("prog_delete_container");
        if path_program_delete_container.exists() {
            fs::remove_file(path_program_delete_container.clone())?;
        }
        skel.progs_mut()
            .delete_container()
            .pin(path_program_delete_container)?;

        let path_program_add_process = path_base.join("prog_add_process");
        if path_program_add_process.exists() {
            fs::remove_file(path_program_add_process.clone())?;
        }
        skel.progs_mut()
            .add_process()
            .pin(path_program_add_process)?;

        let mut link_fork = skel.progs_mut().sched_process_fork().attach()?;
        let path_link_fork = path_base.join("link_fork");
        if path_link_fork.exists() {
            fs::remove_file(path_link_fork.clone())?;
        }
        link_fork.pin(path_link_fork)?;

        let mut link_clone = skel.progs_mut().clone_audit().attach_lsm()?;
        let path_link_clone = path_base.join("link_clone_audit");
        if path_link_clone.exists() {
            fs::remove_file(path_link_clone.clone())?;
        }
        link_clone.pin(path_link_clone)?;

        let mut link_syslog = skel.progs_mut().syslog_audit().attach_lsm()?;
        let path_link_syslog = path_base.join("link_syslog_audit");
        if path_link_syslog.exists() {
            fs::remove_file(path_link_syslog.clone())?;
        }
        link_syslog.pin(path_link_syslog)?;

        let mut link_mount = skel.progs_mut().mount_audit().attach_lsm()?;
        let path_link_mount = path_base.join("link_mount_audit");
        if path_link_mount.exists() {
            fs::remove_file(path_link_mount.clone())?;
        }
        link_mount.pin(path_link_mount)?;

        let mut link_open = skel.progs_mut().open_audit().attach_lsm()?;
        let path_link_open = path_base.join("link_open_audit");
        if path_link_open.exists() {
            fs::remove_file(path_link_open.clone())?;
        }
        link_open.pin(path_link_open)?;

        let link_add_container = skel.progs_mut().add_container().attach_uprobe_addr(
            false,
            -1,
            add_container as *const () as usize,
        )?;
        skel.links.add_container = link_add_container.into();
        // NOTE(vadorovsky): Currently it's impossible to pin uprobe links, but
        // it would be REALLY NICE to be able to do so.
        // let path_link_add_container = path_base.join("link_add_container");
        // link_add_container.pin(path_link_add_container)?;

        let link_delete_container = skel.progs_mut().delete_container().attach_uprobe_addr(
            false,
            -1,
            delete_container as *const () as usize,
        )?;
        skel.links.delete_container = link_delete_container.into();
        // NOTE(vadorovsky): Currently it's impossible to pin uprobe links, but
        // it would be REALLY NICE to be able to do so.
        // let path_link_delete_container = path_base.join("link_delete_container");
        // link_delete_container.pin(path_link_delete_container)?;

        let link_add_process = skel.progs_mut().add_process().attach_uprobe_addr(
            false,
            -1,
            add_process as *const () as usize,
        )?;
        skel.links.add_process = link_add_process.into();
        // NOTE(vadorovsky): Currently it's impossible to pin uprobe links, but
        // it would be REALLY NICE to be able to do so.
        // let path_link_add_process = path_base.join("link_add_process");
        // link_add_process.pin(path_link_add_process)?;

        Ok(BpfContext { skel })
    }

    pub fn work_loop(&self) -> Result<(), ctrlc::Error> {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })?;
        while running.load(Ordering::SeqCst) {
            eprint!(".");
            thread::sleep(time::Duration::from_secs(1));
        }

        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use std::panic;

    use tempfile::tempdir;

    use super::*;

    static PATH_BASE: &str = "/sys/fs/bpf/lockc-test";

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

    #[test]
    fn get_pid_max_when_correct() {
        assert!(get_pid_max().is_ok());
    }

    // It doesn't work on Github actions, see
    // https://github.com/rancher-sandbox/lockc/issues/65
    #[test]
    #[ignore]
    fn test_bpf_context() {
        let _cleanup = PathBase::new();
        assert!(BpfContext::new(PATH_BASE).is_ok());
    }

    #[test]
    fn find_lockc_bpf_path_when_correct() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("test");
        fs::create_dir_all(subdir.clone()).unwrap();
        assert_eq!(find_lockc_bpf_path(dir.path()).unwrap(), subdir);
    }
}
