use aya_bpf::helpers::bpf_get_current_pid_tgid;

use lockc_common::{ContainerID, ContainerPolicyLevel};

use crate::maps::*;

/// Finds the policy level for the current LSM hook.
///
/// If the current process (which triggered the LSM hook) is in a container,
/// returns the container ID and policy level of the container.
///
/// If the current process is not in a container, returns `None`.
#[inline(always)]
pub(crate) fn get_container_and_policy_level(
) -> Result<(Option<ContainerID>, ContainerPolicyLevel), i32> {
    let pid = bpf_get_current_pid_tgid() as u32;
    let process_o = unsafe { PROCESSES.get(&(pid as i32)) };
    match process_o {
        Some(process) => {
            let container_o = unsafe { CONTAINERS.get(&process.container_id) };
            match container_o {
                Some(container) => Ok((Some(process.container_id), container.policy_level)),
                None => Err(-2),
            }
        }
        None => Ok((None, ContainerPolicyLevel::NotFound)),
    }
}
