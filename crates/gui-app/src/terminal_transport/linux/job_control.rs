use super::process_session::{self, ProcessIdentity};
use anyhow::{Context, Result};
use std::{io, os::fd::RawFd};

pub(in crate::terminal_transport) fn signal_owned_process_group(
    representative: ProcessIdentity,
    expected_session_id: libc::pid_t,
    signal: libc::c_int,
) -> io::Result<()> {
    if representative.pid <= 1 || representative.process_group_id <= 1 || expected_session_id <= 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unsafe terminal process identity",
        ));
    }
    let current = process_session::read_process_identity(representative.pid).map_err(|error| {
        if error.is_process_gone() {
            io::Error::from_raw_os_error(libc::ESRCH)
        } else {
            io::Error::other(error.to_string())
        }
    })?;
    if current != representative || current.session_id != expected_session_id {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "terminal process identity changed before signal",
        ));
    }
    loop {
        if unsafe { libc::kill(-current.process_group_id, signal) } == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

pub(in crate::terminal_transport) fn resize(master_fd: RawFd, cols: u16, rows: u16) -> Result<()> {
    resize_fd(master_fd, cols, rows).context("resize terminal PTY")
}

pub(in crate::terminal_transport) fn resize_fd(fd: RawFd, cols: u16, rows: u16) -> io::Result<()> {
    let size = libc::winsize {
        ws_row: rows.max(1),
        ws_col: cols.max(1),
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    loop {
        let rc = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &size) };
        if rc == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}
