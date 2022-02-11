use std::{io, path::Path};

use aya::{
    include_bytes_aligned,
    programs::{BtfTracePoint, Lsm, ProgramError},
    Bpf, BpfError, BpfLoader, Btf, BtfError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Bpf(#[from] BpfError),
}

/// Loads BPF programs from the object file built with clang.
pub fn load_bpf<P: AsRef<Path>>(path_base_r: P) -> Result<Bpf, LoadError> {
    let path_base = path_base_r.as_ref();
    std::fs::create_dir_all(&path_base)?;

    let data = include_bytes_aligned!(concat!(env!("OUT_DIR"), "/lockc.bpf.o"));
    let bpf = BpfLoader::new().map_pin_path(path_base).load(data)?;

    Ok(bpf)
}

#[derive(Error, Debug)]
pub enum AttachError {
    #[error(transparent)]
    Btf(#[from] BtfError),

    #[error(transparent)]
    Program(#[from] ProgramError),

    #[error("could not load the program")]
    ProgLoad,
}

pub fn attach_programs(bpf: &mut Bpf) -> Result<(), AttachError> {
    let btf = Btf::from_sys_fs()?;

    let sched_process_fork: &mut BtfTracePoint = bpf
        .program_mut("sched_process_fork")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    sched_process_fork.load("sched_process_fork", &btf)?;
    sched_process_fork.attach()?;

    let clone_audit: &mut Lsm = bpf
        .program_mut("task_alloc")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    clone_audit.load("task_alloc", &btf)?;
    clone_audit.attach()?;

    let syslog: &mut Lsm = bpf
        .program_mut("syslog")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    syslog.load("syslog", &btf)?;
    syslog.attach()?;

    let mount_audit: &mut Lsm = bpf
        .program_mut("sb_mount")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    mount_audit.load("sb_mount", &btf)?;
    mount_audit.attach()?;

    let open_audit: &mut Lsm = bpf
        .program_mut("file_open")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    open_audit.load("file_open", &btf)?;
    open_audit.attach()?;

    let setuid_audit: &mut Lsm = bpf
        .program_mut("task_fix_setuid")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    setuid_audit.load("task_fix_setuid", &btf)?;
    setuid_audit.attach()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(not(feature = "tests_bpf"), ignore)]
    fn load_and_attach_bpf() {
        let mut bpf = load_bpf("/sys/fs/bpf/lockc-test").expect("Loading BPF failed");
        attach_programs(&mut bpf).expect("Attaching BPF programs failed");
    }
}
