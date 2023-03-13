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
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[derive(Copy, Clone)]
#[repr(C)]
pub enum ContainerPolicyLevel {
    NotFound = -1,

    Lockc,

    // Policy levels.
    Restricted,
    Offline,
    Baseline,
    Privileged,
}

#[cfg(feature = "user")]
impl std::fmt::Display for ContainerPolicyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerPolicyLevel::NotFound => write!(f, "not found"),
            ContainerPolicyLevel::Lockc => write!(f, "lockc"),
            ContainerPolicyLevel::Restricted => write!(f, "restricted"),
            ContainerPolicyLevel::Offline => write!(f, "offline"),
            ContainerPolicyLevel::Baseline => write!(f, "baseline"),
            ContainerPolicyLevel::Privileged => write!(f, "privileged"),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ContainerID {
    pub id: [u8; CONTAINER_ID_LEN],
}

#[cfg(feature = "user")]
impl std::str::FromStr for ContainerID {
    type Err = std::ffi::NulError;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        let mut id = std::ffi::CString::new(id)?.into_bytes_with_nul();
        id.resize(CONTAINER_ID_LEN, 0);
        Ok(ContainerID {
            id: id.try_into().unwrap(),
        })
    }
}

#[cfg(feature = "user")]
impl ContainerID {
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.id)
    }
}

#[cfg(not(feature = "user"))]
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
pub struct Path {
    pub path: [u8; PATH_LEN],
}

#[cfg(feature = "user")]
mod user {
    use super::*;

    unsafe impl aya::Pod for ContainerID {}
    unsafe impl aya::Pod for Container {}
    unsafe impl aya::Pod for Process {}
}
