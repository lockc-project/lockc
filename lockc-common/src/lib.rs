#![cfg_attr(not(feature = "user"), no_std)]

/// Max configurable PID limit (for x86_64, for the other architectures it's
/// less or equal).
// TODO(vadorovsky): I need to teach aya to be able to resize maps before they
// are loaded into the kernel. So far aya doesn't differentiate between open()
// and load(), it opens the ELF object and loads it immediately in one step.
// I need to change it.
// After that, we will be able to set the limit again up to the upper possible
// limit. And resize according to the max PID limit in sysctl.
// Before it's done - let's stick to the default value to not use too much RAM.
// pub const PID_MAX_LIMIT: u32 = 4194304;
pub const PID_MAX_LIMIT: u32 = 32768;

pub const MOUNT_TYPE_LEN: usize = 5;

pub const PATH_LEN: usize = 64;

const CONTAINER_ID_LEN: usize = 64;

#[cfg_attr(feature = "user", derive(Debug))]
#[derive(Copy, Clone)]
#[repr(C)]
pub enum ContainerPolicyLevel {
    NotFound = -1,

    Lockc,

    // Policy levels.
    Restricted,
    Baseline,
    Privileged,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ContainerID {
    pub id: [u8; CONTAINER_ID_LEN],
}

impl ContainerID {
    /// Convert container ID to a string.
    ///
    /// # Safety
    ///
    /// Container ID is a fixed-size array which has to be nul terminated.
    /// Otherwise this conversion to string is going to fail.
    pub unsafe fn as_str(&self) -> &str {
        core::str::from_utf8_unchecked(&self.id)
    }
}

#[cfg(feature = "user")]
#[derive(thiserror::Error, Debug)]
pub enum NewContainerIDError {
    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),

    #[error("could not convert Vec<u8> to CString")]
    VecU8CStringConv,
}

#[cfg(feature = "user")]
impl ContainerID {
    /// Creates a new container_id instance and converts the given Rust string
    /// into C fixed size char array.
    pub fn new(id: &str) -> Result<Self, NewContainerIDError> {
        let mut id_b = std::ffi::CString::new(id)?.into_bytes_with_nul();
        id_b.resize(CONTAINER_ID_LEN, 0);
        Ok(ContainerID {
            id: id_b
                .try_into()
                .map_err(|_| NewContainerIDError::VecU8CStringConv)?,
        })
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Container {
    pub policy_level: ContainerPolicyLevel,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Process {
    pub container_id: ContainerID,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MountType {
    pub mount_type: [u8; MOUNT_TYPE_LEN],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ContainerPath {
    pub path: [u8; PATH_LEN],
}

#[cfg(feature = "user")]
mod user {
    use super::*;

    unsafe impl aya::Pod for ContainerID {}
    unsafe impl aya::Pod for Container {}
    unsafe impl aya::Pod for Process {}
}
