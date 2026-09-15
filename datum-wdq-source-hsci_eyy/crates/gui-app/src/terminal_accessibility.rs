//! Immutable accessibility projection for the active native terminal.
//!
//! This is the application-owned semantic tree consumed by the Linux AT-SPI
//! provider. It contains no PTY handles and cannot mutate TerminalCore.

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TerminalAccessibilityLink {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) uri: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TerminalAccessibilitySnapshot {
    pub(crate) session_id: String,
    pub(crate) title: String,
    pub(crate) text: String,
    pub(crate) caret: usize,
    pub(crate) selection: Option<(usize, usize)>,
    pub(crate) links: Vec<TerminalAccessibilityLink>,
    pub(crate) focused: bool,
    pub(crate) bell_count: usize,
    pub(crate) bounds: TerminalAccessibilityBounds,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TerminalAccessibilityBounds {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
}
