use anyhow::{Context, Result};
use std::{io, os::fd::RawFd};

pub(in crate::terminal_transport) fn signal_process_group(
    process_group_id: libc::pid_t,
    signal: libc::c_int,
    context: &'static str,
) -> Result<()> {
    let rc = unsafe { libc::kill(-process_group_id, signal) };
    if rc < 0 {
        return Err(io::Error::last_os_error()).context(context);
    }
    Ok(())
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
    let rc = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &size) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
