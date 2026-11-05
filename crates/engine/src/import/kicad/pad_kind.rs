//! Imported pad kind and its source spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KiCadPadKind {
    Smd,
    ThruHole,
    NpThruHole,
    Connect,
}

impl std::fmt::Display for KiCadPadKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KiCadPadKind::Smd => write!(f, "smd"),
            KiCadPadKind::ThruHole => write!(f, "thru_hole"),
            KiCadPadKind::NpThruHole => write!(f, "np_thru_hole"),
            KiCadPadKind::Connect => write!(f, "connect"),
        }
    }
}
