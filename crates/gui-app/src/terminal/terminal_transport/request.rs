use std::{
    ffi::{OsStr, OsString},
    io,
    path::PathBuf,
    process::Command,
};

/// Process launch data at the terminal/process boundary.
///
/// Datum context is flattened to environment pairs before it reaches this
/// type. P04 extends construction and I/O policy without coupling transport to
/// GUI or terminal-state structures.
pub(crate) struct TerminalTransportRequest {
    executable: OsString,
    args: Vec<OsString>,
    cwd: PathBuf,
    environment: Vec<(OsString, Option<OsString>)>,
    columns: u16,
    rows: u16,
}

impl TerminalTransportRequest {
    pub(crate) fn new(executable: impl AsRef<OsStr>, cwd: PathBuf) -> Self {
        Self {
            executable: executable.as_ref().to_owned(),
            args: Vec::new(),
            cwd,
            environment: Vec::new(),
            columns: 80,
            rows: 24,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn arg(mut self, value: impl AsRef<OsStr>) -> Self {
        self.args.push(value.as_ref().to_owned());
        self
    }

    #[allow(dead_code)]
    pub(crate) fn args<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(values.into_iter().map(|value| value.as_ref().to_owned()));
        self
    }

    pub(crate) fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.environment
            .push((key.as_ref().to_owned(), Some(value.as_ref().to_owned())));
        self
    }

    #[allow(dead_code)]
    pub(crate) fn env_remove(mut self, key: impl AsRef<OsStr>) -> Self {
        self.environment.push((key.as_ref().to_owned(), None));
        self
    }

    #[allow(dead_code)]
    pub(crate) fn initial_size(mut self, columns: u16, rows: u16) -> Self {
        self.columns = columns.max(1);
        self.rows = rows.max(1);
        self
    }

    pub(super) fn spawn_failure_context(&self) -> String {
        format!(
            "spawn PTY terminal shell {} in {}",
            self.executable.to_string_lossy(),
            self.cwd.display()
        )
    }

    pub(super) fn validate_cwd(&self) -> io::Result<()> {
        if self.cwd.metadata()?.is_dir() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                "terminal cwd is not a directory",
            ))
        }
    }

    pub(crate) fn into_command(self) -> (Command, u16, u16) {
        let mut command = Command::new(self.executable);
        command.args(self.args).current_dir(self.cwd);
        for (key, value) in self.environment {
            if let Some(value) = value {
                command.env(key, value);
            } else {
                command.env_remove(key);
            }
        }
        (command, self.columns, self.rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn request_materializes_only_process_launch_data() {
        let request = TerminalTransportRequest::new("/bin/sh", PathBuf::from("/tmp"))
            .arg("-l")
            .args(["", "--literal;$(ignored)"])
            .env("TERM", "xterm-256color")
            .env("REMOVE_ME", "before")
            .env_remove("REMOVE_ME")
            .env("NON_UTF8", OsString::from_vec(vec![b'x', 0xff]))
            .initial_size(120, 36);
        let (command, columns, rows) = request.into_command();
        assert_eq!(command.get_program(), "/bin/sh");
        assert_eq!(
            command.get_current_dir(),
            Some(std::path::Path::new("/tmp"))
        );
        assert!(
            command.get_envs().any(|(key, value)| {
                key == "TERM" && value == Some(OsStr::new("xterm-256color"))
            })
        );
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [
                OsStr::new("-l"),
                OsStr::new(""),
                OsStr::new("--literal;$(ignored)")
            ]
        );
        assert!(
            command
                .get_envs()
                .any(|(key, value)| key == "REMOVE_ME" && value.is_none())
        );
        let non_utf8 = OsString::from_vec(vec![b'x', 0xff]);
        assert!(
            command
                .get_envs()
                .any(|(key, value)| { key == "NON_UTF8" && value == Some(non_utf8.as_os_str()) })
        );
        assert_eq!((columns, rows), (120, 36));
    }
}
