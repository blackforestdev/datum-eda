use crate::{
    CellStyle, CoreLimits, CursorState, Margins, ModeState, SavedCursorState, ScreenBuffer,
    TabStops, TerminalSize,
};

#[derive(Clone, Debug)]
pub struct ScreenState {
    size: TerminalSize,
    active_buffer: ScreenBuffer,
    cursor: CursorState,
    margins: Margins,
    modes: ModeState,
    tabs: TabStops,
    style: CellStyle,
    saved: Option<SavedCursorState>,
}

impl ScreenState {
    pub fn new(size: TerminalSize) -> Self {
        Self {
            size,
            active_buffer: ScreenBuffer::Primary,
            cursor: CursorState::home(size),
            margins: Margins::full(size),
            modes: ModeState::default(),
            tabs: TabStops::default(),
            style: CellStyle::default(),
            saved: None,
        }
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

    pub fn tab_stops(&self) -> &TabStops {
        &self.tabs
    }

    pub fn saved_cursor(&self) -> Option<&SavedCursorState> {
        self.saved.as_ref()
    }
}

/// The sole terminal semantic authority. DTC-P07 holds only closed state;
/// DTC-P08 and later packages add parsing and mutation behind this boundary.
#[derive(Clone, Debug)]
pub struct TerminalCore {
    limits: CoreLimits,
    state: ScreenState,
}

impl TerminalCore {
    pub fn new(limits: CoreLimits, size: TerminalSize) -> Self {
        Self {
            limits,
            state: ScreenState::new(size),
        }
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
}
