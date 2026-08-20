use crate::{
    ClipboardBytesLimit, Color, LimitError, NotificationBytesLimit, PaletteIndex, ReplyBytesLimit,
    TitleBytesLimit, WorkingDirectoryBytesLimit,
};

macro_rules! bounded_text {
    ($name:ident, $limit:ty) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>, limit: $limit) -> Result<Self, LimitError> {
                let value = value.into();
                limit.check(value.len())?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn len(&self) -> usize {
                self.0.len()
            }

            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }
    };
}

bounded_text!(TitleText, TitleBytesLimit);
bounded_text!(WorkingDirectoryText, WorkingDirectoryBytesLimit);
bounded_text!(NotificationText, NotificationBytesLimit);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardBytes(Vec<u8>);

impl ClipboardBytes {
    pub fn new(value: Vec<u8>, limit: ClipboardBytesLimit) -> Result<Self, LimitError> {
        limit.check(value.len())?;
        Ok(Self(value))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardSelection {
    Clipboard,
    Primary,
    Select,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Percent(u8);

impl Percent {
    pub fn new(value: u8) -> Option<Self> {
        (value <= 100).then_some(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgressState {
    Clear,
    Set { percent: Percent },
    Error { percent: Percent },
    Paused { percent: Percent },
    Indeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreEvent {
    Bell,
    TitleChanged(TitleText),
    WorkingDirectoryChanged(WorkingDirectoryText),
    ClipboardRequest {
        selection: ClipboardSelection,
        contents: ClipboardBytes,
    },
    Notification(NotificationText),
    Progress(ProgressState),
    PaletteChanged {
        index: PaletteIndex,
        color: Color,
    },
    LimitReached(crate::LimitKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplyKind {
    DeviceAttributes,
    DeviceStatus,
    CursorPosition,
    ModeReport,
    ColorReport,
    KeyboardProtocol,
    Graphics,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalReply {
    kind: ReplyKind,
    bytes: Vec<u8>,
}

impl TerminalReply {
    pub fn new(
        kind: ReplyKind,
        bytes: Vec<u8>,
        limit: ReplyBytesLimit,
    ) -> Result<Self, LimitError> {
        limit.check(bytes.len())?;
        Ok(Self { kind, bytes })
    }

    pub const fn kind(&self) -> ReplyKind {
        self.kind
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
