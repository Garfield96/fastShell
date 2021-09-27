// Adapted from https://github.com/cptpcrd/simple_libc/blob/master/src/xattr.rs
use std::ffi::{CStr, CString, OsStr};
use std::io;
use std::os::unix::prelude::*;

enum Target {
    File(CString),
    Link(CString),
    Fd(i32),
}

fn convert_neg(ret: isize, res: isize) -> io::Result<isize> {
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(res)
    }
}

impl Target {
    fn build_from_path<P: AsRef<OsStr>>(path: P, follow_links: bool) -> io::Result<Self> {
        let c_path = CString::new(path.as_ref().as_bytes())?;

        Ok(if follow_links {
            Self::File(c_path)
        } else {
            Self::Link(c_path)
        })
    }

    fn getxattr(&self, name: &CStr, value: &mut [u8]) -> io::Result<usize> {
        unsafe {
            #[cfg(target_os = "linux")]
            let res = match self {
                Self::File(path) => libc::getxattr(
                    path.as_ptr(),
                    name.as_ptr(),
                    value.as_mut_ptr() as *mut libc::c_void,
                    value.len(),
                ),
                Self::Link(path) => libc::lgetxattr(
                    path.as_ptr(),
                    name.as_ptr(),
                    value.as_mut_ptr() as *mut libc::c_void,
                    value.len(),
                ),
                Self::Fd(fd) => libc::fgetxattr(
                    *fd,
                    name.as_ptr(),
                    value.as_mut_ptr() as *mut libc::c_void,
                    value.len(),
                ),
            };

            let n = convert_neg(res, res)?;
            Ok(n as usize)
        }
    }
}

fn getxattr_impl(target: Target, name: &CStr) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let init_size = target.getxattr(&name, &mut buf)?;

    if init_size == 0 {
        // Empty
        return Ok(buf);
    }

    buf.resize(init_size, 0);

    loop {
        match target.getxattr(&name, &mut buf) {
            Ok(n) => {
                buf.resize(n as usize, 0);

                return Ok(buf);
            }
            Err(e) => {
                if e.raw_os_error() != Some(libc::ERANGE) || buf.len() > init_size * 4 {
                    return Err(e);
                }
            }
        }

        buf.resize(buf.len() * 2, 0);
    }
}

pub fn getxattr<P: AsRef<OsStr>, N: AsRef<OsStr>>(
    path: P,
    name: N,
    follow_links: bool,
) -> io::Result<Vec<u8>> {
    let c_name = CString::new(name.as_ref().as_bytes())?;

    getxattr_impl(Target::build_from_path(path, follow_links)?, &c_name)
}
