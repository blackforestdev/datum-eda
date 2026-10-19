//! One value/cursor/selection/clipboard input controller for native form fields.
use super::*;
use datum_gui_protocol::{NewProjectFocus, single_line_editor::SingleLineEditor};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    ProjectName,
    FolderName,
    Location,
    EntryName,
}
pub(super) struct EditorDrag {
    target: Target,
    region: datum_gui_render::HitRegion,
    scale: f32,
}
struct Snapshot {
    text: String,
    editor: SingleLineEditor,
    editable: bool,
}
impl Runtime {
    fn active_editor(&self) -> Option<Target> {
        let ui = &self.workspace().ui;
        if ui.new_project.open {
            match ui.new_project.focus {
                NewProjectFocus::ProjectName => return Some(Target::ProjectName),
                NewProjectFocus::FolderName => return Some(Target::FolderName),
                _ => {}
            }
        }
        match self.folder_navigation_focus() {
            Some(Focus::Path) => Some(Target::Location),
            Some(Focus::NameEditor) => Some(Target::EntryName),
            _ => None,
        }
    }
    fn editor_snapshot(&self, target: Target) -> Option<Snapshot> {
        let ui = &self.workspace().ui;
        match target {
            Target::ProjectName => Some(Snapshot {
                text: ui.new_project.project_name.clone(),
                editor: ui.new_project.project_name_editor,
                editable: ui.new_project.open,
            }),
            Target::FolderName => Some(Snapshot {
                text: ui.new_project.effective_folder_name().into(),
                editor: ui.new_project.folder_name_editor,
                editable: ui.new_project.open && !ui.new_project.use_project_name,
            }),
            Target::Location => self.folder_browser.state.as_ref().map(|s| Snapshot {
                text: s.path_input.clone(),
                editor: s.path_editor,
                editable: !ui.project_open.busy,
            }),
            Target::EntryName => self
                .folder_browser
                .state
                .as_ref()?
                .naming
                .as_ref()
                .map(|d| Snapshot {
                    text: d.value.clone(),
                    editor: d.editor,
                    editable: !d.busy,
                }),
        }
    }
    fn store_editor(&mut self, target: Target, snapshot: Snapshot, changed: bool) {
        match target {
            Target::ProjectName => {
                let d = &mut self.session.workspace_mut().ui.new_project;
                d.project_name = snapshot.text;
                d.project_name_editor = snapshot.editor;
                if changed {
                    d.clear_creation_refusal();
                }
            }
            Target::FolderName => {
                let d = &mut self.session.workspace_mut().ui.new_project;
                if snapshot.editable {
                    d.folder_name = snapshot.text;
                }
                d.folder_name_editor = snapshot.editor;
                if changed {
                    d.clear_creation_refusal();
                }
            }
            Target::Location => {
                if let Some(s) = &mut self.folder_browser.state {
                    s.path_input = snapshot.text;
                    s.path_editor = snapshot.editor;
                }
                if changed {
                    self.folder_browser.invalidate_listing();
                    let state = self.folder_browser.state.as_ref().unwrap();
                    if !state.path_input.is_empty() {
                        match entered_path(&state.path_input) {
                            Ok(path) => self.read_folder_request(path, false, true),
                            Err(error) => {
                                self.folder_browser.state.as_mut().unwrap().error = Some(error)
                            }
                        }
                    }
                    self.session.workspace_mut().ui.project_open.refusal = None;
                }
            }
            Target::EntryName => {
                if let Some(d) = self
                    .folder_browser
                    .state
                    .as_mut()
                    .and_then(|s| s.naming.as_mut())
                {
                    if snapshot.editable {
                        d.value = snapshot.text;
                    }
                    d.editor = snapshot.editor;
                    if changed {
                        d.refusal = None;
                    }
                }
            }
        }
        if changed && let Some(state) = &mut self.folder_browser.state {
            state.input_refusal = None;
        }
    }
    pub(crate) fn handle_shared_editor_key(&mut self, event: &KeyEvent) -> bool {
        if event.state != ElementState::Pressed
            || self
                .folder_browser
                .state
                .as_ref()
                .is_some_and(|s| s.menu.is_some())
        {
            return false;
        }
        let Some(target) = self.active_editor() else {
            return false;
        };
        let Some(mut snapshot) = self.editor_snapshot(target) else {
            return false;
        };
        let before = snapshot.text.clone();
        let key = &event.logical_key;
        let result: Result<(), String> = (|| {
            if self.modifiers.control_key()
                && !self.modifiers.alt_key()
                && !self.modifiers.super_key()
            {
                if let Key::Character(letter) = key {
                    match letter.to_ascii_lowercase().as_str() {
                        "a" => {
                            snapshot.editor.select_all(&snapshot.text);
                            return Ok(());
                        }
                        "c" | "x" => {
                            if letter.eq_ignore_ascii_case("x") && !snapshot.editable {
                                return Ok(());
                            }
                            let selected = snapshot.editor.selected_text(&snapshot.text).to_owned();
                            if !selected.is_empty() {
                                self.write_clipboard_text(&selected)
                                    .map_err(|e| format!("Clipboard write unavailable: {e:#}"))?;
                                if letter.eq_ignore_ascii_case("x") {
                                    snapshot
                                        .editor
                                        .replace(&mut snapshot.text, "")
                                        .map_err(str::to_owned)?;
                                }
                            }
                            return Ok(());
                        }
                        "v" => {
                            if snapshot.editable {
                                let value = self
                                    .read_clipboard_text()
                                    .map_err(|e| format!("Clipboard read unavailable: {e:#}"))?;
                                if !value.is_empty() {
                                    snapshot
                                        .editor
                                        .replace(&mut snapshot.text, &value)
                                        .map_err(str::to_owned)?;
                                }
                            }
                            return Ok(());
                        }
                        _ => return Err(String::new()),
                    }
                }
                return Err(String::new());
            }
            let extend = self.modifiers.shift_key();
            match key {
                Key::Named(NamedKey::ArrowLeft) => snapshot.editor.left(&snapshot.text, extend),
                Key::Named(NamedKey::ArrowRight) => snapshot.editor.right(&snapshot.text, extend),
                Key::Named(NamedKey::Home) => snapshot.editor.home(&snapshot.text, extend),
                Key::Named(NamedKey::End) => snapshot.editor.end(&snapshot.text, extend),
                Key::Named(NamedKey::Backspace) => {
                    if snapshot.editable {
                        snapshot.editor.backspace(&mut snapshot.text);
                    }
                }
                Key::Named(NamedKey::Delete) => {
                    if snapshot.editable {
                        snapshot.editor.delete(&mut snapshot.text);
                    }
                }
                key => {
                    let Some(value) =
                        crate::global_preferences_window::dialog_text_input(key, self.modifiers)
                    else {
                        return Err(String::new());
                    };
                    if snapshot.editable {
                        snapshot
                            .editor
                            .replace(&mut snapshot.text, value)
                            .map_err(str::to_owned)?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                let changed = before != snapshot.text;
                self.store_editor(target, snapshot, changed);
            }
            Err(reason) if reason.is_empty() => return false,
            Err(reason) => {
                if target == Target::EntryName {
                    self.folder_input_refusal(reason);
                } else if let Some(s) = &mut self.folder_browser.state {
                    s.input_refusal = Some(reason);
                }
            }
        }
        self.refresh_folder_browser();
        true
    }
    pub(crate) fn start_dialog_editor_pointer(
        &mut self,
        region: datum_gui_render::HitRegion,
        x: f32,
        scale: f32,
    ) -> bool {
        let target = match region.target {
            HitTarget::NewProjectName => Target::ProjectName,
            HitTarget::NewProjectFolderName => Target::FolderName,
            HitTarget::FolderBrowser(Focus::Path) => Target::Location,
            HitTarget::FolderBrowser(Focus::NameEditor) => Target::EntryName,
            _ => return false,
        };
        let Some(mut snapshot) = self.editor_snapshot(target) else {
            return false;
        };
        let Ok(byte) = datum_gui_render::single_line_caret_at(
            &snapshot.text,
            snapshot.editor,
            region.rect.width / scale,
            (x - region.rect.x) / scale,
        ) else {
            return true;
        };
        snapshot
            .editor
            .position(&snapshot.text, byte, self.modifiers.shift_key());
        self.store_editor(target, snapshot, false);
        match target {
            Target::ProjectName => {
                self.session.workspace_mut().ui.new_project.focus = NewProjectFocus::ProjectName
            }
            Target::FolderName => {
                self.session.workspace_mut().ui.new_project.focus = NewProjectFocus::FolderName
            }
            Target::Location => self.set_folder_navigation_focus(Focus::Path),
            Target::EntryName => self.set_folder_navigation_focus(Focus::NameEditor),
        }
        self.folder_browser.editor_drag = Some(EditorDrag {
            target,
            region,
            scale,
        });
        self.refresh_folder_browser();
        true
    }
    pub(crate) fn drag_dialog_editor_pointer(&mut self, x: f32) -> bool {
        let Some(drag) = &self.folder_browser.editor_drag else {
            return false;
        };
        let target = drag.target;
        let width = drag.region.rect.width / drag.scale;
        let offset = (x - drag.region.rect.x) / drag.scale;
        let Some(mut snapshot) = self.editor_snapshot(target) else {
            self.folder_browser.editor_drag = None;
            return false;
        };
        if let Ok(byte) =
            datum_gui_render::single_line_caret_at(&snapshot.text, snapshot.editor, width, offset)
        {
            snapshot.editor.position(&snapshot.text, byte, true);
            self.store_editor(target, snapshot, false);
            self.refresh_folder_browser();
        }
        true
    }
    pub(crate) fn finish_dialog_editor_pointer(&mut self) -> bool {
        self.folder_browser.editor_drag.take().is_some()
    }
}
