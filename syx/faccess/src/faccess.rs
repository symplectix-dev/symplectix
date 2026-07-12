//! Determine accessibility of a file descriptor.

use std::ffi::{
    CString,
    OsStr,
};
use std::io;

use bitflags::bitflags;

bitflags! {
    /// Access permissions to be checked or the existence test.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct Mode: libc::c_int {
        #[allow(missing_docs)]
        const EXISTS = libc::F_OK;
        #[allow(missing_docs)]
        const EXECUTE = libc::X_OK;
        #[allow(missing_docs)]
        const WRITE = libc::W_OK;
        #[allow(missing_docs)]
        const READ = libc::R_OK;
    }
}

/// Perform accessibility check.
/// Use the effecitive user ID and group ID by default.
#[bon::builder(finish_fn = at)]
pub fn faccess<P: AsRef<OsStr>>(
    #[builder(finish_fn)] path: P,
    #[builder(default = false, with = || true)] r_ok: bool,
    #[builder(default = false, with = || true)] w_ok: bool,
    #[builder(default = false, with = || true)] x_ok: bool,
    #[builder(default = false, with = || true)] real: bool,
) -> io::Result<()> {
    use std::os::unix::ffi::OsStrExt;

    let cstr = CString::new(path.as_ref().as_bytes())?;
    let path = cstr.as_ptr() as *const libc::c_char;

    let mode = {
        let mut def = Mode::EXISTS;
        if r_ok {
            def |= Mode::READ;
        }
        if w_ok {
            def |= Mode::WRITE;
        }
        if x_ok {
            def |= Mode::EXECUTE;
        }
        def
    };

    let flag = if real {
        0
    } else {
        #[cfg(not(target_os = "android"))]
        use libc::AT_EACCESS;
        // Android does not support AT_EACCESS.
        // https://android.googlesource.com/platform/bionic/+/master/libc/bionic/faccessat.cpp#45
        #[cfg(target_os = "android")]
        const AT_EACCESS: libc::c_int = 0;
        AT_EACCESS
    };

    if unsafe { libc::faccessat(libc::AT_FDCWD, path, mode.bits(), flag) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
