use aya_bpf::{
    macros::map,
    maps::{HashMap, PerCpuArray},
};

use lockc_common::{Container, ContainerID, MountType, Path, Process, PID_MAX_LIMIT};

/// BPF map containing the info about a policy which should be enforced on the
/// given container.
#[map]
pub(crate) static mut CONTAINERS: HashMap<ContainerID, Container> =
    HashMap::with_max_entries(PID_MAX_LIMIT, 0);

/// BPF map which maps the PID to a container it belongs to. The value of this
/// map, which represents the container, is a key of `containers` BPF map, so
/// it can be used immediately for lookups in `containers` map.
#[map]
pub(crate) static mut PROCESSES: HashMap<i32, Process> =
    HashMap::with_max_entries(PID_MAX_LIMIT, 0);

#[map]
pub(crate) static mut CONTAINER_INITIAL_SETUID: HashMap<ContainerID, bool> =
    HashMap::with_max_entries(PID_MAX_LIMIT, 0);

#[map]
pub(crate) static mut MOUNT_TYPE_BUF: PerCpuArray<MountType> = PerCpuArray::with_max_entries(1, 0);

#[map]
pub(crate) static mut PATH_BUF: PerCpuArray<Path> = PerCpuArray::with_max_entries(1, 0);
