//! Typed read-only document actions over the shared pane/focus/camera owners.
use super::*;
use datum_gui_protocol::schematic_documents::SchematicDocumentKey;

#[derive(Default)]
pub(super) struct NavigationWork {
    last_click: Option<(
        (SchematicDocumentKey, String, std::path::PathBuf),
        std::time::Instant,
    )>,
}
impl Runtime {
    fn navigation_basis(&self, key: &SchematicDocumentKey, revision: &str) -> bool {
        let state = self.workspace();
        state.schematic_documents.project == Some(key.project)
            && state
                .schematic_documents
                .revision
                .as_ref()
                .is_some_and(|r| r.0 == revision)
            && state.schematic_documents.row(key).is_some()
            && state.scene.project_uuid == key.project.to_string()
            && state.scene.source_revision == revision
            && state.library.basis.as_ref().is_some_and(|model| {
                model.project.project_id == key.project && model.model_revision.0 == revision
            })
    }
    pub(super) fn focus_schematic_navigator(&mut self) -> bool {
        if self.workspace().schematic_documents.project.is_none() {
            return false;
        }
        if self.workspace().schematic_documents.inspected.is_none() {
            let first = self
                .workspace()
                .schematic_documents
                .visible_rows()
                .first()
                .map(|r| r.key.clone());
            if let Some(key) = first {
                self.session
                    .workspace_mut()
                    .schematic_documents
                    .inspect(&key);
            }
        }
        self.set_application_focus(ApplicationFocus::Navigator);
        self.invalidate_frame();
        true
    }
    fn open_schematic_document(&mut self, key: &SchematicDocumentKey, revision: &str) -> bool {
        let result = (|| -> anyhow::Result<()> {
            anyhow::ensure!(
                self.navigation_basis(key, revision),
                "Document view source changed; refresh Navigator"
            );
            let model = self
                .workspace()
                .library
                .basis
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Document source unavailable"))?;
            let root = self
                .workspace()
                .backing
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Document origin unavailable"))?
                .request
                .project_root
                .clone();
            self.session
                .workspace_mut()
                .schematic_documents
                .prepare_scene(key, &root, &model)?;
            let mut opened = None;
            self.change_pane_views(|state| {
                opened = state
                    .schematic_documents
                    .open_beside(&mut state.ui.layout, key);
            });
            anyhow::ensure!(opened.is_some(), "Qualified document cannot be opened");
            Ok(())
        })();
        if let Err(error) = result {
            self.log_console_refusal(ConsoleFeedbackSource::Viewport, error.to_string());
            self.invalidate_frame();
        }
        true
    }
    pub(super) fn activate_schematic_navigation_target(
        &mut self,
        target: &HitTarget,
    ) -> Option<bool> {
        match target {
            HitTarget::FocusNavigator => return Some(self.focus_schematic_navigator()),
            HitTarget::FocusInspector => {
                self.set_application_focus(ApplicationFocus::Inspector);
                self.invalidate_frame();
                return Some(true);
            }
            HitTarget::SchematicNavigatorScroll { .. } => return Some(true),
            HitTarget::SchematicDocumentRow { document, revision }
            | HitTarget::SchematicDocumentExpand { document, revision }
            | HitTarget::SchematicDocumentOpen { document, revision } => {
                if !self.navigation_basis(document, revision) {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Viewport,
                        "Document row source changed; refresh Navigator",
                    );
                    return Some(true);
                }
                if matches!(target, HitTarget::SchematicDocumentOpen { .. }) {
                    return Some(self.open_schematic_document(document, revision));
                }
                self.set_application_focus(ApplicationFocus::Navigator);
                if matches!(target, HitTarget::SchematicDocumentExpand { .. }) {
                    self.schematic_navigation.last_click = None;
                    self.session
                        .workspace_mut()
                        .schematic_documents
                        .toggle_expanded(document);
                } else {
                    self.session
                        .workspace_mut()
                        .schematic_documents
                        .inspect(document);
                    let root = self
                        .workspace()
                        .backing
                        .as_ref()
                        .map(|b| b.request.project_root.clone())
                        .unwrap_or_default();
                    let double = crate::gui_runtime_support::register_navigation_click(
                        &mut self.schematic_navigation.last_click,
                        (document.clone(), revision.clone(), root),
                        std::time::Instant::now(),
                    );
                    if double {
                        return Some(self.open_schematic_document(document, revision));
                    }
                }
                self.invalidate_frame();
                Some(true)
            }
            _ => None,
        }
    }
    fn navigation_page(&self) -> Option<(usize, usize)> {
        self.presented_hits
            .regions()
            .iter()
            .find_map(|region| match &region.target {
                HitTarget::SchematicNavigatorScroll {
                    project,
                    revision,
                    total,
                    visible,
                } if self.workspace().schematic_documents.project == Some(*project)
                    && self
                        .workspace()
                        .schematic_documents
                        .revision
                        .as_ref()
                        .is_some_and(|r| &r.0 == revision) =>
                {
                    Some((*total, *visible))
                }
                _ => None,
            })
    }
    pub(super) fn handle_schematic_navigation_scroll(&mut self, lines: f32) -> Option<bool> {
        let (x, y) = self.last_cursor_pos?;
        let within = self.presented_hits.regions().iter().any(|region| {
            region.rect.contains(x, y)
                && matches!(region.target, HitTarget::SchematicNavigatorScroll { .. })
        });
        if !within {
            return None;
        }
        let Some((total, visible)) = self.navigation_page() else {
            return Some(false);
        };
        let offset = self.workspace().schematic_documents.scroll_offset;
        let next = datum_gui_viewport::scroll::scrolled_row_offset(offset, total, visible, lines);
        self.session
            .workspace_mut()
            .schematic_documents
            .scroll_offset = next;
        if next != offset {
            self.invalidate_frame();
        }
        Some(next != offset)
    }
    pub(super) fn handle_schematic_navigation_key(
        &mut self,
        event: &winit::event::KeyEvent,
    ) -> bool {
        use winit::keyboard::{Key, NamedKey};
        let focus = self.application_focus();
        if !matches!(
            focus,
            ApplicationFocus::Navigator | ApplicationFocus::Inspector
        ) {
            return false;
        }
        // Keep both halves with the local owner. Handing focus back on press
        // lets the editor interpret the release as a selection cancellation.
        if event.logical_key == Key::Named(NamedKey::Escape) {
            if event.state == winit::event::ElementState::Released {
                self.set_application_focus(ApplicationFocus::Editor(
                    self.workspace().ui.layout.focused,
                ));
                self.invalidate_frame();
            }
            return true;
        }
        if event.state != winit::event::ElementState::Pressed {
            return true;
        }
        if focus == ApplicationFocus::Inspector
            && !matches!(
                event.logical_key,
                Key::Named(NamedKey::Escape | NamedKey::Tab)
            )
        {
            return self.handle_inspector_navigation_key(event);
        }
        match &event.logical_key {
            Key::Named(NamedKey::Tab) => {
                let next = match (focus, self.modifiers.shift_key()) {
                    (ApplicationFocus::Navigator, false) => ApplicationFocus::Inspector,
                    (ApplicationFocus::Inspector, true) => ApplicationFocus::Navigator,
                    _ => ApplicationFocus::Editor(self.workspace().ui.layout.focused),
                };
                self.set_application_focus(next);
            }
            Key::Named(NamedKey::Enter)
                if focus == ApplicationFocus::Navigator && !event.repeat =>
            {
                if let Some(key) = self.workspace().schematic_documents.inspected.clone() {
                    let revision = self
                        .workspace()
                        .schematic_documents
                        .revision
                        .as_ref()
                        .map(|r| r.0.clone())
                        .unwrap_or_default();
                    return self.open_schematic_document(&key, &revision);
                }
            }
            Key::Named(NamedKey::ArrowDown | NamedKey::ArrowUp)
                if focus == ApplicationFocus::Navigator =>
            {
                let delta = if event.logical_key == Key::Named(NamedKey::ArrowUp) {
                    -1
                } else {
                    1
                };
                self.session
                    .workspace_mut()
                    .schematic_documents
                    .move_inspection(delta);
                if let Some((_, visible)) = self.navigation_page() {
                    let state = self.session.workspace_mut();
                    let rows = state.schematic_documents.visible_rows();
                    let index = rows.iter().position(|row| {
                        Some(&row.key) == state.schematic_documents.inspected.as_ref()
                    });
                    if let Some(index) = index {
                        let offset = state.schematic_documents.scroll_offset;
                        state.schematic_documents.scroll_offset = if index < offset {
                            index
                        } else if index >= offset + visible {
                            index.saturating_sub(visible.saturating_sub(1))
                        } else {
                            offset
                        };
                    }
                }
            }
            Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowRight)
                if focus == ApplicationFocus::Navigator =>
            {
                let state = self.session.workspace_mut();
                if let Some(key) = state.schematic_documents.inspected.clone() {
                    if event.logical_key == Key::Named(NamedKey::ArrowLeft) {
                        state.schematic_documents.expanded.remove(&key);
                    } else {
                        state.schematic_documents.expanded.insert(key);
                    }
                }
            }
            _ => {}
        }
        self.invalidate_frame();
        true
    }
}
