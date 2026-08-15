use super::{
    control::ControlBacklog, event::TerminalIoError, limits, linux::io as descriptor_io,
    output::OutputBacklog,
};
use std::{
    fs::File,
    io::{self, Read},
    os::fd::AsRawFd,
    sync::Arc,
    thread,
};

pub(super) fn spawn_reader(reader: File, output: Arc<OutputBacklog>, control: Arc<ControlBacklog>) {
    thread::spawn(move || read_output(reader, output, control));
}

fn read_output(mut reader: File, output: Arc<OutputBacklog>, control: Arc<ControlBacklog>) {
    read_output_from(&mut FileReader(&mut reader), output, control);
}

trait PtyReadIo {
    fn wait_readable(&mut self) -> io::Result<libc::c_short>;
    fn read_bytes(&mut self, buffer: &mut [u8]) -> io::Result<usize>;
    fn is_hung_up(&mut self) -> io::Result<bool>;
}

struct FileReader<'a>(&'a mut File);

impl PtyReadIo for FileReader<'_> {
    fn wait_readable(&mut self) -> io::Result<libc::c_short> {
        descriptor_io::wait_readable(self.0.as_raw_fd())
    }

    fn read_bytes(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.0.read(buffer)
    }

    fn is_hung_up(&mut self) -> io::Result<bool> {
        descriptor_io::is_hung_up(self.0.as_raw_fd())
    }
}

