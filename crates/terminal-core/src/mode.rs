use crate::{CellPoint, CellStyle, CharacterSetState, Column, CoordinateError, Row, TerminalSize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CursorShape {
    Block,
    Underline,
    Bar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CursorState {
    pub position: CellPoint,
    pub visible: bool,
    pub blinking: bool,
    pub shape: CursorShape,
    pub pending_wrap: bool,
}

impl CursorState {
    pub fn home(size: TerminalSize) -> Self {
        Self {
            position: CellPoint::new(0, 0, size).expect("nonzero terminal size has an origin"),
            visible: true,
            blinking: true,
            shape: CursorShape::Block,
            pending_wrap: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Margins {
    pub top: Row,
    pub bottom: Row,
    pub left: Column,
    pub right: Column,
}

impl Margins {
    pub fn full(size: TerminalSize) -> Self {
        Self {
            top: Row::new(0, size.rows).expect("nonzero terminal size has a top row"),
            bottom: Row::new(size.rows.get() - 1, size.rows).expect("last row belongs to terminal"),
            left: Column::new(0, size.columns).expect("nonzero terminal size has a left column"),
            right: Column::new(size.columns.get() - 1, size.columns)
                .expect("last column belongs to terminal"),
        }
    }

    pub fn new(
        top: u16,
        bottom: u16,
        left: u16,
        right: u16,
        size: TerminalSize,
    ) -> Result<Self, CoordinateError> {
        let margins = Self {
            top: Row::new(top, size.rows)?,
            bottom: Row::new(bottom, size.rows)?,
            left: Column::new(left, size.columns)?,
            right: Column::new(right, size.columns)?,
        };
        if margins.top <= margins.bottom && margins.left <= margins.right {
            Ok(margins)
        } else {
            Err(CoordinateError::InvertedMargins)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ModeState {
    pub application_cursor: bool,
    pub application_keypad: bool,
    pub auto_wrap: bool,
    pub origin: bool,
    pub insert: bool,
    pub newline: bool,
    pub bracketed_paste: bool,
    pub focus_reporting: bool,
    pub synchronized_output: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScreenBuffer {
    Primary,
    Alternate,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MouseTracking {
    #[default]
    Off,
    X10,
    Button,
    Drag,
    Any,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MouseEncoding {
    #[default]
    Default,
    Utf8,
    Sgr,
    Urxvt,
    SgrPixels,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KittyKeyboardState {
    pub(crate) flags: u8,
    pub(crate) stack: Vec<u8>,
}

impl KittyKeyboardState {
    pub const fn flags(&self) -> u8 {
        self.flags
    }

    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TabStops {
    columns: Vec<Column>,
}

impl TabStops {
    pub fn every_eight(columns: crate::Columns) -> Self {
        Self::from_columns(
            (8..columns.get())
                .step_by(8)
                .filter_map(|column| Column::new(column, columns).ok())
                .collect(),
        )
    }

    pub fn from_columns(mut columns: Vec<Column>) -> Self {
        columns.sort_unstable();
        columns.dedup();
        Self { columns }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = Column> + DoubleEndedIterator + '_ {
        self.columns.iter().copied()
    }

    pub fn set(&mut self, column: Column) {
        match self.columns.binary_search(&column) {
            Ok(_) => {}
            Err(index) => self.columns.insert(index, column),
        }
    }

    pub fn clear(&mut self, column: Column) {
        if let Ok(index) = self.columns.binary_search(&column) {
            self.columns.remove(index);
        }
    }

    pub fn clear_all(&mut self) {
        self.columns.clear();
    }

    pub(crate) fn resize(&mut self, columns: crate::Columns) {
        self.columns.retain(|column| column.get() < columns.get());
        for column in (8..columns.get()).step_by(8) {
            if let Ok(column) = Column::new(column, columns) {
                self.set(column);
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedCursorState {
    pub cursor: CursorState,
    pub style: CellStyle,
    pub modes: ModeState,
    pub charsets: CharacterSetState,
    pub protected: bool,
}
