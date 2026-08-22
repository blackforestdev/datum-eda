use std::error::Error;
use std::fmt;
use std::num::NonZeroU16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoordinateError {
    ZeroColumns,
    ZeroRows,
    ColumnOutOfBounds { column: u16, columns: u16 },
    RowOutOfBounds { row: u16, rows: u16 },
    InvertedMargins,
}

impl fmt::Display for CoordinateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroColumns => formatter.write_str("terminal columns must be nonzero"),
            Self::ZeroRows => formatter.write_str("terminal rows must be nonzero"),
            Self::ColumnOutOfBounds { column, columns } => {
                write!(formatter, "column {column} is outside {columns} columns")
            }
            Self::RowOutOfBounds { row, rows } => {
                write!(formatter, "row {row} is outside {rows} rows")
            }
            Self::InvertedMargins => formatter.write_str("terminal margins are inverted"),
        }
    }
}

impl Error for CoordinateError {}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Columns(NonZeroU16);

impl Columns {
    pub fn new(value: u16) -> Result<Self, CoordinateError> {
        NonZeroU16::new(value)
            .map(Self)
            .ok_or(CoordinateError::ZeroColumns)
    }

    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Rows(NonZeroU16);

impl Rows {
    pub fn new(value: u16) -> Result<Self, CoordinateError> {
        NonZeroU16::new(value)
            .map(Self)
            .ok_or(CoordinateError::ZeroRows)
    }

    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PixelSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalSize {
    pub columns: Columns,
    pub rows: Rows,
    /// Zero means the cell surface's pixel extent is not known.
    pub pixels: PixelSize,
}

impl TerminalSize {
    pub fn new(
        columns: u16,
        rows: u16,
        pixel_width: u32,
        pixel_height: u32,
    ) -> Result<Self, CoordinateError> {
        Ok(Self {
            columns: Columns::new(columns)?,
            rows: Rows::new(rows)?,
            pixels: PixelSize {
                width: pixel_width,
                height: pixel_height,
            },
        })
    }

    pub fn cell_count(self) -> Option<usize> {
        usize::from(self.columns.get()).checked_mul(usize::from(self.rows.get()))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Column(u16);

impl Column {
    pub fn new(value: u16, columns: Columns) -> Result<Self, CoordinateError> {
        if value < columns.get() {
            Ok(Self(value))
        } else {
            Err(CoordinateError::ColumnOutOfBounds {
                column: value,
                columns: columns.get(),
            })
        }
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Row(u16);

impl Row {
    pub fn new(value: u16, rows: Rows) -> Result<Self, CoordinateError> {
        if value < rows.get() {
            Ok(Self(value))
        } else {
            Err(CoordinateError::RowOutOfBounds {
                row: value,
                rows: rows.get(),
            })
        }
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CellPoint {
    pub row: Row,
    pub column: Column,
}

impl CellPoint {
    pub fn new(row: u16, column: u16, size: TerminalSize) -> Result<Self, CoordinateError> {
        Ok(Self {
            row: Row::new(row, size.rows)?,
            column: Column::new(column, size.columns)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LogicalLineId(u64);

impl LogicalLineId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LogicalPoint {
    pub line: LogicalLineId,
    pub cluster: u32,
}
