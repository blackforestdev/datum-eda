//! Actual workspace paint dependencies of Global Preferences projection.
use super::DialogInputOutcome;
use datum_gui_protocol::{TerminalTheme, WorkspaceUiState};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct GlobalPreferenceRenderState {
    high_contrast: bool,
    terminal_theme: TerminalTheme,
    visible_console: Option<u64>,
}

impl GlobalPreferenceRenderState {
    pub(crate) fn capture(ui: &WorkspaceUiState) -> Self {
        Self {
            high_contrast: ui.global_preferences.high_contrast_noncolor,
            terminal_theme: ui.terminal.theme,
            visible_console: ui.console.visible_latest().map(|record| record.sequence),
        }
    }

    pub(crate) fn finish(
        self,
        outcome: DialogInputOutcome,
        ui: &WorkspaceUiState,
    ) -> DialogInputOutcome {
        // Closure changes application focus. Preserve that broader lifecycle path.
        if outcome != DialogInputOutcome::Dependents || !ui.global_preferences.open {
            return outcome;
        }
        // Future-project Units and reduced motion do not change current workspace
        // paint. Console deadline scheduling independently reads the new duration;
        // only a changed visible message needs an immediate frame.
        if self == Self::capture(ui) {
            DialogInputOutcome::Dialog
        } else {
            DialogInputOutcome::Workspace
        }
    }
}
