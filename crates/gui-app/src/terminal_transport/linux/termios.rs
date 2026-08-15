//! Deterministic interactive slave-terminal baseline for desktop launches.

use std::{io, mem::MaybeUninit, os::fd::RawFd};

pub(in crate::terminal_transport) fn configure_interactive(fd: RawFd) -> io::Result<()> {
    let mut attributes = read(fd)?;
    let control_characters = attributes.c_cc;
    attributes.c_iflag &= !(libc::IGNBRK
        | libc::IGNPAR
        | libc::PARMRK
        | libc::INPCK
        | libc::ISTRIP
        | libc::INLCR
        | libc::IGNCR
        | libc::IUCLC
        | libc::IXOFF
        | libc::IXANY);
    attributes.c_iflag |= libc::BRKINT | libc::ICRNL | libc::IXON | libc::IUTF8;
    attributes.c_oflag &= !(libc::OCRNL
        | libc::ONOCR
        | libc::ONLRET
        | libc::OLCUC
        | libc::OFILL
        | libc::OFDEL
        | libc::NLDLY
        | libc::CRDLY
        | libc::TABDLY
        | libc::BSDLY
        | libc::VTDLY
        | libc::FFDLY);
    attributes.c_oflag |= libc::OPOST | libc::ONLCR;
    attributes.c_cflag &=
        !(libc::CSIZE | libc::PARENB | libc::CSTOPB | libc::CLOCAL | libc::CRTSCTS);
    attributes.c_cflag |= libc::CS8 | libc::CREAD | libc::HUPCL;
    attributes.c_lflag |= libc::ISIG
        | libc::ICANON
        | libc::ECHO
        | libc::ECHOE
        | libc::ECHOK
        | libc::IEXTEN
        | libc::ECHOCTL
        | libc::ECHOKE;
    attributes.c_lflag &= !(libc::ECHONL
        | libc::ECHOPRT
        | libc::NOFLSH
        | libc::TOSTOP
        | libc::XCASE
        | libc::FLUSHO
        | libc::PENDIN
        | libc::EXTPROC);
    attributes.c_cc = control_characters;
    write(fd, &attributes)
}

fn read(fd: RawFd) -> io::Result<libc::termios> {
    loop {
        let mut attributes = MaybeUninit::<libc::termios>::uninit();
        let result = unsafe { libc::tcgetattr(fd, attributes.as_mut_ptr()) };
        if result == 0 {
            return Ok(unsafe { attributes.assume_init() });
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

fn write(fd: RawFd, attributes: &libc::termios) -> io::Result<()> {
    loop {
        let result = unsafe { libc::tcsetattr(fd, libc::TCSANOW, attributes) };
        if result == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::AsRawFd;

    #[test]
    fn interactive_profile_is_deterministic_without_claiming_p05_control_characters() {
        let pair = super::super::pty::open_pty_pair().unwrap();
        let mut before = read(pair.slave.as_raw_fd()).unwrap();
        before.c_iflag |= libc::IGNBRK
            | libc::IGNPAR
            | libc::PARMRK
            | libc::INPCK
            | libc::ISTRIP
            | libc::INLCR
            | libc::IGNCR
            | libc::IUCLC
            | libc::IXOFF
            | libc::IXANY;
        before.c_oflag |=
            libc::OCRNL | libc::ONOCR | libc::ONLRET | libc::OLCUC | libc::OFILL | libc::OFDEL;
        before.c_lflag |= libc::ECHONL
            | libc::ECHOPRT
            | libc::NOFLSH
            | libc::TOSTOP
            | libc::XCASE
            | libc::FLUSHO
            | libc::PENDIN
            | libc::EXTPROC;
        write(pair.slave.as_raw_fd(), &before).unwrap();
        configure_interactive(pair.slave.as_raw_fd()).unwrap();
        let after = read(pair.slave.as_raw_fd()).unwrap();
        assert_eq!(after.c_cc, before.c_cc);
        assert_eq!(
            after.c_iflag & (libc::BRKINT | libc::ICRNL | libc::IXON | libc::IUTF8),
            libc::BRKINT | libc::ICRNL | libc::IXON | libc::IUTF8
        );
        assert_eq!(
            after.c_oflag & (libc::OPOST | libc::ONLCR),
            libc::OPOST | libc::ONLCR
        );
        assert_eq!(
            after.c_lflag & (libc::ISIG | libc::ICANON | libc::IEXTEN | libc::ECHO),
            libc::ISIG | libc::ICANON | libc::IEXTEN | libc::ECHO
        );
        assert_eq!(
            after.c_iflag
                & (libc::IGNBRK
                    | libc::IGNPAR
                    | libc::PARMRK
                    | libc::INPCK
                    | libc::ISTRIP
                    | libc::INLCR
                    | libc::IGNCR
                    | libc::IUCLC
                    | libc::IXOFF
                    | libc::IXANY),
            0
        );
        assert_eq!(
            after.c_oflag
                & (libc::OCRNL
                    | libc::ONOCR
                    | libc::ONLRET
                    | libc::OLCUC
                    | libc::OFILL
                    | libc::OFDEL),
            0
        );
        assert_eq!(
            after.c_lflag
                & (libc::ECHONL
                    | libc::ECHOPRT
                    | libc::NOFLSH
                    | libc::TOSTOP
                    | libc::XCASE
                    | libc::FLUSHO
                    | libc::PENDIN
                    | libc::EXTPROC),
            0
        );
    }
}
