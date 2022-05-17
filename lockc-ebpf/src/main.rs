#![no_std]
#![no_main]

use aya_bpf::{
    bindings::path,
    cty::{c_char, c_long},
    helpers::{bpf_d_path, bpf_probe_read_kernel_str},
    macros::lsm,
    programs::LsmContext,
};
use aya_log_ebpf::{debug, error, info};

use lockc_common::{ContainerPolicyLevel, PATH_LEN};

mod maps;
mod policy;
mod proc;
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
mod vmlinux;

use maps::{CONTAINER_INITIAL_SETUID, MOUNT_TYPE_BUF, PATH_BUF};
use policy::get_container_and_policy_level;
use vmlinux::{cred, file};

/// LSM program triggered by attempts to access the kernel logs. Behavior based
/// on policy levels:
///
/// * restricted: deny
/// * baseline: deny
/// * privileged: allow
#[lsm(name = "syslog")]
pub fn syslog(ctx: LsmContext) -> i32 {
    match try_syslog(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_syslog(ctx: LsmContext) -> Result<i32, i32> {
    let (_, policy_level) = get_container_and_policy_level()?;

    match policy_level {
        ContainerPolicyLevel::NotFound => {
            return Ok(0);
        }
        ContainerPolicyLevel::Lockc => {
            return Ok(0);
        }
        ContainerPolicyLevel::Restricted => {
            info!(&ctx, "syslog: deny accessing syslog");
            return Err(-1);
        }
        ContainerPolicyLevel::Baseline => {
            info!(&ctx, "syslog: deny accessing syslog");
            return Err(-1);
        }
        ContainerPolicyLevel::Privileged => {
            return Ok(0);
        }
    }
}

/// LSM program triggered by any mount attempt. It denies bind mounts to
/// restricted and baseline containers.
#[lsm(name = "sb_mount")]
pub fn sb_mount(ctx: LsmContext) -> i32 {
    match try_sb_mount(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_sb_mount(ctx: LsmContext) -> Result<i32, i32> {
    let (container_id, policy_level) = get_container_and_policy_level()?;

    match policy_level {
        ContainerPolicyLevel::NotFound => {
            return Ok(0);
        }
        ContainerPolicyLevel::Lockc => {
            return Ok(0);
        }
        ContainerPolicyLevel::Restricted => {}
        ContainerPolicyLevel::Baseline => {}
        ContainerPolicyLevel::Privileged => {
            return Ok(0);
        }
    }

    let mount_type = unsafe {
        let mount_type: *const c_char = ctx.arg(2);
        let mount_type_buf = MOUNT_TYPE_BUF.get_mut(0).ok_or(0)?;
        let len =
            bpf_probe_read_kernel_str(mount_type as *const u8, &mut mount_type_buf.mount_type)
                .map_err(|e| e as i32)?;
        core::str::from_utf8_unchecked(&mount_type_buf.mount_type[..len])
    };

    // Apply the policy only on bind mounts, ignore all the other types.
    if !mount_type.starts_with("bind") {
        return Ok(0);
    }

    let src_path = unsafe {
        let dev_name: *const c_char = ctx.arg(0);
        let path_buf = PATH_BUF.get_mut(0).ok_or(0)?;
        let len = bpf_probe_read_kernel_str(dev_name as *const u8, &mut path_buf.path)
            .map_err(|e| e as i32)?;
        core::str::from_utf8_unchecked(&path_buf.path[..len])
    };

    if src_path.starts_with("/run/k3s")
        || src_path.starts_with("/var/lib/docker")
        || src_path.starts_with("/var/lib/kubelet")
        || src_path.starts_with("/var/lib/rancher")
        || src_path.starts_with("/dev/pts")
    {
        return Ok(0);
    }

    let container_id = container_id.ok_or(-1)?;
    let container_id = unsafe { container_id.as_str() };
    error!(
        &ctx,
        "sb_mount: {}: deny bind mounting {}", container_id, src_path
    );

    Err(-1)
}

/// LSM program triggered when user attempts to change the UID. It denies
/// changing the UID to 0 (logging in as root) in restricted and baseline
/// containers.
#[lsm(name = "task_fix_setuid")]
pub fn task_fix_setuid(ctx: LsmContext) -> i32 {
    match { try_task_fix_setuid(ctx) } {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_task_fix_setuid(ctx: LsmContext) -> Result<i32, i32> {
    debug!(&ctx, "function task_fix_setuid called by LSM");

    let (container_id, policy_level) = get_container_and_policy_level()?;
    match policy_level {
        ContainerPolicyLevel::NotFound => {
            return Ok(0);
        }
        ContainerPolicyLevel::Lockc => {
            return Ok(0);
        }
        ContainerPolicyLevel::Restricted => {}
        ContainerPolicyLevel::Baseline => {}
        ContainerPolicyLevel::Privileged => {
            return Ok(0);
        }
    }

    let container_id = container_id.ok_or(-1)?;

    let new: *const cred = unsafe { ctx.arg(0) };
    let uid_new = unsafe { (*new).uid.val };

    if let Some(initial_setuid) = unsafe { CONTAINER_INITIAL_SETUID.get(&container_id) } {
        if *initial_setuid {
            if uid_new == 0 {
                let container_id = unsafe { container_id.as_str() };
                error!(
                    &ctx,
                    "task_fix_setuid: {}: deny logging as root", container_id
                );
                return Err(-1);
            }
        }
    } else {
        debug!(
            &ctx,
            "task_fix_setuid: an initial setuid, policy not enforced"
        );
        unsafe {
            CONTAINER_INITIAL_SETUID
                .insert(&container_id, &true, 0)
                .map_err(|e| e as i32)?
        };
    }

    Ok(0)
}

// TODO(vadorovsky): Remove this once the following PR is merged:
// https://github.com/aya-rs/aya/pull/257
#[inline(always)]
pub fn my_bpf_d_path(path: *mut path, buf: &mut [u8]) -> Result<usize, c_long> {
    let ret = unsafe { bpf_d_path(path, buf.as_mut_ptr() as *mut c_char, buf.len() as u32) };
    if ret < 0 {
        return Err(ret);
    }

    Ok(ret as usize)
}

/// LSM program triggered by opening a file. It denies access to directories
/// which might leak information about host (/sys/fs, /proc/acpi etc.) to
/// restricted and baseline containers.
#[lsm(name = "file_open")]
pub fn file_open(ctx: LsmContext) -> i32 {
    match { try_file_open(ctx) } {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_file_open(ctx: LsmContext) -> Result<i32, i32> {
    let (container_id, policy_level) = get_container_and_policy_level()?;
    match policy_level {
        ContainerPolicyLevel::NotFound => {
            return Ok(0);
        }
        ContainerPolicyLevel::Lockc => {
            return Ok(0);
        }
        ContainerPolicyLevel::Restricted => {}
        ContainerPolicyLevel::Baseline => {}
        ContainerPolicyLevel::Privileged => {
            return Ok(0);
        }
    }

    let buf = unsafe { PATH_BUF.get_mut(0).ok_or(0)? };

    let p = unsafe {
        let f: *const file = ctx.arg(0);
        let p = &(*f).f_path as *const _ as *mut path;
        let len = my_bpf_d_path(p, &mut buf.path).map_err(|_| 0)?;
        if len >= PATH_LEN {
            return Err(0);
        }
        core::str::from_utf8_unchecked(&buf.path[..len])
    };

    let container_id = container_id.ok_or(-1)?;
    let container_id = unsafe { container_id.as_str() };
    debug!(&ctx, "file_open: {}, path: {}", container_id, p);

    if p.starts_with("/proc/acpi")
        || p.starts_with("/sys/fs")
        || p.starts_with("/var/run/secrets/kubernetes.io")
    {
        error!(&ctx, "file_open: {}: deny opening {}", container_id, p);
        return Err(-1);
    }

    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
