use std::{fmt, io};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TerminalLaunchStage {
    WorkingDirectory,
    AllocateMaster,
    NormalizeMaster,
    GrantSlave,
    UnlockSlave,
    ResolveSlavePath,
    OpenSlave,
    NormalizeSlave,
    CloneReader,
    CloneControl,
    ConfigureTermios,
    InitialSize,
    Spawn,
}

#[derive(Debug)]
pub(crate) struct TerminalLaunchError {
    pub(crate) stage: TerminalLaunchStage,
    pub(crate) kind: io::ErrorKind,
    pub(crate) os_code: Option<i32>,
    source: io::Error,
}

impl TerminalLaunchError {
    pub(crate) fn new(stage: TerminalLaunchStage, source: io::Error) -> Self {
        Self {
            stage,
            kind: source.kind(),
            os_code: source.raw_os_error(),
            source,
        }
    }
}

impl fmt::Display for TerminalLaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "terminal launch {:?} failed ({:?}, errno {:?}): {}",
            self.stage, self.kind, self.os_code, self.source,
        )
    }
}

impl std::error::Error for TerminalLaunchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
