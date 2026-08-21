//! Workspace-global terminal fallback-theme controls.

use super::*;

impl Runtime {
    pub(super) fn cycle_terminal_theme(&mut self) -> bool {
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.theme = terminal.theme.next();
        self.invalidate_scene();
        true
    }
}
