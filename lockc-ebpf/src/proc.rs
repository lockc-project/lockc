use aya_bpf::{macros::btf_tracepoint, programs::BtfTracePointContext};
use aya_log_ebpf::debug;

use lockc_common::Process;

use crate::{maps::*, vmlinux::task_struct};

/// Monitors all new tasks/functions created in the system and checks whether
/// it's a child of some already containerized process (either the container
/// runtime or any of its children)
/// In any other case, it does not do anything.
///
/// # Arguments
///
/// * `ppid` - PID of the parent task
/// * `child` - PID of the new task
#[inline]
fn handle_new_process(ctx: BtfTracePointContext, ppid: i32, pid: i32) -> Result<i32, i32> {
    // Check if parent process is containerized (already registeed in BPF map).
    // If not, don't do anything.
    if let Some(parent) = unsafe { PROCESSES.get(&ppid) } {
        // Check if child process is already registered. If yes, don't do
        // anything.
        let child_lookup = unsafe { PROCESSES.get(&pid) };
        if child_lookup.is_some() {
            return Ok(0);
        }

        // Register a new process.
        let container_id = parent.container_id;
        debug!(
            &ctx,
            "new containerized process: pid: {}, container_id: {}",
            pid,
            unsafe { container_id.as_str() }
        );
        let child = Process { container_id };
        unsafe { PROCESSES.insert(&pid, &child, 0).map_err(|e| e as i32)? };
    }

    Ok(0)
}

/// Tracepoint program triggered by forking a process.
///
/// It's used to find a potential new runc process.
#[btf_tracepoint(name = "sched_process_fork")]
pub fn sched_process_fork(ctx: BtfTracePointContext) -> i32 {
    match { try_sched_process_fork(ctx) } {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_sched_process_fork(ctx: BtfTracePointContext) -> Result<i32, i32> {
    let parent_task: *const task_struct = unsafe { ctx.arg(0) };
    let child_task: *const task_struct = unsafe { ctx.arg(1) };

    let ppid = unsafe { (*parent_task).pid };
    let pid = unsafe { (*child_task).pid };

    handle_new_process(ctx, ppid, pid)
}

/// Tracepoint program triggered by running a new proccess with a binary
/// executable.
///
/// It's used to find a potential new runc process.
#[btf_tracepoint(name = "sched_process_exec")]
pub fn sched_process_exec(ctx: BtfTracePointContext) -> i32 {
    match { try_sched_process_exec(ctx) } {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_sched_process_exec(ctx: BtfTracePointContext) -> Result<i32, i32> {
    let task: *const task_struct = unsafe { ctx.arg(0) };

    let ppid = unsafe { (*(*task).parent).pid };
    let pid = unsafe { (*task).pid };

    handle_new_process(ctx, ppid, pid)
}

/// Tracepoint program triggered by a process exiting.
///
/// It's used to check if any process registered by lockc is exiting and
/// information about it can be removed.
#[btf_tracepoint(name = "sched_process_exit")]
pub fn sched_process_exit(ctx: BtfTracePointContext) -> i32 {
    match { try_sched_process_exit(ctx) } {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_sched_process_exit(ctx: BtfTracePointContext) -> Result<i32, i32> {
    let task: *const task_struct = unsafe { ctx.arg(0) };

    let pid = unsafe { (*task).pid };

    unsafe { PROCESSES.remove(&pid).map_err(|e| e as i32)? };

    Ok(0)
}
