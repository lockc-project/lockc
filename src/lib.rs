use byteorder::{NativeEndian, WriteBytesExt};
use std::io::prelude::*;
use std::{fs, path};

mod bpf;
use bpf::*;

const RUNC_CHILD: &str = "runc:[2:INIT]";

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
    let path = path::Path::new("/sys").join("kernel").join("security").join("lsm");
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
pub fn hash(s: &str) -> Result<Vec<u8>, HashError> {
    let mut hash: u32 = 0;

    for c in s.chars() {
	let c_u32 = c as u32;
	hash += c_u32;
    }

    let mut wtr = vec![];
    wtr.write_u32::<NativeEndian>(hash)?;

    Ok(wtr)
}

#[derive(thiserror::Error, Debug)]
pub enum LoadProgramError {
    #[error("hash error")]
    HashError(#[from] HashError),

    #[error("libbpf error")]
    LibbpfError(#[from] libbpf_rs::Error),
}

/// Registers the names of supported container runtime init processes in a BPF
/// map. Based on that information, BPF programs will track those processes and
/// their children.
pub fn init_runtimes(map: &mut libbpf_rs::Map) -> Result<(), LoadProgramError> {
    let key = hash(RUNC_CHILD)?;
    let val: [u8; 4] = [0, 0, 0, 0];

    map.update(&key, &val, libbpf_rs::MapFlags::empty())?;

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

    let path_program_fork = path_base_ts.join("prog_fork");
    skel.progs_mut().sched_process_fork().pin(path_program_fork)?;

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
pub fn cleanup(path_base: path::PathBuf, dirname: &String) -> Result<(), CleanupError> {
    let rx = regex::Regex::new(format!(r#"{}"#, dirname).as_str())?;

    for entry in fs::read_dir(path_base)? {
	let path = entry?.path();
	let path_s = path.to_str().ok_or(CleanupError::PathToStrConvError)?;

	if !rx.is_match(path_s) {
	    fs::remove_dir_all(path)?;
	}
    }

    Ok(())
}
