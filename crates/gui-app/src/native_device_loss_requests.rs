//! Opt-in backend fault requests for one long-lived qualification process.
//! The harness publishes integers 1..=20 atomically, then supplies ordinary
//! native input to wake the existing event loop. No polling timer is installed.
use anyhow::{Context, Result, ensure};
use std::{fs::OpenOptions, io::Read, os::unix::fs::OpenOptionsExt, path::PathBuf};

pub(super) struct LossRequests {
    path: PathBuf,
    accepted: u8,
}

impl LossRequests {
    pub(super) fn new(path: PathBuf) -> Self {
        Self { path, accepted: 0 }
    }

    pub(super) fn accepted(&self) -> u8 {
        self.accepted
    }

    pub(super) fn take_request(&mut self) -> Result<bool> {
        // Reject special files/symlinks without blocking the native loop on a
        // FIFO opened accidentally by the harness. This path is diagnostics only.
        let file = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW)
            .open(&self.path)
        {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) => return Err(err).context("open device loss request"),
        };
        ensure!(file.metadata()?.is_file(), "request must be a regular file");
        let mut value = String::new();
        file.take(32).read_to_string(&mut value)?;
        ensure!(value.len() < 32, "device loss request is too long");
        let sequence: u8 = value.trim().parse().context("request sequence integer")?;
        ensure!(sequence <= 20, "at most20 device loss requests per process");
        if sequence == self.accepted {
            return Ok(false);
        }
        ensure!(
            sequence == self.accepted + 1,
            "request sequence must advance by one"
        );
        self.accepted = sequence;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_are_sequential_bounded_and_never_replayed() {
        let path = std::env::temp_dir().join(format!(
            "datum-device-loss-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut requests = LossRequests::new(path.clone());
        assert!(!requests.take_request().unwrap());
        std::fs::write(&path, "0").unwrap();
        assert!(!requests.take_request().unwrap());
        for sequence in 1..=20 {
            std::fs::write(&path, (sequence + 1).to_string()).unwrap();
            assert!(requests.take_request().is_err());
            assert_eq!(requests.accepted(), sequence - 1);
            std::fs::write(&path, format!("{sequence}\n")).unwrap();
            assert!(requests.take_request().unwrap());
            assert!(!requests.take_request().unwrap());
        }
        for invalid in ["0", "19", "21", "256", "", "garbage", &"1".repeat(32)] {
            std::fs::write(&path, invalid).unwrap();
            assert!(requests.take_request().is_err());
            assert_eq!(requests.accepted(), 20);
        }
        std::fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink("/dev/null", &path).unwrap();
        assert!(requests.take_request().is_err());
        std::fs::remove_file(path).unwrap();
    }
}
