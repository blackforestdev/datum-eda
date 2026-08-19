use crate::grid::GridBuffer;
use crate::{
    Cell, CellStyle, CoreLimits, CursorState, Margins, ModeState, SavedCursorState, ScreenBuffer,
    ScreenError, SnapshotError, SnapshotRow, TabStops, TerminalSize, TerminalSnapshot,
};

#[derive(Clone, Debug)]
pub struct ScreenState {
    pub(crate) size: TerminalSize,
    pub(crate) active_buffer: ScreenBuffer,
    pub(crate) cursor: CursorState,
    pub(crate) margins: Margins,
    pub(crate) modes: ModeState,
    pub(crate) tabs: TabStops,
    pub(crate) style: CellStyle,
    pub(crate) protected: bool,
    pub(crate) saved: Option<SavedCursorState>,
    pub(crate) primary: GridBuffer,
    pub(crate) alternate: GridBuffer,
}

impl ScreenState {
    pub(crate) fn new(size: TerminalSize, limits: &CoreLimits) -> Result<Self, ScreenError> {
        let cells = size.cell_count().ok_or(ScreenError::CellCountOverflow)?;
        limits
            .screen_cells
            .checked_total(cells, cells)
            .map_err(ScreenError::Limit)?;
        let primary = GridBuffer::new(size, limits.screen_cells).map_err(ScreenError::from)?;
        let alternate = GridBuffer::new(size, limits.screen_cells).map_err(ScreenError::from)?;
        let modes = ModeState {
            auto_wrap: true,
            ..ModeState::default()
        };
        Ok(Self {
            size,
            active_buffer: ScreenBuffer::Primary,
            cursor: CursorState::home(size),
            margins: Margins::full(size),
            modes,
            tabs: TabStops::default(),
            style: CellStyle::default(),
            protected: false,
            saved: None,
            primary,
            alternate,
        })
    }

    pub const fn size(&self) -> TerminalSize {
        self.size
    }

    pub const fn active_buffer(&self) -> ScreenBuffer {
        self.active_buffer
    }

    pub const fn cursor(&self) -> CursorState {
        self.cursor
    }

    pub const fn margins(&self) -> Margins {
        self.margins
    }

    pub const fn modes(&self) -> ModeState {
        self.modes
    }

    pub const fn style(&self) -> CellStyle {
        self.style
    }

    pub const fn protected(&self) -> bool {
        self.protected
    }

    pub fn tab_stops(&self) -> &TabStops {
        &self.tabs
    }

    pub fn saved_cursor(&self) -> Option<&SavedCursorState> {
        self.saved.as_ref()
    }

    pub fn cell(&self, row: u16, column: u16) -> Option<&Cell> {
        self.active_grid()
            .rows
            .get(usize::from(row))?
            .cells
            .get(usize::from(column))
    }

    pub fn snapshot(&self, limits: &CoreLimits) -> Result<TerminalSnapshot, SnapshotError> {
        let rows = self
            .active_grid()
            .rows
            .iter()
            .map(|row| SnapshotRow::new(row.cells.clone(), self.size.columns, row.soft_wrapped))
            .collect::<Result<Vec<_>, _>>()?;
        TerminalSnapshot::new(
            self.size,
            rows,
            self.cursor,
            self.modes,
            self.active_buffer,
            limits.snapshot_cells,
        )
    }

    pub(crate) fn active_grid(&self) -> &GridBuffer {
        match self.active_buffer {
            ScreenBuffer::Primary => &self.primary,
            ScreenBuffer::Alternate => &self.alternate,
        }
    }

    pub(crate) fn active_grid_mut(&mut self) -> &mut GridBuffer {
        match self.active_buffer {
            ScreenBuffer::Primary => &mut self.primary,
            ScreenBuffer::Alternate => &mut self.alternate,
        }
    }
}

/// The sole terminal semantic authority. DTC-P07 holds only closed state;
/// DTC-P08 and later packages add parsing and mutation behind this boundary.
#[derive(Clone, Debug)]
pub struct TerminalCore {
    pub(crate) limits: CoreLimits,
    pub(crate) state: ScreenState,
}

impl TerminalCore {
    pub fn new(limits: CoreLimits, size: TerminalSize) -> Result<Self, ScreenError> {
        let state = ScreenState::new(size, &limits)?;
        Ok(Self { limits, state })
    }

    pub const fn limits(&self) -> &CoreLimits {
        &self.limits
    }

    pub const fn state(&self) -> &ScreenState {
        &self.state
    }

    pub const fn size(&self) -> TerminalSize {
        self.state.size()
    }

    pub fn snapshot(&self) -> Result<TerminalSnapshot, SnapshotError> {
        self.state.snapshot(&self.limits)
    }
}
