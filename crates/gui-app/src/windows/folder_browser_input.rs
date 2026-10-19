//! Shared inline directory controls. Form actions remain with New/Open owners.
use super::*;

impl Runtime {
    fn navigation_enabled(&self) -> bool {
        self.folder_browser.state.is_some() && !self.workspace().ui.project_open.busy
    }
    pub(crate) fn activate_folder_browser_hit(&mut self, target: &HitTarget) -> bool {
        if !self.navigation_enabled() {
            return false;
        }
        if !matches!(target, HitTarget::FolderBrowserRow(_)) {
            self.folder_browser.last_click = None;
        }
        if self.activate_folder_action_hit(target) {
            return true;
        }
        let before = self.folder_browser.state.as_ref().map(navigation_key);
        let mut project_open_dispatched = false;
        match target {
            HitTarget::FolderBrowserRow(index) => {
                self.set_folder_navigation_focus(Focus::Folders);
                let state = self.folder_browser.state.as_mut().unwrap();
                if !state.expanded || !state.listing_ready() || *index >= state.entries.len() {
                    return true;
                }
                state.selected = Some(*index);
                let now = std::time::Instant::now();
                let double = self.folder_browser.last_click.is_some_and(|(old, when)| {
                    old == *index
                        && now.duration_since(when) < std::time::Duration::from_millis(400)
                });
                self.folder_browser.last_click = if double { None } else { Some((*index, now)) };
                if double {
                    project_open_dispatched = self
                        .folder_browser
                        .state
                        .as_ref()
                        .unwrap()
                        .opens_selected_manifest();
                    self.activate_folder_browser_focus(Focus::Folders);
                }
            }
            HitTarget::FolderBrowser(focus) => {
                if matches!(focus, Focus::Up | Focus::Home | Focus::Folders)
                    && !self.folder_browser.state.as_ref().unwrap().expanded
                {
                    return false;
                }
                self.set_folder_navigation_focus(*focus);
                if matches!(focus, Focus::Up | Focus::Home | Focus::Browse) {
                    self.activate_folder_browser_focus(*focus);
                }
            }
            _ => return false,
        }
        if !project_open_dispatched
            && !self.workspace().ui.project_open.busy
            && before != self.folder_browser.state.as_ref().map(navigation_key)
        {
            self.session.workspace_mut().ui.project_open.refusal = None;
        }
        self.refresh_folder_browser();
        true
    }
    fn activate_folder_browser_focus(&mut self, focus: Focus) {
        let Some(state) = self.folder_browser.state.as_ref() else {
            return;
        };
        if focus == Focus::Browse {
            let state = self.folder_browser.state.as_mut().unwrap();
            state.expanded = true;
            let path = (!state.ready()).then(|| entered_path(&state.path_input));
            if let Some(path) = path {
                match path {
                    Ok(path) => self.read_folder_navigation(path, false),
                    Err(error) => self.folder_browser.state.as_mut().unwrap().error = Some(error),
                }
            }
            self.set_folder_navigation_focus(Focus::Folders);
            return;
        }
        let path = match focus {
            Focus::Path => entered_path(&state.path_input),
            Focus::Home => std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .ok_or_else(|| "Home folder unavailable. Enter an absolute folder path.".into()),
            Focus::Up => entered_path(&state.path_input)
                .map(|p| p.parent().unwrap_or(Path::new("/")).to_path_buf()),
            Focus::Folders => {
                if let Some(value) = state.selected.and_then(|i| state.suggestion_path(i)) {
                    self.accept_path_assistance(value);
                    return;
                }
                if state.opens_selected_manifest() {
                    self.folder_browser.last_click = None;
                    self.begin_project_open();
                    return;
                }
                let Some(path) = selected_folder_path(state) else {
                    return;
                };
                path
            }
            Focus::Preview
            | Focus::Browse
            | Focus::NameEditor
            | Focus::NameCancel
            | Focus::NameSubmit => return,
        };
        match path {
            Ok(path) => {
                self.folder_browser.last_click = None;
                self.start_folder_navigation(path);
            }
            Err(error) => self.folder_browser.state.as_mut().unwrap().error = Some(error),
        }
    }
    pub(crate) fn handle_folder_browser_key(&mut self, event: &KeyEvent) -> bool {
        if !self.navigation_enabled() || event.state != ElementState::Pressed {
            return false;
        }
        self.folder_browser.last_click = None;
        if self.handle_path_assistance_key(&event.logical_key) {
            return true;
        }
        if self.folder_browser.state.as_ref().unwrap().expanded
            && self.modifiers.alt_key()
            && matches!(
                event.logical_key,
                Key::Named(NamedKey::ArrowUp | NamedKey::Home)
            )
        {
            self.activate_folder_browser_focus(
                if event.logical_key == Key::Named(NamedKey::Home) {
                    Focus::Home
                } else {
                    Focus::Up
                },
            );
            self.refresh_folder_browser();
            return true;
        }
        let before = self.folder_browser.state.as_ref().map(navigation_key);
        let Some(focus) = self.folder_navigation_focus() else {
            return false;
        };
        let state = self.folder_browser.state.as_mut().unwrap();
        if matches!(focus, Focus::Up | Focus::Home | Focus::Folders) && !state.expanded {
            return false;
        }
        match &event.logical_key {
            Key::Named(NamedKey::Tab | NamedKey::Escape) => return false,
            Key::Named(NamedKey::Enter) => self.activate_folder_browser_focus(focus),
            Key::Named(NamedKey::Space)
                if matches!(focus, Focus::Up | Focus::Home | Focus::Browse) =>
            {
                self.activate_folder_browser_focus(focus)
            }
            Key::Named(NamedKey::ArrowDown | NamedKey::ArrowUp) if focus == Focus::Folders => state
                .move_selection(if event.logical_key == Key::Named(NamedKey::ArrowDown) {
                    1
                } else {
                    -1
                }),
            Key::Named(NamedKey::PageDown | NamedKey::PageUp) if focus == Focus::Folders => {
                let page = isize::try_from(state.visible_rows.len().max(1)).unwrap_or(isize::MAX);
                state.move_selection(if event.logical_key == Key::Named(NamedKey::PageDown) {
                    page
                } else {
                    -page
                });
            }
            Key::Named(NamedKey::Home | NamedKey::End) if focus == Focus::Folders => {
                state.selected = (!state.entries.is_empty()).then(|| {
                    if event.logical_key == Key::Named(NamedKey::Home) {
                        0
                    } else {
                        state.entries.len() - 1
                    }
                })
            }
            Key::Named(NamedKey::Backspace) if focus == Focus::Folders => {
                self.activate_folder_browser_focus(Focus::Up)
            }
            _ => return false,
        }
        if !self.workspace().ui.project_open.busy
            && before != self.folder_browser.state.as_ref().map(navigation_key)
        {
            self.session.workspace_mut().ui.project_open.refusal = None;
        }
        self.refresh_folder_browser();
        true
    }
}

fn navigation_key(state: &FolderBrowserState) -> (String, Option<usize>) {
    (state.path_input.clone(), state.selected)
}

/// Resolve the displayed spelling before row navigation, just as scanning and
/// final actions do. A tilde or relative Location is never a literal child root.
pub(super) fn selected_folder_path(state: &FolderBrowserState) -> Option<Result<PathBuf, String>> {
    let entry = state
        .selected
        .and_then(|i| state.entries.get(i))
        .filter(|e| e.navigable())?;
    Some(entered_path(&state.directory).map(|parent| parent.join(&entry.name)))
}

#[cfg(test)]
#[path = "folder_browser_manifest_tests.rs"]
mod manifest_tests;
