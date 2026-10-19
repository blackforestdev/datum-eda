//! Shared Location keyboard assistance; no publication or filesystem access.
use super::*;
impl Runtime {
    pub(super) fn handle_path_assistance_key(&mut self, key: &Key) -> bool {
        let Some(state) = self.folder_browser.state.as_ref() else {
            return false;
        };
        if !(state.suggestions_ready() || state.ready() && *key == Key::Named(NamedKey::Tab))
            || state.menu.is_some()
            || state.naming.is_some()
            || self.modifiers.control_key()
            || self.modifiers.alt_key()
            || self.modifiers.super_key()
        {
            return false;
        }
        let focus = self.folder_navigation_focus();
        if *key == Key::Named(NamedKey::Escape)
            && state.selected.is_some()
            && matches!(focus, Some(Focus::Path | Focus::Folders))
        {
            self.folder_browser.state.as_mut().unwrap().selected = None;
            self.refresh_folder_browser();
            return true;
        }
        if focus != Some(Focus::Path)
            || self.modifiers.shift_key()
            || state.path_editor.caret != state.path_input.len()
            || !state.path_editor.selection(&state.path_input).is_empty()
        {
            return false;
        }
        let value = match key {
            Key::Named(NamedKey::ArrowDown | NamedKey::ArrowUp) => {
                self.folder_browser.state.as_mut().unwrap().move_selection(
                    if *key == Key::Named(NamedKey::ArrowDown) {
                        1
                    } else {
                        -1
                    },
                );
                self.refresh_folder_browser();
                return true;
            }
            Key::Named(NamedKey::Tab) => state.tab_suggestion(),
            Key::Named(NamedKey::Enter) => state.selected.and_then(|i| state.suggestion_path(i)),
            _ => None,
        };
        let Some(value) = value else {
            return false;
        };
        self.accept_path_assistance(value);
        true
    }
    pub(super) fn accept_path_assistance(&mut self, value: String) {
        let state = self.folder_browser.state.as_mut().unwrap();
        state.path_input = value;
        state.path_editor =
            datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&state.path_input);
        let assisted = !state.path_input.ends_with('/');
        self.terminal_accessibility.announce_console(
            crate::console_accessibility::AccessibilityAnnouncement {
                text: format!("Location {}", state.path_input),
                priority: crate::console_accessibility::AnnouncementPriority::Medium,
            },
        );
        self.folder_browser.invalidate_listing();
        let path = entered_path(&self.folder_browser.state.as_ref().unwrap().path_input);
        match path {
            Ok(path) => self.read_folder_request(path, false, assisted),
            Err(error) => self.folder_browser.state.as_mut().unwrap().error = Some(error),
        }
        self.session.workspace_mut().ui.project_open.refusal = None;
        self.refresh_folder_browser();
    }
}

#[cfg(test)]
#[path = "folder_browser_path_assistance_tests.rs"]
mod tests;
