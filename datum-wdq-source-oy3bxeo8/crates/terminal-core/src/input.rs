use crate::{InputBytesLimit, LimitError, TerminalCore};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputBytes(Vec<u8>);

impl InputBytes {
    fn new(bytes: Vec<u8>, limit: InputBytesLimit) -> Result<Self, InputError> {
        limit.check(bytes.len())?;
        Ok(Self(bytes))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputDisposition {
    Send(InputBytes),
    LocalOnly,
    Ignored,
}

impl InputDisposition {
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Send(bytes) => Some(bytes.as_slice()),
            Self::LocalOnly | Self::Ignored => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputError {
    Limit(LimitError),
}

impl fmt::Display for InputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit(error) => error.fmt(formatter),
        }
    }
}

impl Error for InputError {}

impl From<LimitError> for InputError {
    fn from(value: LimitError) -> Self {
        Self::Limit(value)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub alt: bool,
    pub control: bool,
    pub super_key: bool,
    pub hyper: bool,
    pub meta: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyEventKind {
    Press,
    Repeat,
    Release,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeypadKey {
    Digit(u8),
    Decimal,
    Divide,
    Multiply,
    Subtract,
    Add,
    Enter,
    Equal,
    Separator,
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Insert,
    Delete,
    Begin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaKey {
    Play,
    Pause,
    PlayPause,
    Reverse,
    Stop,
    FastForward,
    Rewind,
    Next,
    Previous,
    Record,
    VolumeDown,
    VolumeUp,
    Mute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModifierKey {
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    LeftHyper,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    RightHyper,
    RightMeta,
    IsoLevel3Shift,
    IsoLevel5Shift,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyCode {
    Text(String),
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Function(u8),
    Keypad(KeypadKey),
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    Media(MediaKey),
    Modifier(ModifierKey),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyInput {
    pub code: KeyCode,
    pub shifted_key: Option<u32>,
    pub base_layout_key: Option<u32>,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImeInput {
    Preedit(String),
    Commit(String),
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusInput {
    Gained,
    Lost,
}

impl TerminalCore {
    pub fn encode_key(&self, input: &KeyInput) -> Result<InputDisposition, InputError> {
        let bytes = crate::input_key::encode_key(self, input);
        self.input_disposition(bytes)
    }

    pub fn encode_focus(&self, input: FocusInput) -> Result<InputDisposition, InputError> {
        if !self.state.modes.focus_reporting {
            return Ok(InputDisposition::Ignored);
        }
        let bytes = match input {
            FocusInput::Gained => b"\x1b[I".to_vec(),
            FocusInput::Lost => b"\x1b[O".to_vec(),
        };
        self.input_disposition(Some(bytes))
    }

    pub fn encode_paste(&self, text: &str) -> Result<InputDisposition, InputError> {
        let mut bytes = Vec::with_capacity(text.len().saturating_add(12));
        if self.state.modes.bracketed_paste {
            bytes.extend_from_slice(b"\x1b[200~");
        }
        bytes.extend_from_slice(text.as_bytes());
        if self.state.modes.bracketed_paste {
            bytes.extend_from_slice(b"\x1b[201~");
        }
        self.input_disposition(Some(bytes))
    }

    pub fn encode_ime(&self, input: &ImeInput) -> Result<InputDisposition, InputError> {
        match input {
            ImeInput::Preedit(_) | ImeInput::Disabled => Ok(InputDisposition::LocalOnly),
            ImeInput::Commit(text) if text.is_empty() => Ok(InputDisposition::Ignored),
            ImeInput::Commit(text) => self.input_disposition(Some(text.as_bytes().to_vec())),
        }
    }

    pub(crate) fn input_disposition(
        &self,
        bytes: Option<Vec<u8>>,
    ) -> Result<InputDisposition, InputError> {
        let Some(bytes) = bytes else {
            return Ok(InputDisposition::Ignored);
        };
        if bytes.is_empty() {
            return Ok(InputDisposition::Ignored);
        }
        Ok(InputDisposition::Send(InputBytes::new(
            bytes,
            self.limits.input_bytes,
        )?))
    }
}
