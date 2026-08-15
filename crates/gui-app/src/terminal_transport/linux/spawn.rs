use std::{
    io,
    os::{fd::RawFd, unix::process::CommandExt},
    process::Command,
};

pub(in crate::terminal_transport) fn attach_child_pty(
    command: &mut Command,
    slave_path: Vec<u8>,
    master_fd: RawFd,
) {
    unsafe {
        command.pre_exec(move || configure_child_pty(&slave_path, master_fd));
    }
}

fn configure_child_pty(slave_path: &[u8], master_fd: RawFd) -> io::Result<()> {
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
