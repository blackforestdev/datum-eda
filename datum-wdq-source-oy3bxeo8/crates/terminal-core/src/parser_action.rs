use crate::LimitKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParserStateKind {
    Ground,
    Escape,
    Csi,
    Osc,
    Dcs,
    Apc,
    Pm,
    Sos,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControlCode(u8);

impl ControlCode {
    pub fn new(byte: u8) -> Option<Self> {
        ((byte <= 0x1f) || (0x7f..=0x9f).contains(&byte)).then_some(Self(byte))
    }

    pub const fn byte(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlStringKind {
    Osc,
    Dcs,
    Apc,
    Pm,
    Sos,
}

impl ControlStringKind {
    pub const fn state(self) -> ParserStateKind {
        match self {
            Self::Osc => ParserStateKind::Osc,
            Self::Dcs => ParserStateKind::Dcs,
            Self::Apc => ParserStateKind::Apc,
            Self::Pm => ParserStateKind::Pm,
            Self::Sos => ParserStateKind::Sos,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StringTerminator {
    Bell,
    StringTerminator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscapeSequence {
    pub intermediates: Vec<u8>,
    pub final_byte: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CsiParameter {
    pub subparameters: Vec<Option<usize>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CsiSequence {
    pub private_markers: Vec<u8>,
    pub parameters: Vec<CsiParameter>,
    pub intermediates: Vec<u8>,
    pub final_byte: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlString {
    pub kind: ControlStringKind,
    pub bytes: Vec<u8>,
    pub terminator: StringTerminator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseError {
    MalformedUtf8,
    IncompleteSequence { state: ParserStateKind },
    UnexpectedByte { state: ParserStateKind, byte: u8 },
    LimitExceeded(LimitKind),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Print(char),
    Execute(ControlCode),
    Escape(EscapeSequence),
    Csi(CsiSequence),
    ControlString(ControlString),
    Cancelled {
        state: ParserStateKind,
        by: ControlCode,
    },
    Error(ParseError),
}
