//! Linux PTY descriptor flags and readiness.

use std::{
    fs::File,
    io,
    os::fd::{AsRawFd, FromRawFd, RawFd},
};

#[cfg_attr(not(test), allow(dead_code))]
pub(in crate::terminal_transport) fn descriptor_flags(fd: RawFd) -> io::Result<libc::c_int> {
    loop {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if flags >= 0 {
            return Ok(flags);
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

pub(in crate::terminal_transport) fn normalize_above_stdio(file: File) -> io::Result<File> {
    if file.as_raw_fd() > libc::STDERR_FILENO {
        return Ok(file);
    }
    let duplicate = loop {
        let fd = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
        if fd >= 0 {
            break fd;
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    };
    drop(file);
    Ok(unsafe { File::from_raw_fd(duplicate) })
}

pub(in crate::terminal_transport) fn wait_readable(fd: RawFd) -> io::Result<libc::c_short> {
    wait(fd, libc::POLLIN)
}

pub(in crate::terminal_transport) fn wait_writable(fd: RawFd) -> io::Result<libc::c_short> {
    wait(fd, libc::POLLOUT)
}

pub(in crate::terminal_transport) fn is_hung_up(fd: RawFd) -> io::Result<bool> {
    let mut descriptor = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    loop {
        let result = unsafe { libc::poll(&mut descriptor, 1, 0) };
        if result >= 0 {
            return Ok(descriptor.revents & libc::POLLHUP != 0);
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

fn wait(fd: RawFd, events: libc::c_short) -> io::Result<libc::c_short> {
    let mut descriptor = libc::pollfd {
        fd,
        events,
        revents: 0,
    };
    loop {
        let result = unsafe { libc::poll(&mut descriptor, 1, -1) };
        if result > 0 {
            return Ok(descriptor.revents);
        }
        if result == 0 {
            continue;
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}
