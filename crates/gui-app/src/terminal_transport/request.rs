use std::{
    ffi::{OsStr, OsString},
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
    environment: Vec<(OsString, OsString)>,
}

impl TerminalTransportRequest {
    pub(crate) fn new(executable: impl AsRef<OsStr>, cwd: PathBuf) -> Self {
        Self {
            executable: executable.as_ref().to_owned(),
            args: Vec::new(),
            cwd,
            environment: Vec::new(),
        }
    }

    pub(crate) fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.environment
            .push((key.as_ref().to_owned(), value.as_ref().to_owned()));
        self
    }

    pub(super) fn spawn_failure_context(&self) -> String {
        format!(
            "spawn PTY terminal shell {} in {}",
            self.executable.to_string_lossy(),
            self.cwd.display()
        )
    }

    pub(crate) fn into_command(self) -> Command {
        let mut command = Command::new(self.executable);
        command
            .args(self.args)
            .current_dir(self.cwd)
            .envs(self.environment);
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_materializes_only_process_launch_data() {
        let request = TerminalTransportRequest::new("/bin/sh", PathBuf::from("/tmp"))
            .env("TERM", "xterm-256color");
        let command = request.into_command();
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
    }
}
