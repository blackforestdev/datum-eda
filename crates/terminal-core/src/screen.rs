use crate::graphics::{GraphicLimits, GraphicStore};
use crate::grid::GridBuffer;
use crate::history::HistoryStore;
use crate::hyperlink::HyperlinkRegistry;
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
    pub(crate) current_hyperlink: Option<crate::HyperlinkId>,
    pub(crate) hyperlinks: HyperlinkRegistry,
    pub(crate) shell_mark: Option<crate::ShellMark>,
    pub(crate) progress: crate::ProgressState,
    pub(crate) synchronized_dirty: bool,
    pub(crate) last_printed: Option<Cluster>,
    pub(crate) grapheme_anchor: Option<crate::CellPoint>,
    pub(crate) saved: Option<SavedCursorState>,
    pub(crate) primary: GridBuffer,
    pub(crate) alternate: GridBuffer,
    pub(crate) history: HistoryStore,
    pub(crate) graphics: GraphicStore,
    pub(crate) sixel_colors: crate::SixelColorRegisters,
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
            sixel_scrolling: true,
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
            current_hyperlink: None,
            hyperlinks: HyperlinkRegistry::new(limits.hyperlink_bytes),
            shell_mark: None,
            progress: crate::ProgressState::Clear,
            synchronized_dirty: false,
            last_printed: None,
            grapheme_anchor: None,
            saved: None,
            primary,
            alternate,
            history: HistoryStore::new(limits.history_lines, limits.history_bytes),
            graphics: GraphicStore::new(GraphicLimits {
                objects: limits.graphic_objects,
                pixels: limits.graphic_pixels,
                decoded_bytes: limits.graphic_decoded_bytes,
                frames: limits.graphic_frames,
            }),
            sixel_colors: crate::SixelColorRegisters::default(),
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

    pub fn hyperlink(&self, id: crate::HyperlinkId) -> Option<&crate::Hyperlink> {
        self.hyperlinks.get(id)
    }

    pub const fn shell_mark(&self) -> Option<crate::ShellMark> {
        self.shell_mark
    }

    pub const fn progress(&self) -> crate::ProgressState {
        self.progress
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

    pub fn graphics(&self) -> impl ExactSizeIterator<Item = &crate::GraphicPlacement> {
        self.graphics.iter()
    }

    pub fn kitty_images(&self) -> impl ExactSizeIterator<Item = &crate::KittyImage> {
        self.graphics.kitty.images()
    }

    pub fn resolve_graphic(&self, id: crate::GraphicId) -> crate::GraphicAnchorResolution {
        let Some(graphic) = self.graphics.get(id) else {
            return crate::GraphicAnchorResolution::Unknown;
        };
        if graphic.buffer() != self.active_buffer {
            return crate::GraphicAnchorResolution::InactiveBuffer;
        }
        match self.resolve_logical_point(graphic.anchor()) {
            crate::AnchorResolution::History { row, column } => {
                crate::GraphicAnchorResolution::History { row, column }
            }
            crate::AnchorResolution::Screen { row, column } => {
                let (row, column) = if let Some(parent) = graphic.parent() {
                    let row = i64::from(row) + i64::from(parent.vertical_cells);
                    let column = i64::from(column) + i64::from(parent.horizontal_cells);
                    if row < 0
                        || column < 0
                        || row >= i64::from(self.size.rows.get())
                        || column >= i64::from(self.size.columns.get())
                    {
                        return crate::GraphicAnchorResolution::Trimmed;
                    }
                    (row as u16, column as u16)
                } else {
                    (row, column)
                };
                let cell_width = pixel_extent(
                    self.size.pixels.width,
                    u32::from(self.size.columns.get()),
                    1,
                );
                let cell_height =
                    pixel_extent(self.size.pixels.height, u32::from(self.size.rows.get()), 6);
                let x = u32::from(column).saturating_mul(cell_width);
                let y = u32::from(row).saturating_mul(cell_height);
                let surface_width = if self.size.pixels.width == 0 {
                    cell_width.saturating_mul(u32::from(self.size.columns.get()))
                } else {
                    self.size.pixels.width
                };
                let surface_height = if self.size.pixels.height == 0 {
                    cell_height.saturating_mul(u32::from(self.size.rows.get()))
                } else {
                    self.size.pixels.height
                };
                let source = graphic.source();
                let source_width = if source.width == 0 {
                    graphic.width().saturating_sub(source.x)
                } else {
                    source.width.min(graphic.width().saturating_sub(source.x))
                };
                let source_height = if source.height == 0 {
                    graphic.height().saturating_sub(source.y)
                } else {
                    source.height.min(graphic.height().saturating_sub(source.y))
                };
                let cells = graphic.cell_extent();
                let display_width = if cells.columns == 0 {
                    source_width
                } else {
                    cells.columns.saturating_mul(cell_width)
                };
                let display_height = if cells.rows == 0 {
                    source_height
                } else {
                    cells.rows.saturating_mul(cell_height)
                };
                crate::GraphicAnchorResolution::Screen {
                    row,
                    column,
                    visible_pixel_width: display_width.min(surface_width.saturating_sub(x)),
                    visible_pixel_height: display_height.min(surface_height.saturating_sub(y)),
                }
            }
            crate::AnchorResolution::Trimmed => crate::GraphicAnchorResolution::Trimmed,
            crate::AnchorResolution::Unknown => crate::GraphicAnchorResolution::Unknown,
        }
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

fn pixel_extent(total: u32, cells: u32, fallback: u32) -> u32 {
    if total == 0 {
        fallback
    } else {
        (total / cells).max(1)
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
