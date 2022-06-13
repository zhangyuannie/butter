use libc::{c_int, c_ulong, c_void};
use std::{
    ffi::{CStr, CString, OsStr},
    io,
    os::unix::prelude::OsStrExt,
    path::Path,
};

pub trait ToCString {
    fn to_cstring(&self) -> Option<CString>;
}

impl ToCString for &Path {
    fn to_cstring(&self) -> Option<CString> {
        self.as_os_str().to_cstring()
    }
}
impl ToCString for OsStr {
    fn to_cstring(&self) -> Option<CString> {
        CString::new(self.as_bytes()).ok()
    }
}

pub fn mount<T: AsRef<CStr>>(
    src: T,
    target: T,
    fstype: T,
    flags: c_ulong,
    data: *const c_void,
) -> io::Result<()> {
    let ret = unsafe {
        libc::mount(
            src.as_ref().as_ptr(),
            target.as_ref().as_ptr(),
            fstype.as_ref().as_ptr(),
            flags,
            data,
        )
    };
    match ret {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}

pub fn umount2<T: AsRef<CStr>>(target: T, flags: c_int) -> io::Result<()> {
    let ret = unsafe { libc::umount2(target.as_ref().as_ptr(), flags) };

    match ret {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}
