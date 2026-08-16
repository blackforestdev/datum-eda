//! Deterministic interactive slave-terminal baseline for desktop launches.

use std::{io, mem::MaybeUninit, os::fd::RawFd};

pub(in crate::terminal_transport) fn configure_interactive(fd: RawFd) -> io::Result<()> {
    let mut attributes = read(fd)?;
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
    configure_control_characters(&mut attributes.c_cc);
    write(fd, &attributes)
}

fn configure_control_characters(control: &mut [libc::cc_t; libc::NCCS]) {
    control[libc::VINTR] = 0x03;
    control[libc::VQUIT] = 0x1c;
    control[libc::VERASE] = 0x7f;
    control[libc::VKILL] = 0x15;
    control[libc::VEOF] = 0x04;
    control[libc::VSTART] = 0x11;
    control[libc::VSTOP] = 0x13;
    control[libc::VSUSP] = 0x1a;
    control[libc::VREPRINT] = 0x12;
    control[libc::VWERASE] = 0x17;
    control[libc::VLNEXT] = 0x16;
    control[libc::VDISCARD] = 0x0f;
    control[libc::VEOL] = 0;
    control[libc::VEOL2] = 0;
    control[libc::VMIN] = 1;
    control[libc::VTIME] = 0;
    control[libc::VSWTC] = 0;
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
    fn interactive_profile_is_deterministic_and_pins_p05_control_characters() {
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
        for index in [
            libc::VINTR,
            libc::VQUIT,
            libc::VERASE,
            libc::VKILL,
            libc::VEOF,
            libc::VSTART,
            libc::VSTOP,
            libc::VSUSP,
            libc::VREPRINT,
            libc::VWERASE,
            libc::VLNEXT,
            libc::VDISCARD,
            libc::VEOL,
            libc::VEOL2,
            libc::VMIN,
            libc::VTIME,
            libc::VSWTC,
        ] {
            before.c_cc[index] = 0x55;
        }
        let unrelated_index = (0..libc::NCCS)
            .find(|index| {
                ![
                    libc::VINTR,
                    libc::VQUIT,
                    libc::VERASE,
                    libc::VKILL,
                    libc::VEOF,
                    libc::VSTART,
                    libc::VSTOP,
                    libc::VSUSP,
                    libc::VREPRINT,
                    libc::VWERASE,
                    libc::VLNEXT,
                    libc::VDISCARD,
                    libc::VEOL,
                    libc::VEOL2,
                    libc::VMIN,
                    libc::VTIME,
                    libc::VSWTC,
                ]
                .contains(index)
            })
            .unwrap();
        before.c_cc[unrelated_index] = 0x33;
        write(pair.slave.as_raw_fd(), &before).unwrap();
        configure_interactive(pair.slave.as_raw_fd()).unwrap();
        let after = read(pair.slave.as_raw_fd()).unwrap();
        assert_eq!(after.c_cc[libc::VINTR], 0x03);
        assert_eq!(after.c_cc[libc::VQUIT], 0x1c);
        assert_eq!(after.c_cc[libc::VERASE], 0x7f);
        assert_eq!(after.c_cc[libc::VKILL], 0x15);
        assert_eq!(after.c_cc[libc::VEOF], 0x04);
        assert_eq!(after.c_cc[libc::VSTART], 0x11);
        assert_eq!(after.c_cc[libc::VSTOP], 0x13);
        assert_eq!(after.c_cc[libc::VSUSP], 0x1a);
        assert_eq!(after.c_cc[libc::VREPRINT], 0x12);
        assert_eq!(after.c_cc[libc::VWERASE], 0x17);
        assert_eq!(after.c_cc[libc::VLNEXT], 0x16);
        assert_eq!(after.c_cc[libc::VDISCARD], 0x0f);
        assert_eq!(after.c_cc[libc::VEOL], 0);
        assert_eq!(after.c_cc[libc::VEOL2], 0);
        assert_eq!(after.c_cc[libc::VMIN], 1);
        assert_eq!(after.c_cc[libc::VTIME], 0);
        assert_eq!(after.c_cc[libc::VSWTC], 0);
        assert_eq!(after.c_cc[unrelated_index], 0x33);
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
