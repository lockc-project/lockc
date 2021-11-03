use libc::pid_t;

#[no_mangle]
#[inline(never)]
pub extern "C" fn add_container(_retp: *mut i32, _container_id: u32, _pid: pid_t, _policy: i32) {}

#[no_mangle]
#[inline(never)]
pub extern "C" fn delete_container(_retp: *mut i32, _container_id: u32) {}

#[no_mangle]
#[inline(never)]
pub extern "C" fn add_process(_retp: *mut i32, _container_id: u32, _pid: pid_t) {}
