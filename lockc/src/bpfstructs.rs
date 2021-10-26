#![allow(dead_code)]
#![allow(deref_nullptr)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use byteorder::{NativeEndian, WriteBytesExt};

#[derive(thiserror::Error, Debug)]
pub enum NewBpfstructError {
    #[error("FFI nul error")]
    NulError(#[from] std::ffi::NulError),
}

#[derive(thiserror::Error, Debug)]
pub enum MapOperationError {
    #[error("could not convert the key to a byte array")]
    ByteWriteError(#[from] std::io::Error),

    #[error("libbpf error")]
    LibbpfError(#[from] libbpf_rs::Error),
}

/// Deletes an entry from the given map under the given key.
pub fn map_delete(map: &mut libbpf_rs::Map, key: u32) -> Result<(), MapOperationError> {
    let mut key_b = vec![];
    key_b.write_u32::<NativeEndian>(key)?;

    map.delete(&key_b)?;

    Ok(())
}

pub trait BpfStruct {
    /// Updates the given map with an entry under the given key and a value
    /// with a binary representation of the struct.
    fn map_update(&self, map: &mut libbpf_rs::Map, key: u32) -> Result<(), MapOperationError> {
        let mut key_b = vec![];
        key_b.write_u32::<NativeEndian>(key)?;

        let val_b = unsafe { plain::as_bytes(self) };

        map.update(&key_b, val_b, libbpf_rs::MapFlags::empty())?;

        Ok(())
    }
}

impl BpfStruct for container {}
impl BpfStruct for process {}
impl BpfStruct for accessed_path {}

impl accessed_path {
    /// Creates a new accessed_path instance and converts the given Rust string
    /// into C fixed-size char array.
    pub fn new(path: &str) -> Result<Self, NewBpfstructError> {
        let mut path_b = std::ffi::CString::new(path)?.into_bytes_with_nul();
        path_b.resize(PATH_LEN as usize, 0);
        Ok(accessed_path {
            path: path_b.try_into().unwrap(),
        })
    }
}

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
