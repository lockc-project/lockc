use std::{
    fs::File,
    io::{self, prelude::*},
    path::Path,
};

#[derive(thiserror::Error, Debug)]
pub enum CheckBpfLsmError {
    #[error("regex compilation error")]
    Regex(#[from] regex::Error),

    #[error("I/O error")]
    IO(#[from] io::Error),

    #[error("BPF LSM is not enabled")]
    BpfLsmDisabled,
}

/// Checks whether BPF LSM is enabled in the system.
pub fn check_bpf_lsm_enabled<P: AsRef<Path>>(sys_lsm_path: P) -> Result<(), CheckBpfLsmError> {
    let rx = regex::Regex::new(r"bpf")?;
    let mut file = File::open(sys_lsm_path)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;

    match rx.is_match(&content) {
        true => Ok(()),
        false => Err(CheckBpfLsmError::BpfLsmDisabled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn check_bpf_lsm_enabled_when_correct() {
        let dir = tempdir().unwrap();
        let sys_lsm_path = dir.path().join("lsm");
        let mut f = File::create(&sys_lsm_path).unwrap();
        f.write_all(b"lockdown,capability,bpf").unwrap();
        assert!(check_bpf_lsm_enabled(&sys_lsm_path).is_ok());
    }

    #[test]
    fn check_bpf_lsm_enabled_should_return_error() {
        let dir = tempdir().unwrap();
        let sys_lsm_path = dir.path().join("lsm");
        let mut f = File::create(&sys_lsm_path).unwrap();
        f.write_all(b"lockdown,capability,selinux").unwrap();
        let res = check_bpf_lsm_enabled(&sys_lsm_path);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), CheckBpfLsmError::BpfLsmDisabled));
    }
}
