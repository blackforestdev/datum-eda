use crate::{
    Cell, CellContent, CellWidth, Columns, CursorState, LimitError, ModeState, ScreenBuffer,
    SnapshotCellsLimit, TerminalSize,
};
use std::error::Error;
use std::fmt;

pub type SnapshotCell = Cell;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    WrongColumnCount { expected: u16, actual: usize },
    WrongRowCount { expected: u16, actual: usize },
    CellLimit(LimitError),
    ContinuationWithoutWideLead { row: usize, column: usize },
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongColumnCount { expected, actual } => {
                write!(
                    formatter,
                    "snapshot row has {actual} cells; expected {expected}"
                )
            }
            Self::WrongRowCount { expected, actual } => {
                write!(formatter, "snapshot has {actual} rows; expected {expected}")
            }
            Self::CellLimit(error) => error.fmt(formatter),
            Self::ContinuationWithoutWideLead { row, column } => write!(
                formatter,
                "snapshot continuation at row {row}, column {column} has no wide lead cell"
            ),
        }
    }
}

impl Error for SnapshotError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotRow {
    cells: Vec<SnapshotCell>,
    soft_wrapped: bool,
}

impl SnapshotRow {
    pub fn new(
        cells: Vec<SnapshotCell>,
        columns: Columns,
        soft_wrapped: bool,
    ) -> Result<Self, SnapshotError> {
        if cells.len() != usize::from(columns.get()) {
            return Err(SnapshotError::WrongColumnCount {
                expected: columns.get(),
                actual: cells.len(),
            });
        }
        Ok(Self {
            cells,
            soft_wrapped,
        })
    }

    pub fn cells(&self) -> &[SnapshotCell] {
        &self.cells
    }

    pub const fn soft_wrapped(&self) -> bool {
        self.soft_wrapped
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalSnapshot {
    size: TerminalSize,
    rows: Vec<SnapshotRow>,
    cursor: CursorState,
    modes: ModeState,
    active_buffer: ScreenBuffer,
}

impl TerminalSnapshot {
    pub fn new(
        size: TerminalSize,
        rows: Vec<SnapshotRow>,
        cursor: CursorState,
        modes: ModeState,
        active_buffer: ScreenBuffer,
        cell_limit: SnapshotCellsLimit,
    ) -> Result<Self, SnapshotError> {
        if rows.len() != usize::from(size.rows.get()) {
            return Err(SnapshotError::WrongRowCount {
                expected: size.rows.get(),
                actual: rows.len(),
            });
        }
        let cells =
            size.cell_count()
                .ok_or(SnapshotError::CellLimit(LimitError::ArithmeticOverflow {
                    kind: crate::LimitKind::SnapshotCells,
                }))?;
        cell_limit.check(cells).map_err(SnapshotError::CellLimit)?;
        validate_continuations(&rows)?;
        Ok(Self {
            size,
            rows,
            cursor,
            modes,
            active_buffer,
        })
    }

    pub const fn size(&self) -> TerminalSize {
        self.size
    }

    pub fn rows(&self) -> impl ExactSizeIterator<Item = &SnapshotRow> {
        self.rows.iter()
    }

    pub const fn cursor(&self) -> CursorState {
        self.cursor
    }

    pub const fn modes(&self) -> ModeState {
        self.modes
    }

    pub const fn active_buffer(&self) -> ScreenBuffer {
        self.active_buffer
    }
}

fn validate_continuations(rows: &[SnapshotRow]) -> Result<(), SnapshotError> {
    for (row_index, row) in rows.iter().enumerate() {
        for (column_index, cell) in row.cells.iter().enumerate() {
            let CellContent::Continuation { lead } = cell.content else {
                continue;
            };
            let lead_index = usize::from(lead.get());
            let valid = lead_index.checked_add(1) == Some(column_index)
                && matches!(
                    row.cells.get(lead_index).map(|cell| &cell.content),
                    Some(CellContent::Cluster(cluster)) if cluster.width() == CellWidth::Two
                );
            if !valid {
                return Err(SnapshotError::ContinuationWithoutWideLead {
                    row: row_index,
                    column: column_index,
                });
            }
        }
    }
    Ok(())
}
