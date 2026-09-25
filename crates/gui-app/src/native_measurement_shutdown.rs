//! Opt-in ACC-03 final observation while the shared device remains owned.
//! This endpoint alone does not establish earlier DRM-client lifecycle coverage.
use crate::App;
use anyhow::{Context, Result, ensure};
use std::io::{Read, Write};
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(2);

impl App {
    pub(crate) fn finish_measurement_observation(&mut self) -> Result<()> {
        let Some(path) = std::env::var_os("DATUM_MEASUREMENT_SHUTDOWN_SOCKET") else {
            return Ok(());
        };
        let runtime = self
            .runtime
            .as_ref()
            .context("measurement runtime absent")?;
        ensure!(
            runtime.application_terminal_shutdown_complete(),
            "measurement endpoint requires completed controlled terminal shutdown"
        );
        ensure!(
            !runtime.device_health.failed(),
            "measurement endpoint cannot certify a failed device"
        );
        let epoch = runtime.measurements.epoch();
        // No further event dispatch or frame submission occurs after run_app.
        // Drain the real shared queue, including implicit uploads, with a bound.
        runtime.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(TIMEOUT),
        })?;
        ensure!(
            self.service_gpu_measurements()?.is_none(),
            "GPU measurement mappings remain pending after shutdown drain"
        );
        let mut stream = connect_observer(Path::new(&path))?;
        final_handoff(&mut stream, epoch, TIMEOUT)
    }
}

fn connect_observer(path: &Path) -> Result<UnixStream> {
    // Local nonblocking connect fails closed if the observer's backlog is full;
    // it must not turn the bounded shutdown into an unbounded connect wait.
    let bytes = path.as_os_str().as_bytes();
    // SAFETY: sockaddr_un is a C integer/byte record; zero initializes padding.
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    ensure!(
        !bytes.is_empty() && !bytes.contains(&0) && bytes.len() < address.sun_path.len(),
        "invalid final observer socket path"
    );
    address.sun_family = libc::AF_UNIX as libc::sa_family_t;
    for (destination, source) in address.sun_path.iter_mut().zip(bytes) {
        *destination = *source as libc::c_char;
    }
    // SAFETY: socket has no pointer arguments; the returned fd is owned below.
    let fd = unsafe {
        libc::socket(
            libc::AF_UNIX,
            libc::SOCK_STREAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
            0,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("create final observer socket");
    }
    // SAFETY: successful socket returned a new, uniquely owned descriptor.
    let owned = unsafe { OwnedFd::from_raw_fd(fd) };
    // SAFETY: address is initialized and remains live for the complete call.
    let result = unsafe {
        libc::connect(
            fd,
            (&address as *const libc::sockaddr_un).cast(),
            std::mem::size_of_val(&address) as libc::socklen_t,
        )
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error()).context("connect final measurement observer");
    }
    let stream = UnixStream::from(owned);
    stream.set_nonblocking(false)?;
    Ok(stream)
}

fn final_handoff(stream: &mut UnixStream, epoch: u64, timeout: Duration) -> Result<()> {
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    // The observer reads final process/cgroup and fdinfo counters after this
    // receipt, closes its common measurement window, then acknowledges. Holding
    // the stream blocks teardown, not rendering: all workload actions have ended.
    writeln!(
        stream,
        "{{\"phase\":\"drained_device_live\",\"pid\":{},\"device_epoch\":{epoch}}}",
        std::process::id()
    )?;
    let mut ack = [0_u8; 1];
    stream
        .read_exact(&mut ack)
        .context("final measurement acknowledgement missing")?;
    ensure!(ack == *b"!", "invalid final measurement acknowledgement");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};

    #[test]
    fn observer_connect_is_explicit_and_missing_endpoint_fails() {
        let path =
            std::env::temp_dir().join(format!("datum-final-observer-{}", uuid::Uuid::new_v4()));
        assert!(connect_observer(&path).is_err());
        let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        let stream = connect_observer(&path).unwrap();
        let (peer, _) = listener.accept().unwrap();
        drop((stream, peer, listener));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn final_receipt_precedes_release_and_requires_acknowledgement() {
        let (mut app, observer) = UnixStream::pair().unwrap();
        let worker = std::thread::spawn(move || final_handoff(&mut app, 37, TIMEOUT));
        let mut observer = BufReader::new(observer);
        let mut line = String::new();
        observer.read_line(&mut line).unwrap();
        let receipt: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(receipt["phase"], "drained_device_live");
        assert_eq!(receipt["device_epoch"], 37);
        assert_eq!(receipt["pid"], std::process::id());
        assert!(
            !worker.is_finished(),
            "device owner must await the observer"
        );
        observer.get_mut().write_all(b"!").unwrap();
        worker.join().unwrap().unwrap();
    }

    #[test]
    fn lost_invalid_and_unresponsive_observers_fail() {
        for reply in [None, Some(b'?')] {
            let (mut app, mut observer) = UnixStream::pair().unwrap();
            let worker = std::thread::spawn(move || final_handoff(&mut app, 1, TIMEOUT));
            if let Some(reply) = reply {
                observer.write_all(&[reply]).unwrap();
            }
            drop(observer);
            assert!(worker.join().unwrap().is_err());
        }
        let (mut app, _observer) = UnixStream::pair().unwrap();
        assert!(final_handoff(&mut app, 1, Duration::from_millis(20)).is_err());
    }
}
