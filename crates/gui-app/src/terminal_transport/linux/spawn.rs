use std::{
    io,
    os::{fd::RawFd, unix::process::CommandExt},
    process::Command,
};

pub(in crate::terminal_transport) fn attach_child_pty(
    command: &mut Command,
    slave_fd: RawFd,
    master_fd: RawFd,
) {
    unsafe {
        command.pre_exec(move || configure_child_pty(slave_fd, master_fd));
    }
}

fn configure_child_pty(slave_fd: RawFd, master_fd: RawFd) -> io::Result<()> {
    if unsafe { libc::setsid() } < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::ioctl(slave_fd, libc::TIOCSCTTY, 0) } < 0 {
        let error = io::Error::last_os_error();
        unsafe { libc::close(slave_fd) };
        return Err(error);
    }
    for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
        if let Err(error) = duplicate_to(slave_fd, fd) {
            unsafe { libc::close(slave_fd) };
            return Err(error);
        }
    }
    unsafe { libc::close(slave_fd) };
    unsafe { libc::close(master_fd) };
    Ok(())
}

fn duplicate_to(source: RawFd, target: RawFd) -> io::Result<()> {
    loop {
        if unsafe { libc::dup2(source, target) } >= 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}
