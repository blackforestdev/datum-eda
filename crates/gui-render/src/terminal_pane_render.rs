//! Immutable renderer input for one visible terminal split leaf.

use datum_gui_protocol::TerminalLaneState;

/// One session's complete renderer-facing state for the current frame.
///
/// The snapshot and damage are consumed from that session's owned
/// TerminalCore. `lane` carries only the matching session projection; app-wide
/// chrome remains in the surrounding `ReviewWorkspaceState`.
pub struct TerminalPaneRenderState {
    pub session_id: String,
    pub focused: bool,
    pub lane: TerminalLaneState,
    pub snapshot: datum_terminal_core::RenderSnapshot,
    pub damage: Vec<datum_terminal_core::Damage>,
}
