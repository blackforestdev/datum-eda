use crate::{ClusterBytesLimit, Color, Column};
use std::error::Error;
use std::fmt;
use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnderlineStyle {
    None,
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellAttribute {
    Bold,
    Faint,
    Italic,
    Blink,
    Inverse,
    Hidden,
    Strike,
    Overline,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CellAttributes(u16);

impl CellAttributes {
    pub const fn contains(self, attribute: CellAttribute) -> bool {
        self.0 & attribute_bit(attribute) != 0
    }

    pub fn set(&mut self, attribute: CellAttribute, enabled: bool) {
        let bit = attribute_bit(attribute);
        if enabled {
            self.0 |= bit;
        } else {
            self.0 &= !bit;
        }
    }
}

const fn attribute_bit(attribute: CellAttribute) -> u16 {
    match attribute {
        CellAttribute::Bold => 1 << 0,
        CellAttribute::Faint => 1 << 1,
        CellAttribute::Italic => 1 << 2,
        CellAttribute::Blink => 1 << 3,
        CellAttribute::Inverse => 1 << 4,
        CellAttribute::Hidden => 1 << 5,
        CellAttribute::Strike => 1 << 6,
        CellAttribute::Overline => 1 << 7,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CellStyle {
    pub foreground: Color,
    pub background: Color,
    pub underline_color: Color,
    pub underline: UnderlineStyle,
    pub attributes: CellAttributes,
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            foreground: Color::Default,
            background: Color::Default,
            underline_color: Color::Default,
            underline: UnderlineStyle::None,
            attributes: CellAttributes::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellWidth {
    One,
    Two,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClusterError {
    Empty,
    TooLarge(crate::LimitError),
}

impl fmt::Display for ClusterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("terminal cluster must contain text"),
            Self::TooLarge(error) => error.fmt(formatter),
        }
    }
}

impl Error for ClusterError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cluster {
    text: String,
    width: CellWidth,
}

impl Cluster {
    pub fn new(
        text: impl Into<String>,
        width: CellWidth,
        limit: ClusterBytesLimit,
    ) -> Result<Self, ClusterError> {
        let text = text.into();
        if text.is_empty() {
            return Err(ClusterError::Empty);
        }
        limit.check(text.len()).map_err(ClusterError::TooLarge)?;
        Ok(Self { text, width })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub const fn width(&self) -> CellWidth {
        self.width
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HyperlinkId(NonZeroU64);

impl HyperlinkId {
    pub const fn new(value: NonZeroU64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellContent {
    Empty,
    Cluster(Cluster),
    Continuation { lead: Column },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cell {
    pub content: CellContent,
    pub style: CellStyle,
    pub hyperlink: Option<HyperlinkId>,
    pub protected: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            content: CellContent::Empty,
            style: CellStyle::default(),
            hyperlink: None,
            protected: false,
        }
    }
}
