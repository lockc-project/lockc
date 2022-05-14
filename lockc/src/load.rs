use std::{io, path::Path};

use aya::{
    include_bytes_aligned,
    programs::{BtfTracePoint, Lsm, ProgramError},
    Bpf, BpfError, Btf, BtfError,
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

    #[cfg(debug_assertions)]
    let bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/lockc"
    ))?;
    #[cfg(not(debug_assertions))]
    let bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/lockc"
    ))?;

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

    let program: &mut BtfTracePoint = bpf
        .program_mut("sched_process_fork")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    program.load("sched_process_fork", &btf)?;
    program.attach()?;

    let program: &mut BtfTracePoint = bpf
        .program_mut("sched_process_exec")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    program.load("sched_process_exec", &btf)?;
    program.attach()?;

    let program: &mut BtfTracePoint = bpf
        .program_mut("sched_process_exit")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    program.load("sched_process_exit", &btf)?;
    program.attach()?;

    let program: &mut Lsm = bpf
        .program_mut("syslog")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    program.load("syslog", &btf)?;
    program.attach()?;

    let program: &mut Lsm = bpf
        .program_mut("sb_mount")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    program.load("sb_mount", &btf)?;
    program.attach()?;

    let program: &mut Lsm = bpf
        .program_mut("task_fix_setuid")
        .ok_or(AttachError::ProgLoad)?
        .try_into()?;
    program.load("task_fix_setuid", &btf)?;
    program.attach()?;

    // let open_audit: &mut Lsm = bpf
    //     .program_mut("file_open")
    //     .ok_or(AttachError::ProgLoad)?
    //     .try_into()?;
    // open_audit.load("file_open", &btf)?;
    // open_audit.attach()?;

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
