#![allow(dead_code)]
#![allow(deref_nullptr)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewBpfstructError {
    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),

    #[error("could not convert Vec<u8> to CString")]
    VecU8CStringConv,
}

impl accessed_path {
    /// Creates a new accessed_path instance and converts the given Rust string
    /// into C fixed-size char array.
    pub fn new(path: &str) -> Result<Self, NewBpfstructError> {
        let mut path_b = CString::new(path)?.into_bytes_with_nul();
        path_b.resize(PATH_LEN as usize, 0);
        Ok(accessed_path {
            path: path_b
                .try_into()
                .map_err(|_| NewBpfstructError::VecU8CStringConv)?,
        })
    }
}

impl container_id {
    /// Creates a new container_id instance and converts the given Rust string
    /// into C fixed size char array.
    pub fn new(id: &str) -> Result<Self, NewBpfstructError> {
        let mut id_b = CString::new(id)?.into_bytes_with_nul();
        id_b.resize(CONTAINER_ID_LIMIT as usize, 0);
        Ok(container_id {
            id: id_b
                .try_into()
                .map_err(|_| NewBpfstructError::VecU8CStringConv)?,
        })
    }
}

unsafe impl aya::Pod for accessed_path {}
unsafe impl aya::Pod for container {}
unsafe impl aya::Pod for container_id {}
unsafe impl aya::Pod for process {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessed_path_new() {
        let ap1 = accessed_path::new("/foo/bar").unwrap();
        assert_eq!(
            &ap1.path,
            b"/foo/bar\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
        );

        let ap2 = accessed_path::new("/ayy/lmao").unwrap();
        assert_eq!(
            &ap2.path,
            b"/ayy/lmao\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
        );

        let ap3 = accessed_path::new(
            "/this/is/gonna/be/a/veeeeeeeeery/looooooooooooooooong/paaaaaaaaaaaaaaaaaaaath",
        )
        .unwrap();
        assert_eq!(
            &ap3.path,
            b"/this/is/gonna/be/a/veeeeeeeeery/looooooooooooooooong/paaaaaaaaa",
        );
    }
}
