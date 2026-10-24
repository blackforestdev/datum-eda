//! Existing marking-menu entry and read-only selection leaves.
use super::*;
use datum_gui_protocol::SelectionTarget;
use datum_gui_protocol::selection_resolution::SelectionAuthority;
use datum_gui_viewport::selection_acquisition::{AcquisitionEntry, AcquisitionMode};
use std::collections::BTreeSet;
impl Runtime {
    /// Open the existing marking menu with source-bound read-only Select leaves
    /// for either available native surface. General schematic authoring stays S7.
    pub(super) fn open_marking_menu_at_cursor(&mut self) -> bool {
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        if self.cursor_in_dock() {
            return false;
        }
        let world_point = {
            let Some(prepared) = self.prepared_scene() else {
                return false;
            };
            prepared.world_point_at_screen(x, y)
        };
        let Some((world_point, surface)) = world_point else {
            return false;
        };
        let pane = self.pane_at_screen(x, y);
        let basis = match self.native_binding_basis(surface, pane) {
            Ok(basis) => basis,
            Err(error) => {
                self.selection_notice(error.to_string());
                return true;
            }
        };
        if !self.ensure_retained_scene() {
            return false;
        }
        let retained = match surface {
            SceneSurface::Board => self.renderer.render_session().board(),
            SceneSurface::Schematic => {
                pane.and_then(|pane| self.renderer.render_session().schematic_for_pane(pane))
            }
        };
        let Some(retained) = retained else {
            return false;
        };
        let hits = retained.authored_overlap_world(world_point, self.workspace(), |target| {
            let Some(basis) = &basis else {
                return true;
            };
            let HitTarget::AuthoredObject(key) = target else {
                return false;
            };
            basis
                .bindings
                .get(key)
                .is_some_and(|binding| binding.eligible(&self.workspace().ui.filters))
        });
        let retained_target = hits.first().cloned();
        let selection = basis.as_ref().map(|basis| {
            let origins = hits
                .iter()
                .filter_map(|target| {
                    let HitTarget::AuthoredObject(key) = target else {
                        return None;
                    };
                    let b = basis.bindings.get(key)?;
                    Some((b.acquired.clone(), b.projected.clone()))
                })
                .collect();
            let mut menu = datum_gui_protocol::selection_menu::SelectionMenu::capture(
                &self.native_selection.as_ref().unwrap().authority,
                origins,
                world_point,
                self.workspace().ui.filters.clone(),
            );
            menu.schematic = surface == SceneSurface::Schematic;
            menu.source_scene = if menu.schematic {
                self.workspace()
                    .schematic_scene
                    .as_ref()
                    .unwrap()
                    .scene_id
                    .clone()
            } else {
                self.workspace().scene.scene_id.clone()
            };
            menu.filters = self.workspace().ui.filters.clone();
            let scene = if menu.schematic {
                self.workspace().schematic_scene.as_ref().unwrap()
            } else {
                &self.workspace().scene
            };
            menu.bind_presentation(
                scene,
                &basis.bindings,
                &self.native_selection.as_ref().unwrap().authority,
            );
            menu.append_checks(&self.workspace().checks);
            if let SelectionTarget::Native(view) = &self.workspace().selection {
                menu.append_inspection(view);
                menu.append_inspector_pages(
                    self.workspace().ui.selection_inspector.inventory_expanded,
                    view.scope,
                );
            }
            menu
        });
        let target_object_id = match retained_target {
            Some(HitTarget::AuthoredObject(id)) | Some(HitTarget::ReviewAction(id)) => Some(id),
            _ => None,
        };
        let menu_key = if surface == SceneSurface::Schematic {
            "schematic.selection".into()
        } else {
            marking_menu_key_for_target(target_object_id.as_deref())
        };
        let ui = &mut self.session.workspace_mut().ui;
        ui.active_menu = None;
        ui.marking_menu = Some(MarkingMenuState {
            menu_key,
            target_object_id,
            anchor_x_px: x.round() as i32,
            anchor_y_px: y.round() as i32,
            preview_slot: None,
            gesture_dx_px: 0,
            gesture_dy_px: 0,
            selection,
        });
        // TF-01: while open, the marking menu is a transient Overlay keyboard
        // owner; `dismiss_marking_menu` restores Editor ownership.
        self.set_application_focus(datum_gui_protocol::ApplicationFocus::Overlay);
        self.invalidate_frame();
        true
    }
}
impl Runtime {
    pub(super) fn open_native_select_list(&mut self) -> bool {
        let Some(menu) = self
            .session
            .workspace_mut()
            .ui
            .marking_menu
            .as_mut()
            .and_then(|m| m.selection.as_mut())
        else {
            return false;
        };
        menu.open = true;
        menu.focus = 0;
        menu.preview = Some(0);
        menu.scroll = 0;
        self.invalidate_frame();
        true
    }
    pub(super) fn activate_native_select_item(&mut self, index: usize) -> bool {
        let (project, revision, schematic, source_scene, filters, inspection_subject, item) = {
            let Some(menu) = self
                .workspace()
                .ui
                .marking_menu
                .as_ref()
                .and_then(|m| m.selection.as_ref())
            else {
                return false;
            };
            let Some(item) = menu.items.get(index) else {
                return true;
            };
            (
                menu.project,
                menu.revision.clone(),
                menu.schematic,
                menu.source_scene.clone(),
                menu.filters.clone(),
                menu.inspection_subject.clone(),
                item.clone(),
            )
        };
        if let Some(reason) = item.unavailable {
            self.selection_notice(reason);
            return true;
        }
        match self.ensure_selection_authority() {
            Ok(true) => {}
            _ => {
                self.selection_notice("Native selection authority is unavailable".into());
                return true;
            }
        }
        let authority = &self.native_selection.as_ref().unwrap().authority;
        let scene_id = if schematic {
            self.workspace()
                .schematic_scene
                .as_ref()
                .map(|s| s.scene_id.as_str())
        } else {
            Some(self.workspace().scene.scene_id.as_str())
        };
        if authority.project() != project
            || authority.revision().0 != revision
            || self.workspace().ui.filters != filters
            || scene_id != Some(source_scene.as_str())
        {
            self.dismiss_marking_menu();
            self.selection_notice("Selection menu source revision changed; reopen it".into());
            return true;
        }
        use datum_gui_protocol::selection_menu::Command;
        if item.command == Command::All {
            let candidates = self
                .workspace()
                .ui
                .marking_menu
                .as_ref()
                .unwrap()
                .selection
                .as_ref()
                .unwrap()
                .overlap_candidates()
                .cloned()
                .collect();
            self.dismiss_marking_menu();
            self.cancel_native_selection_gesture();
            self.native_selection.as_mut().unwrap().reset_click_depth();
            return self.publish_native_acquisition(
                candidates,
                AcquisitionMode::Replace,
                AcquisitionEntry::Region,
                None,
            );
        }
        if let Command::Diagnostic(fingerprint) = &item.command {
            self.dismiss_marking_menu();
            return self
                .acquire_native_artifact(datum_gui_protocol::SelectionSubject::Diagnostic(
                    fingerprint.clone(),
                ))
                .unwrap_or(false);
        }
        if matches!(
            item.command,
            Command::InspectScope(_)
                | Command::InventoryToggle
                | Command::InspectorPage { .. }
                | Command::FilterToggle(_)
        ) {
            if !matches!(&self.workspace().selection,SelectionTarget::Native(v) if Some(&v.subject)==inspection_subject.as_ref())
            {
                self.dismiss_marking_menu();
                self.selection_notice("Inspector selection changed; reopen the menu".into());
                return true;
            }
            // Read the presented Inspector before menu dismissal invalidates composition.
            if let Command::InspectorPage { page, scope } = item.command {
                if !matches!(&self.workspace().selection, SelectionTarget::Native(view) if view.scope == scope)
                {
                    self.dismiss_marking_menu();
                    self.selection_notice("Inspector scope changed; reopen the menu".into());
                    return true;
                }
                let changed = self.navigate_selection_inspector(page);
                self.dismiss_marking_menu();
                return changed;
            }
            self.dismiss_marking_menu();
            return match item.command {
                Command::InspectScope(scope) => self.set_selection_scope(scope),
                Command::InventoryToggle => self.toggle_selection_inventory(),
                Command::FilterToggle(class) => self.toggle_selection_class(class),
                _ => unreachable!(),
            };
        }
        let subject = match datum_gui_protocol::selection_menu::resolve(authority, &item.command) {
            Ok(subject) => subject,
            Err(e) => {
                self.selection_notice(format!("Selection refused atomically: {e:?}"));
                return true;
            }
        };
        self.dismiss_marking_menu();
        self.cancel_native_selection_gesture();
        self.native_selection.as_mut().unwrap().reset_click_depth();
        if let Some(subject) = subject {
            self.publish_native_acquisition(
                BTreeSet::new(),
                AcquisitionMode::Replace,
                AcquisitionEntry::Individual,
                Some(subject),
            )
        } else {
            true
        }
    }
    pub(super) fn handle_marking_selection_key(&mut self, event: &KeyEvent) -> bool {
        let Some(menu) = self
            .workspace()
            .ui
            .marking_menu
            .as_ref()
            .and_then(|m| m.selection.as_ref())
        else {
            return false;
        };
        if event.state != ElementState::Pressed {
            return true;
        }
        match selection_key_intent(menu, &event.logical_key) {
            SelectionKeyIntent::Close => {
                self.dismiss_marking_menu();
            }
            SelectionKeyIntent::Open => {
                self.open_native_select_list();
            }
            SelectionKeyIntent::Activate(index) => {
                self.activate_native_select_item(index);
            }
            SelectionKeyIntent::Focus(next) => {
                let menu = self
                    .session
                    .workspace_mut()
                    .ui
                    .marking_menu
                    .as_mut()
                    .unwrap()
                    .selection
                    .as_mut()
                    .unwrap();
                menu.focus = next;
                menu.preview = Some(next);
                menu.scroll = next.saturating_sub(7);
                self.invalidate_frame();
            }
            SelectionKeyIntent::Consume => {}
        }
        true
    }
}
#[derive(Debug, PartialEq, Eq)]
enum SelectionKeyIntent {
    Consume,
    Close,
    Open,
    Focus(usize),
    Activate(usize),
}
/// Presented rows alone drive pointer preview; leaving the list clears only
/// transient pre-highlight, preserving the committed subject and keyboard focus.
pub(super) fn focus_pointer_candidate(
    menu: &mut MarkingMenuState,
    target: Option<&HitTarget>,
) -> bool {
    let Some(selection) = menu.selection.as_mut().filter(|m| m.open) else {
        return false;
    };
    let preview = match target {
        Some(HitTarget::MarkingMenuItem { menu_key, slot, .. }) if menu_key == &menu.menu_key => {
            slot.strip_prefix("SELECT:")
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|i| *i < selection.items.len())
        }
        _ => None,
    };
    let changed = selection.preview != preview || preview.is_some_and(|i| selection.focus != i);
    selection.preview = preview;
    if let Some(index) = preview {
        selection.focus = index;
    }
    changed
}
fn selection_key_intent(
    menu: &datum_gui_protocol::selection_menu::SelectionMenu,
    key: &Key,
) -> SelectionKeyIntent {
    use SelectionKeyIntent as I;
    let count = menu.items.len();
    let focus = menu.focus;
    match key {
        Key::Named(NamedKey::Escape) => I::Close,
        Key::Character(c) if !menu.open && c.eq_ignore_ascii_case("s") => I::Open,
        Key::Named(NamedKey::ArrowDown | NamedKey::Enter) if !menu.open => I::Open,
        Key::Named(NamedKey::Enter) if count > 0 => I::Activate(focus),
        Key::Named(NamedKey::ArrowDown) if count > 0 => I::Focus((focus + 1) % count),
        Key::Named(NamedKey::ArrowUp) if count > 0 => {
            I::Focus(focus.checked_sub(1).unwrap_or(count - 1))
        }
        Key::Named(NamedKey::Home) if count > 0 => I::Focus(0),
        Key::Named(NamedKey::End) if count > 0 => I::Focus(count - 1),
        _ => I::Consume,
    }
}
impl Runtime {
    pub(super) fn open_marking_menu_for_keyboard(&mut self) -> bool {
        let focus = self.workspace().ui.layout.focused;
        let previous = self.last_cursor_pos;
        let layout = self
            .current_layout()
            .viewport_panes(&self.workspace().ui.layout);
        let Some(pane) = layout.panes.iter().find(|p| p.id == focus) else {
            return false;
        };
        if !previous.is_some_and(|(x, y)| pane.rect.scene.contains(x, y)) {
            let r = pane.rect.scene;
            self.last_cursor_pos = Some((r.x + r.width * 0.5, r.y + r.height * 0.5));
        }
        let opened = self.open_marking_menu_at_cursor();
        self.last_cursor_pos = previous;
        opened
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pointer_candidate_focus_uses_presented_rows_and_clears_preview_without_activation() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        // Use the existing constructor shape below rather than a runtime/window.
        let mut menu = MarkingMenuState {
            menu_key: "board.selection".into(),
            target_object_id: None,
            anchor_x_px: 0,
            anchor_y_px: 0,
            preview_slot: None,
            gesture_dx_px: 0,
            gesture_dy_px: 0,
            selection: Some(datum_gui_protocol::selection_menu::SelectionMenu {
                project: uuid::Uuid::nil(),
                revision: "test".into(),
                schematic: false,
                source_scene: "test".into(),
                inspection_subject: None,
                filters: state.ui.filters.clone(),
                items: (0..3)
                    .map(|i| datum_gui_protocol::selection_menu::Item {
                        label: i.to_string(),
                        command: datum_gui_protocol::selection_menu::Command::All,
                        unavailable: None,
                    })
                    .collect(),
                open: true,
                focus: 0,
                preview: Some(0),
                preview_keys: Default::default(),
                scroll: 0,
            }),
        };
        let prior = state.selection.clone();
        let target = HitTarget::MarkingMenuItem {
            menu_key: menu.menu_key.clone(),
            slot: "SELECT:2".into(),
            label: "row".into(),
        };
        assert!(focus_pointer_candidate(&mut menu, Some(&target)));
        assert_eq!(menu.selection.as_ref().unwrap().focus, 2);
        assert_eq!(menu.selection.as_ref().unwrap().preview, Some(2));
        assert!(!focus_pointer_candidate(&mut menu, Some(&target)));
        assert!(focus_pointer_candidate(&mut menu, None));
        assert_eq!(menu.selection.as_ref().unwrap().focus, 2);
        assert_eq!(menu.selection.as_ref().unwrap().preview, None);
        state.ui.marking_menu = Some(menu);
        assert_eq!(state.selection, prior);
    }
    #[test]
    fn read_only_menu_keyboard_reaches_entries_beyond_transport_context_cap_and_consumes_others() {
        let mut menu = datum_gui_protocol::selection_menu::SelectionMenu {
            project: datum_gui_protocol::SelectionProjectId::nil(),
            revision: "test".into(),
            schematic: true,
            source_scene: "test".into(),
            inspection_subject: None,
            filters: datum_gui_protocol::load_fixture_workspace_state()
                .ui
                .filters,
            items: (0..303)
                .map(|n| datum_gui_protocol::selection_menu::Item {
                    label: n.to_string(),
                    command: datum_gui_protocol::selection_menu::Command::All,
                    unavailable: None,
                })
                .collect(),
            open: false,
            focus: 0,
            preview: None,
            preview_keys: Default::default(),
            scroll: 0,
        };
        assert_eq!(
            selection_key_intent(&menu, &Key::Character("s".into())),
            SelectionKeyIntent::Open
        );
        menu.open = true;
        assert_eq!(
            selection_key_intent(&menu, &Key::Named(NamedKey::End)),
            SelectionKeyIntent::Focus(302)
        );
        menu.focus = 302;
        assert_eq!(
            selection_key_intent(&menu, &Key::Named(NamedKey::Enter)),
            SelectionKeyIntent::Activate(302)
        );
        assert_eq!(
            selection_key_intent(&menu, &Key::Named(NamedKey::ArrowDown)),
            SelectionKeyIntent::Focus(0)
        );
        assert_eq!(
            selection_key_intent(&menu, &Key::Named(NamedKey::Escape)),
            SelectionKeyIntent::Close
        );
        assert_eq!(
            selection_key_intent(&menu, &Key::Character("m".into())),
            SelectionKeyIntent::Consume
        );
    }
}
