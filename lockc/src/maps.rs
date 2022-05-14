use aya::{
    maps::{HashMap, MapError},
    Bpf,
};
use config::ConfigError;
use thiserror::Error;
use tracing::{debug, warn};

use lockc_common::{Container, ContainerID, ContainerPolicyLevel, NewContainerIDError, Process};

#[derive(Error, Debug)]
pub enum MapOperationError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Map(#[from] MapError),

    #[error(transparent)]
    NewContainerID(#[from] NewContainerIDError),
}

pub fn add_container(
    bpf: &mut Bpf,
    container_id: String,
    pid: i32,
    policy_level: ContainerPolicyLevel,
) -> Result<(), MapOperationError> {
    debug!(
        container = container_id.as_str(),
        pid = pid,
        // policy_level = policy_level,
        map = "CONTAINERS",
        "adding container to eBPF map",
    );

    let mut containers: HashMap<_, ContainerID, Container> =
        bpf.map_mut("CONTAINERS")?.try_into()?;
    let container_key = ContainerID::new(&container_id)?;
    let container = Container { policy_level };
    containers.insert(container_key, container, 0)?;

    let mut processes: HashMap<_, i32, Process> = bpf.map_mut("PROCESSES")?.try_into()?;
    let process = Process {
        container_id: container_key,
    };
    processes.insert(pid, process, 0)?;

    Ok(())
}

pub fn delete_container(bpf: &mut Bpf, container_id: String) -> Result<(), MapOperationError> {
    debug!(
        container = container_id.as_str(),
        map = "CONTAINERS",
        "deleting container from eBPF map"
    );

    let mut containers: HashMap<_, ContainerID, Container> =
        bpf.map_mut("CONTAINERS")?.try_into()?;
    let container_key = ContainerID::new(&container_id)?;

    // An error while removing a container entry is expected when lockc was
    // installed after some containers were running (which is always the case
    // on Kubernetes). Instead of returning an error, let's warn users.
    if let Err(e) = containers.remove(&container_key) {
        if let MapError::SyscallError { .. } = e {
            warn!(
                container = container_id.as_str(),
                error = e.to_string().as_str(),
                "could not remove the eBPF map container entry"
            );
        }
    }

    // TODO(vadorovsky): Add iter_mut() to HashMap in aya. Due to lack of it,
    // we cannot remove elements immediately when iterating, because iter()
    // borrows the HashMap immutably.
    let mut processes: HashMap<_, i32, Process> = bpf.map_mut("PROCESSES")?.try_into()?;
    let mut to_remove = Vec::new();
    for res in processes.iter() {
        let (pid, process) = res?;
        if process.container_id.id == container_key.id {
            to_remove.push(pid);
            // processes.remove(&pid)?;
        }
    }
    for pid in to_remove {
        processes.remove(&pid)?;
    }

    Ok(())
}

pub fn add_process(bpf: &mut Bpf, container_id: String, pid: i32) -> Result<(), MapOperationError> {
    debug!(
        pid = pid,
        container = container_id.as_str(),
        map = "PROCESSES",
        "adding process to eBPF map",
    );

    let mut processes: HashMap<_, i32, Process> = bpf.map_mut("PROCESSES")?.try_into()?;
    let container_key = ContainerID::new(&container_id)?;
    let process = Process {
        container_id: container_key,
    };
    processes.insert(pid, process, 0)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::{Builder, TempDir};

    use crate::load::load_bpf;

    use super::*;

    fn tmp_path_base() -> TempDir {
        Builder::new()
            .prefix("lockc-temp")
            .rand_bytes(5)
            .tempdir_in("/sys/fs/bpf")
            .expect("Creating temporary dir in BPFFS failed")
    }

    #[test]
    #[cfg_attr(not(feature = "tests_bpf"), ignore)]
    fn test_add_container() {
        let path_base = tmp_path_base();
        let mut bpf = load_bpf(path_base).expect("Loading BPF failed");
        add_container(
            &mut bpf,
            "5833851e673d45fab4d12105bf61c3f4892b2bbf9c12d811db509a4f22475ec9".to_string(),
            42069,
            ContainerPolicyLevel::Baseline,
        )
        .expect("Adding container failed");
    }
}