fn read_output_from<R: PtyReadIo>(
    reader: &mut R,
    output: Arc<OutputBacklog>,
    control: Arc<ControlBacklog>,
) {
    let mut buffer = vec![0_u8; limits::MAX_OUTPUT_CHUNK_BYTES];
    loop {
        let revents = match reader.wait_readable() {
            Ok(revents) => revents,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                control.reader_failed(TerminalIoError::read(&error));
                break;
            }
        };
        if revents & libc::POLLNVAL != 0 {
            let error = io::Error::from_raw_os_error(libc::EBADF);
            control.reader_failed(TerminalIoError::read(&error));
            return;
        }
        let saw_hangup = revents & libc::POLLHUP != 0;
        loop {
            let Some(permit) = output.reserve() else {
                return;
            };
            match reader.read_bytes(&mut buffer) {
                Ok(0) => return,
                Ok(count) => {
                    if !permit.publish(buffer[..count].to_vec().into_boxed_slice()) {
                        return;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error)
                    if error.raw_os_error() == Some(libc::EIO)
                        && (saw_hangup || reader.is_hung_up().unwrap_or(false)) =>
                {
                    return;
                }
                Err(error) => {
                    control.reader_failed(TerminalIoError::read(&error));
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_transport::TerminalWakeGate;
    use std::{io::Write, os::fd::FromRawFd};

    enum ReadStep {
        Wait(io::Result<libc::c_short>),
        Read(io::Result<Vec<u8>>),
        HungUp(io::Result<bool>),
    }

    struct ScriptedReader {
        steps: std::collections::VecDeque<ReadStep>,
        reads: usize,
    }

    impl PtyReadIo for ScriptedReader {
        fn wait_readable(&mut self) -> io::Result<libc::c_short> {
            match self.steps.pop_front().expect("wait step") {
                ReadStep::Wait(result) => result,
                _ => panic!("expected wait step"),
            }
        }

        fn read_bytes(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.reads += 1;
            match self.steps.pop_front().expect("read step") {
                ReadStep::Read(Ok(bytes)) => {
                    buffer[..bytes.len()].copy_from_slice(&bytes);
                    Ok(bytes.len())
                }
                ReadStep::Read(Err(error)) => Err(error),
                _ => panic!("expected read step"),
            }
        }

        fn is_hung_up(&mut self) -> io::Result<bool> {
            match self.steps.pop_front().expect("hangup step") {
                ReadStep::HungUp(result) => result,
                _ => panic!("expected hangup step"),
            }
        }
    }

    fn scripted_backlogs() -> (Arc<OutputBacklog>, Arc<ControlBacklog>) {
        let wake = TerminalWakeGate::new(None);
        (
            Arc::new(OutputBacklog::new(wake.clone())),
            Arc::new(ControlBacklog::new(wake)),
        )
    }

    #[test]
    fn reader_preserves_opaque_bytes_through_the_real_read_loop() {
        let mut descriptors = [-1; 2];
        assert_eq!(
            unsafe { libc::pipe2(descriptors.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) },
            0
        );
        let reader = unsafe { File::from_raw_fd(descriptors[0]) };
        let mut writer = unsafe { File::from_raw_fd(descriptors[1]) };
        let expected = [
            0, 0x1b, b'[', b'3', b'1', b'm', 0x1b, b']', b'8', b';', 0xff, 0xfe,
        ];
        writer.write_all(&expected).unwrap();
        drop(writer);
        let wake = TerminalWakeGate::new(None);
        let output = Arc::new(OutputBacklog::new(wake.clone()));
        let control = Arc::new(ControlBacklog::new(wake));
        read_output(reader, output.clone(), control);
        assert_eq!(output.try_pop(), Some(expected.to_vec()));
    }

    #[test]
    fn reader_retries_eintr_and_would_block_without_losing_bytes_or_spinning() {
        let (output, control) = scripted_backlogs();
        let mut reader = ScriptedReader {
            steps: std::collections::VecDeque::from([
                ReadStep::Wait(Err(io::Error::from(io::ErrorKind::Interrupted))),
                ReadStep::Wait(Ok(libc::POLLIN)),
                ReadStep::Read(Ok(b"abc".to_vec())),
                ReadStep::Read(Err(io::Error::from(io::ErrorKind::WouldBlock))),
                ReadStep::Wait(Ok(libc::POLLIN)),
                ReadStep::Read(Ok(Vec::new())),
            ]),
            reads: 0,
        };
        read_output_from(&mut reader, output.clone(), control.clone());
        assert_eq!(reader.reads, 3);
        assert_eq!(output.try_pop(), Some(b"abc".to_vec()));
        assert!(!control.has_pending());
        assert!(
            output.reserve().is_some(),
            "all non-data permits were released"
        );
    }

    #[test]
    fn reader_drains_hup_tail_then_accepts_correlated_eio_as_eof() {
        let (output, control) = scripted_backlogs();
        let mut reader = ScriptedReader {
            steps: std::collections::VecDeque::from([
                ReadStep::Wait(Ok(libc::POLLIN | libc::POLLHUP)),
                ReadStep::Read(Ok(b"tail".to_vec())),
                ReadStep::Read(Err(io::Error::from_raw_os_error(libc::EIO))),
            ]),
            reads: 0,
        };
        read_output_from(&mut reader, output.clone(), control.clone());
        assert_eq!(output.try_pop(), Some(b"tail".to_vec()));
        assert!(!control.has_pending());
    }

    #[test]
    fn reader_reports_uncorrelated_eio_and_invalid_descriptor_once() {
        for steps in [
            std::collections::VecDeque::from([
                ReadStep::Wait(Ok(libc::POLLIN)),
                ReadStep::Read(Err(io::Error::from_raw_os_error(libc::EIO))),
                ReadStep::HungUp(Ok(false)),
            ]),
            std::collections::VecDeque::from([ReadStep::Wait(Ok(libc::POLLNVAL))]),
        ] {
            let (output, control) = scripted_backlogs();
            let mut reader = ScriptedReader { steps, reads: 0 };
            read_output_from(&mut reader, output.clone(), control.clone());
            assert!(!output.has_pending());
            assert!(matches!(
                control.try_pop(),
                Some(super::super::TerminalTransportEvent::Error(error))
                    if error.stage == super::super::event::TerminalIoStage::Read
            ));
            assert!(control.try_pop().is_none());
            assert!(output.reserve().is_some(), "failure released its permit");
        }
    }
}
