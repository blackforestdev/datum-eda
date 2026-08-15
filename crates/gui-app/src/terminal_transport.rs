//! Process transport boundary for Datum's embedded terminal.
//!
//! PTY-01 inventory: allocation, slave setup and window resize are transport
//! concerns; process launch/read/write/wait still live in `terminal_process`
//! until PTY-02 moves them behind `portable-pty`. Terminal parsing, cells,
//! selection, chrome and Datum context projections must never enter this module.

use anyhow::{Context, Result};
use portable_pty::PtySize;
use std::{
    ffi::CStr,
    fs::File,
    io,
    os::fd::{FromRawFd, RawFd},
};

pub(crate) const INITIAL_TERMINAL_SIZE: PtySize = PtySize {
    rows: 24,
    cols: 80,
    pixel_width: 0,
    pixel_height: 0,
};

/// Temporary Unix transport handle retained only for the PTY-02 migration.
/// Keeping every raw descriptor operation here makes the replacement surface
/// explicit and prevents the session/core layers from acquiring new PTY calls.
pub(super) struct LegacyUnixPty {
    pub(super) master: File,
    pub(super) master_fd: RawFd,
    pub(super) slave_path: Vec<u8>,
}

pub(super) fn open_legacy_unix_pty() -> Result<LegacyUnixPty> {
    let master_fd = unsafe { libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY) };
    if master_fd < 0 {
        return Err(io::Error::last_os_error()).context("posix_openpt");
    }
    if unsafe { libc::grantpt(master_fd) } != 0 {
        let error = io::Error::last_os_error();
        unsafe { libc::close(master_fd) };
        return Err(error).context("grantpt");
    }
    if unsafe { libc::unlockpt(master_fd) } != 0 {
        let error = io::Error::last_os_error();
        unsafe { libc::close(master_fd) };
        return Err(error).context("unlockpt");
    }
    let slave_path = slave_path(master_fd)?;
    let master = unsafe { File::from_raw_fd(master_fd) };
    Ok(LegacyUnixPty {
        master,
        master_fd,
        slave_path,
    })
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

pub(super) fn configure_legacy_unix_child(slave_path: &[u8], master_fd: RawFd) -> io::Result<()> {
    if unsafe { libc::setsid() } < 0 {
        return Err(io::Error::last_os_error());
    }
    let slave_fd = unsafe { libc::open(slave_path.as_ptr().cast(), libc::O_RDWR) };
    if slave_fd < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::ioctl(slave_fd, libc::TIOCSCTTY, 0) } < 0 {
        let error = io::Error::last_os_error();
        unsafe { libc::close(slave_fd) };
        return Err(error);
    }
    for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
        if unsafe { libc::dup2(slave_fd, fd) } < 0 {
            let error = io::Error::last_os_error();
            unsafe { libc::close(slave_fd) };
            return Err(error);
        }
    }
    if slave_fd > libc::STDERR_FILENO {
        unsafe { libc::close(slave_fd) };
    }
    unsafe { libc::close(master_fd) };
    Ok(())
}

pub(crate) fn resize_legacy_unix_pty(master_fd: RawFd, size: PtySize) -> Result<()> {
    let size = libc::winsize {
        ws_row: size.rows.max(1),
        ws_col: size.cols.max(1),
        ws_xpixel: size.pixel_width,
        ws_ypixel: size.pixel_height,
    };
    let rc = unsafe { libc::ioctl(master_fd, libc::TIOCSWINSZ, &size) };
    if rc < 0 {
        return Err(io::Error::last_os_error()).context("resize terminal PTY");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_size_uses_portable_pty_transport_shape() {
        assert_eq!(INITIAL_TERMINAL_SIZE, PtySize::default());
    }
}
