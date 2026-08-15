use anyhow::{Context, Result};
use std::{
    ffi::CStr,
    fs::File,
    io,
    os::fd::{FromRawFd, RawFd},
};

pub(in crate::terminal_transport) struct PtyPair {
    pub(in crate::terminal_transport) master: File,
    pub(in crate::terminal_transport) master_fd: RawFd,
    pub(in crate::terminal_transport) slave_path: Vec<u8>,
}

pub(in crate::terminal_transport) fn open_pty_pair() -> Result<PtyPair> {
    let master_fd = unsafe { libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY) };
    if master_fd < 0 {
        return Err(io::Error::last_os_error()).context("posix_openpt");
    }
    if unsafe { libc::grantpt(master_fd) } != 0 {
        return close_with_error(master_fd, "grantpt");
    }
    if unsafe { libc::unlockpt(master_fd) } != 0 {
        return close_with_error(master_fd, "unlockpt");
    }
    let slave_path = slave_path(master_fd)?;
    let master = unsafe { File::from_raw_fd(master_fd) };
    Ok(PtyPair {
        master,
        master_fd,
        slave_path,
    })
}

fn close_with_error(master_fd: RawFd, operation: &'static str) -> Result<PtyPair> {
    let error = io::Error::last_os_error();
    unsafe { libc::close(master_fd) };
    Err(error).context(operation)
}

fn slave_path(master_fd: RawFd) -> Result<Vec<u8>> {
    let mut buffer = [0 as libc::c_char; 128];
    let rc = unsafe { libc::ptsname_r(master_fd, buffer.as_mut_ptr(), buffer.len()) };
    if rc != 0 {
        return Err(io::Error::from_raw_os_error(rc)).context("ptsname_r");
    }
    let path = unsafe { CStr::from_ptr(buffer.as_ptr()) };
    Ok(path.to_bytes_with_nul().to_vec())
}
