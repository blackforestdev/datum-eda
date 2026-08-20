use crate::grid::GridBuffer;
use crate::history::HistoryStore;
use crate::{
    Cell, CellStyle, CharacterSetState, Cluster, Color, CoreLimits, CursorState,
    KittyKeyboardState, Margins, ModeState, MouseEncoding, MouseTracking, SavedCursorState,
    ScreenBuffer, ScreenError, SnapshotError, SnapshotRow, TabStops, TerminalSize,
    TerminalSnapshot, TitleText, WorkingDirectoryText,
};

#[derive(Clone, Debug)]
pub struct ScreenState {
    pub(crate) size: TerminalSize,
    pub(crate) active_buffer: ScreenBuffer,
    pub(crate) cursor: CursorState,
    pub(crate) margins: Margins,
    pub(crate) modes: ModeState,
    pub(crate) mouse_tracking: MouseTracking,
    pub(crate) mouse_encoding: MouseEncoding,
    pub(crate) kitty_keyboard: KittyKeyboardState,
    pub(crate) tabs: TabStops,
    pub(crate) style: CellStyle,
    pub(crate) protected: bool,
    pub(crate) charsets: CharacterSetState,
    pub(crate) palette: [Color; 256],
    pub(crate) default_foreground: Color,
    pub(crate) default_background: Color,
    pub(crate) cursor_color: Color,
    pub(crate) title: Option<TitleText>,
    pub(crate) working_directory: Option<WorkingDirectoryText>,
    pub(crate) synchronized_dirty: bool,
    pub(crate) last_printed: Option<Cluster>,
    pub(crate) grapheme_anchor: Option<crate::CellPoint>,
    pub(crate) saved: Option<SavedCursorState>,
    pub(crate) primary: GridBuffer,
    pub(crate) alternate: GridBuffer,
    pub(crate) history: HistoryStore,
    pub(crate) selection: Option<crate::Selection>,
    pub(crate) next_logical_line: u64,
}

impl ScreenState {
    pub(crate) fn new(size: TerminalSize, limits: &CoreLimits) -> Result<Self, ScreenError> {
        let cells = size.cell_count().ok_or(ScreenError::CellCountOverflow)?;
        limits
            .screen_cells
            .checked_total(cells, cells)
            .map_err(ScreenError::Limit)?;
        let mut next_logical_line = 0;
        let primary = GridBuffer::new(size, limits.screen_cells, &mut next_logical_line)
            .map_err(ScreenError::from)?;
        let alternate = GridBuffer::new(size, limits.screen_cells, &mut next_logical_line)
            .map_err(ScreenError::from)?;
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
            mouse_tracking: MouseTracking::Off,
            mouse_encoding: MouseEncoding::Default,
            kitty_keyboard: KittyKeyboardState::default(),
            tabs: TabStops::every_eight(size.columns),
            style: CellStyle::default(),
            protected: false,
            charsets: CharacterSetState::default(),
            palette: [Color::Default; 256],
            default_foreground: Color::Default,
            default_background: Color::Default,
            cursor_color: Color::Default,
            title: None,
            working_directory: None,
            synchronized_dirty: false,
            last_printed: None,
            grapheme_anchor: None,
            saved: None,
            primary,
            alternate,
            history: HistoryStore::new(limits.history_lines, limits.history_bytes),
            selection: None,
            next_logical_line,
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

    pub const fn mouse_tracking(&self) -> MouseTracking {
        self.mouse_tracking
    }

    pub const fn mouse_encoding(&self) -> MouseEncoding {
        self.mouse_encoding
    }

    pub const fn kitty_keyboard(&self) -> &KittyKeyboardState {
        &self.kitty_keyboard
    }

    pub const fn style(&self) -> CellStyle {
        self.style
    }

    pub const fn protected(&self) -> bool {
        self.protected
    }

    pub const fn character_sets(&self) -> CharacterSetState {
        self.charsets
    }

    pub const fn palette_color(&self, index: u8) -> Color {
        self.palette[index as usize]
    }

    pub const fn default_foreground(&self) -> Color {
        self.default_foreground
    }

    pub const fn default_background(&self) -> Color {
        self.default_background
    }

    pub const fn cursor_color(&self) -> Color {
        self.cursor_color
    }

    pub fn title(&self) -> Option<&TitleText> {
        self.title.as_ref()
    }

    pub fn working_directory(&self) -> Option<&WorkingDirectoryText> {
        self.working_directory.as_ref()
    }

    pub fn tab_stops(&self) -> &TabStops {
        &self.tabs
    }

    pub fn saved_cursor(&self) -> Option<&SavedCursorState> {
        self.saved.as_ref()
    }

    pub fn history(&self) -> crate::HistorySnapshot {
        self.history.snapshot()
    }

    pub fn contains_logical_point(&self, point: crate::LogicalPoint) -> bool {
        match self.active_buffer {
            ScreenBuffer::Primary => {
                self.history.contains(point)
                    || self
                        .primary
                        .rows
                        .iter()
                        .any(|row| crate::history::row_contains(row, point))
            }
            ScreenBuffer::Alternate => self
                .alternate
                .rows
                .iter()
                .any(|row| crate::history::row_contains(row, point)),
        }
    }

    pub fn logical_point_at(&self, row: u16, column: u16) -> Option<crate::LogicalPoint> {
        let row = self.active_grid().rows.get(usize::from(row))?;
        let mut cluster = row.cluster_start;
        for cell in row.cells.iter().take(usize::from(column)) {
            if !matches!(cell.content, crate::CellContent::Continuation { .. }) {
                cluster = cluster.saturating_add(1);
            }
        }
        Some(crate::LogicalPoint {
            line: row.logical_line,
            cluster,
        })
    }

    pub fn resolve_logical_point(&self, point: crate::LogicalPoint) -> crate::AnchorResolution {
        if self.active_buffer == ScreenBuffer::Primary
            && let Some((row, column)) = self.history.resolve(point)
        {
            return crate::AnchorResolution::History { row, column };
        }
        for (row, grid_row) in self.active_grid().rows.iter().enumerate() {
            if let Some(column) = crate::history::column_for_point(grid_row, point) {
                return crate::AnchorResolution::Screen {
                    row: row.min(u16::MAX as usize) as u16,
                    column,
                };
            }
        }
        let oldest = (self.active_buffer == ScreenBuffer::Primary)
            .then(|| {
                self.history
                    .oldest_line()
                    .or_else(|| self.primary.rows.first().map(|row| row.logical_line))
            })
            .flatten();
        if oldest.is_some_and(|oldest| point.line < oldest) {
            crate::AnchorResolution::Trimmed
        } else {
            crate::AnchorResolution::Unknown
        }
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
