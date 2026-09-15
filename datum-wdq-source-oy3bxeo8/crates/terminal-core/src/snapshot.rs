use crate::{
    Cell, CellContent, CellWidth, Color, Columns, CursorState, GraphicAnchorResolution,
    GraphicPlacement, LimitError, LogicalPoint, ModeState, ScreenBuffer, Selection,
    SnapshotCellsLimit, TerminalSize,
};
use std::error::Error;
use std::fmt;

pub type SnapshotCell = Cell;

pub const RENDER_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderRowSource {
    History { index: usize },
    Screen { row: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderRow {
    source: RenderRowSource,
    logical_start: LogicalPoint,
    cells: Vec<SnapshotCell>,
    soft_wrapped: bool,
}

impl RenderRow {
    pub(crate) fn new(
        source: RenderRowSource,
        logical_start: LogicalPoint,
        cells: Vec<SnapshotCell>,
        soft_wrapped: bool,
    ) -> Self {
        Self {
            source,
            logical_start,
            cells,
            soft_wrapped,
        }
    }

    pub const fn source(&self) -> RenderRowSource {
        self.source
    }

    pub const fn logical_start(&self) -> LogicalPoint {
        self.logical_start
    }

    pub fn cells(&self) -> &[SnapshotCell] {
        &self.cells
    }

    pub const fn soft_wrapped(&self) -> bool {
        self.soft_wrapped
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderPalette {
    entries: [Color; 256],
    default_foreground: Color,
    default_background: Color,
    cursor: Color,
}

impl RenderPalette {
    pub(crate) const fn new(
        entries: [Color; 256],
        default_foreground: Color,
        default_background: Color,
        cursor: Color,
    ) -> Self {
        Self {
            entries,
            default_foreground,
            default_background,
            cursor,
        }
    }

    pub const fn color(&self, index: u8) -> Color {
        self.entries[index as usize]
    }

    pub fn entries(&self) -> &[Color; 256] {
        &self.entries
    }

    pub const fn default_foreground(&self) -> Color {
        self.default_foreground
    }

    pub const fn default_background(&self) -> Color {
        self.default_background
    }

    pub const fn cursor(&self) -> Color {
        self.cursor
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderGraphic {
    placement: GraphicPlacement,
    resolution: GraphicAnchorResolution,
}

impl RenderGraphic {
    pub(crate) const fn new(
        placement: GraphicPlacement,
        resolution: GraphicAnchorResolution,
    ) -> Self {
        Self {
            placement,
            resolution,
        }
    }

    pub const fn placement(&self) -> &GraphicPlacement {
        &self.placement
    }

    pub const fn resolution(&self) -> GraphicAnchorResolution {
        self.resolution
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderSnapshot {
    schema_version: u16,
    size: TerminalSize,
    active_buffer: ScreenBuffer,
    rows: Vec<RenderRow>,
    history_rows: usize,
    history_trimmed_rows: u64,
    cursor: CursorState,
    modes: ModeState,
    palette: RenderPalette,
    selection: Option<Selection>,
    graphics: Vec<RenderGraphic>,
}

impl RenderSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        size: TerminalSize,
        active_buffer: ScreenBuffer,
        rows: Vec<RenderRow>,
        history_rows: usize,
        history_trimmed_rows: u64,
        cursor: CursorState,
        modes: ModeState,
        palette: RenderPalette,
        selection: Option<Selection>,
        mut graphics: Vec<RenderGraphic>,
        cell_limit: SnapshotCellsLimit,
    ) -> Result<Self, SnapshotError> {
        let cells = rows.iter().try_fold(0usize, |total, row| {
            total
                .checked_add(row.cells.len())
                .ok_or(SnapshotError::CellLimit(LimitError::ArithmeticOverflow {
                    kind: crate::LimitKind::SnapshotCells,
                }))
        })?;
        cell_limit.check(cells).map_err(SnapshotError::CellLimit)?;
        graphics.sort_by_key(|graphic| (graphic.placement.z_index(), graphic.placement.id().get()));
        Ok(Self {
            schema_version: RENDER_SNAPSHOT_SCHEMA_VERSION,
            size,
            active_buffer,
            rows,
            history_rows,
            history_trimmed_rows,
            cursor,
            modes,
            palette,
            selection,
            graphics,
        })
    }

    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub const fn size(&self) -> TerminalSize {
        self.size
    }

    pub const fn active_buffer(&self) -> ScreenBuffer {
        self.active_buffer
    }

    pub fn rows(&self) -> impl ExactSizeIterator<Item = &RenderRow> {
        self.rows.iter()
    }

    pub const fn history_row_count(&self) -> usize {
        self.history_rows
    }

    pub const fn history_trimmed_rows(&self) -> u64 {
        self.history_trimmed_rows
    }

    pub const fn cursor(&self) -> CursorState {
        self.cursor
    }

    pub const fn modes(&self) -> ModeState {
        self.modes
    }

    pub const fn palette(&self) -> &RenderPalette {
        &self.palette
    }

    pub const fn selection(&self) -> Option<Selection> {
        self.selection
    }

    pub fn graphics(&self) -> impl ExactSizeIterator<Item = &RenderGraphic> {
        self.graphics.iter()
    }
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
