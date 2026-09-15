use crate::terminal_transport::launch_error::{TerminalLaunchError, TerminalLaunchStage};
use std::{
    ffi::CStr,
    fs::File,
    io,
    os::fd::{AsRawFd, FromRawFd, RawFd},
};

pub(in crate::terminal_transport) struct PtyPair {
    pub(in crate::terminal_transport) master: File,
    pub(in crate::terminal_transport) master_fd: RawFd,
    pub(in crate::terminal_transport) slave: File,
}

pub(in crate::terminal_transport) fn open_pty_pair() -> Result<PtyPair, TerminalLaunchError> {
    let master_fd = unsafe {
        libc::open(
            c"/dev/ptmx".as_ptr(),
            libc::O_RDWR | libc::O_NOCTTY | libc::O_CLOEXEC | libc::O_NONBLOCK,
        )
    };
    if master_fd < 0 {
        return Err(TerminalLaunchError::new(
            TerminalLaunchStage::AllocateMaster,
            io::Error::last_os_error(),
        ));
    }
    let master = super::io::normalize_above_stdio(unsafe { File::from_raw_fd(master_fd) })
        .map_err(|error| TerminalLaunchError::new(TerminalLaunchStage::NormalizeMaster, error))?;
    let master_fd = master.as_raw_fd();
    if unsafe { libc::grantpt(master_fd) } != 0 {
        return Err(TerminalLaunchError::new(
            TerminalLaunchStage::GrantSlave,
            io::Error::last_os_error(),
        ));
    }
    if unsafe { libc::unlockpt(master_fd) } != 0 {
        return Err(TerminalLaunchError::new(
            TerminalLaunchStage::UnlockSlave,
            io::Error::last_os_error(),
        ));
    }
    let slave_path = slave_path(master_fd)?;
    let slave_fd = unsafe {
        libc::open(
            slave_path.as_ptr().cast(),
            libc::O_RDWR | libc::O_NOCTTY | libc::O_CLOEXEC,
        )
    };
    if slave_fd < 0 {
        return Err(TerminalLaunchError::new(
            TerminalLaunchStage::OpenSlave,
            io::Error::last_os_error(),
        ));
    }
    let slave = super::io::normalize_above_stdio(unsafe { File::from_raw_fd(slave_fd) })
        .map_err(|error| TerminalLaunchError::new(TerminalLaunchStage::NormalizeSlave, error))?;
    Ok(PtyPair {
        master,
        master_fd,
        slave,
    })
}

fn slave_path(master_fd: RawFd) -> Result<Vec<u8>, TerminalLaunchError> {
    let mut buffer = vec![0 as libc::c_char; 128];
    loop {
        let rc = unsafe { libc::ptsname_r(master_fd, buffer.as_mut_ptr(), buffer.len()) };
        if rc == 0 {
            let path = unsafe { CStr::from_ptr(buffer.as_ptr()) };
            return Ok(path.to_bytes_with_nul().to_vec());
        }
        if rc == libc::ERANGE && buffer.len() < 4096 {
            buffer.resize(buffer.len() * 2, 0);
            continue;
        }
        return Err(TerminalLaunchError::new(
            TerminalLaunchStage::ResolveSlavePath,
            io::Error::from_raw_os_error(rc),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pty_descriptors_are_above_stdio_cloexec_and_master_nonblocking() {
        let pair = open_pty_pair().unwrap();
        assert!(pair.master.as_raw_fd() > libc::STDERR_FILENO);
        assert!(pair.slave.as_raw_fd() > libc::STDERR_FILENO);
        assert_ne!(
            super::super::io::descriptor_flags(pair.master.as_raw_fd()).unwrap() & libc::FD_CLOEXEC,
            0
        );
        assert_ne!(
            super::super::io::descriptor_flags(pair.slave.as_raw_fd()).unwrap() & libc::FD_CLOEXEC,
            0
        );
        let master_flags = unsafe { libc::fcntl(pair.master.as_raw_fd(), libc::F_GETFL) };
        let slave_flags = unsafe { libc::fcntl(pair.slave.as_raw_fd(), libc::F_GETFL) };
        assert_ne!(master_flags & libc::O_NONBLOCK, 0);
        assert_eq!(slave_flags & libc::O_NONBLOCK, 0);
    }

    #[test]
    fn low_fd_helper() {
        let Some(report_path) = std::env::var_os("DATUM_LOW_FD_REPORT") else {
            return;
        };
        let report = crate::terminal_transport::linux::io::normalize_above_stdio(
            std::fs::OpenOptions::new()
                .write(true)
                .open(report_path)
                .unwrap(),
        )
        .unwrap();
        for fd in 0..=2 {
            unsafe { libc::close(fd) };
        }
        let request = crate::terminal_transport::TerminalTransportRequest::new(
            "/bin/sh",
            std::path::PathBuf::from("/tmp"),
        )
        .args([
            "-c",
            "test -t 0 && test -t 1 && test -t 2 && \
             test \"$(readlink /proc/$$/fd/0)\" = \"$(readlink /proc/$$/fd/1)\" && \
             test \"$(readlink /proc/$$/fd/1)\" = \"$(readlink /proc/$$/fd/2)\" && \
             ! ls -l /proc/$$/fd | grep -q /dev/ptmx && printf low-fd-ok",
        ]);
        let session = crate::terminal_transport::prepare_terminal_transport(request)
            .unwrap()
            .start(crate::terminal_transport::TerminalWakeGate::new(None));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let mut output = Vec::new();
        while std::time::Instant::now() < deadline {
            match session.recv_event_timeout(std::time::Duration::from_millis(20)) {
                Ok(crate::terminal_transport::TerminalTransportEvent::Output(bytes)) => {
                    output.extend(bytes);
                    if output
                        .windows(b"low-fd-ok".len())
                        .any(|part| part == b"low-fd-ok")
                    {
                        break;
                    }
                }
                Ok(crate::terminal_transport::TerminalTransportEvent::Exited(_)) => {}
                Ok(crate::terminal_transport::TerminalTransportEvent::Error(_)) => break,
                Err(_) => {}
            }
        }
        let bytes = if output
            .windows(b"low-fd-ok".len())
            .any(|part| part == b"low-fd-ok")
        {
            b"ok".as_slice()
        } else {
            b"bad".as_slice()
        };
        unsafe { libc::write(report.as_raw_fd(), bytes.as_ptr().cast(), bytes.len()) };
    }

    #[test]
    fn pty_allocation_normalizes_descriptors_when_stdio_is_closed() {
        let report_path = std::env::temp_dir().join(format!("datum-low-fd-{}", std::process::id()));
        std::fs::File::create(&report_path).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "terminal_transport::linux::pty::tests::low_fd_helper",
            ])
            .env("DATUM_LOW_FD_REPORT", &report_path)
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(std::fs::read(&report_path).unwrap(), b"ok");
        let _ = std::fs::remove_file(report_path);
    }
}
