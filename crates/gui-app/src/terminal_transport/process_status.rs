use std::os::unix::process::ExitStatusExt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TerminalExitStatus {
    Code(i32),
    Signal { signal: i32, core_dumped: bool },
}

impl TerminalExitStatus {
    pub(super) fn from_std(status: std::process::ExitStatus) -> Self {
        if let Some(code) = status.code() {
            Self::Code(code)
        } else {
            Self::Signal {
                signal: status.signal().unwrap_or(0),
                core_dumped: status.core_dumped(),
            }
        }
    }
}
