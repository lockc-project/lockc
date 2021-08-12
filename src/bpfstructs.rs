#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::convert::TryInto;

#[derive(thiserror::Error, Debug)]
pub enum NewBpfstructError {
    #[error("FFI nul error")]
    NulError(#[from] std::ffi::NulError),
}

impl runtime_key {
    pub fn new(comm: &str) -> Result<Self, NewBpfstructError> {
        let mut comm_b = std::ffi::CString::new(comm)?.into_bytes_with_nul();
        comm_b.resize(TASK_COMM_LEN as usize, 0);
        Ok(runtime_key {
            comm: comm_b.try_into()?,
        })
    }
}

impl container_key {
    pub fn new(container_id: &str) -> Result<Self, NewBpfstructError> {
        let mut container_id_b = std::ffi::CString::new(container_id)?.into_bytes_with_nul();
        container_id_b.resize(CONTAINER_ID_MAX_LIMIT as usize, 0);
        Ok(container_key {
            container_id: container_id_b.try_into()?,
        })
    }
}
