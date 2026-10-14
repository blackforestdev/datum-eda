use std::{fmt, io};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TerminalIoStage {
    Read,
    Write,
    Wait,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TerminalIoError {
    pub(crate) stage: TerminalIoStage,
    pub(crate) kind: io::ErrorKind,
    pub(crate) os_code: Option<i32>,
    pub(crate) accepted_bytes: usize,
    pub(crate) written_bytes: usize,
    pub(crate) remaining_bytes: usize,
    pub(crate) undelivered_requests: usize,
    pub(crate) total_undelivered_bytes: usize,
}

impl TerminalIoError {
    pub(crate) fn read(error: &io::Error) -> Self {
        Self {
            stage: TerminalIoStage::Read,
            kind: error.kind(),
            os_code: error.raw_os_error(),
            accepted_bytes: 0,
            written_bytes: 0,
            remaining_bytes: 0,
            undelivered_requests: 0,
            total_undelivered_bytes: 0,
        }
    }

    pub(crate) fn write(
        error: &io::Error,
        accepted: usize,
        written: usize,
        undelivered_requests: usize,
        total_undelivered_bytes: usize,
    ) -> Self {
        Self {
            stage: TerminalIoStage::Write,
            kind: error.kind(),
            os_code: error.raw_os_error(),
            accepted_bytes: accepted,
            written_bytes: written,
            remaining_bytes: accepted.saturating_sub(written),
            undelivered_requests,
            total_undelivered_bytes,
        }
    }

    pub(crate) fn wait(error: &io::Error) -> Self {
        Self {
            stage: TerminalIoStage::Wait,
            kind: error.kind(),
            os_code: error.raw_os_error(),
            accepted_bytes: 0,
            written_bytes: 0,
            remaining_bytes: 0,
            undelivered_requests: 0,
            total_undelivered_bytes: 0,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum TerminalTransportEvent {
    Output(Vec<u8>),
    Error(TerminalIoError),
    Exited(super::process_status::TerminalExitStatus),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TerminalInputError {
    Busy,
    RequestLimit,
    ByteLimit,
    RequestTooLarge,
    Closed,
}

impl fmt::Display for TerminalInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Busy => "terminal input queue is busy; retry",
            Self::RequestLimit => "terminal input request limit reached; retry",
            Self::ByteLimit => "terminal input byte limit reached; retry",
            Self::RequestTooLarge => "terminal input request exceeds the 4 MiB limit",
            Self::Closed => "terminal input transport is closed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TerminalInputError {}
