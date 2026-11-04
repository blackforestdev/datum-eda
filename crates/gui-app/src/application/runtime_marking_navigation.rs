//! Existing wheel/submenu navigation, including its declared linear overflow.
use super::*;
impl Runtime {
    /// Resolve the item at `slot` in the currently-open wheel (top-level cardinals/
    /// secondaries, or the `SUB:n` leaves of the sub-wheel named by the last
    /// `submenu_path` entry); if it is a submenu parent, push its key so the
    /// sub-wheel opens. Returns true when a sub-wheel was opened — navigation only,
    /// no operation is invoked.
    pub(super) fn marking_open_submenu_if_parent(&mut self, menu_key: &str, slot: &str) -> bool {
        let Some(submenu_key) = self.resolve_marking_submenu_at(menu_key, slot) else {
            return false;
        };
        let Some(menu) = self.session.workspace_mut().ui.marking_menu.as_mut() else {
            return false;
        };
        menu.submenu_path.push(submenu_key);
        menu.preview_slot = None;
        self.invalidate_frame();
        true
    }

    fn resolve_marking_submenu_at(&self, menu_key: &str, slot: &str) -> Option<String> {
        let model = datum_gui_protocol::gui_menu_model::default_gui_menu_model().ok()?;
        let open = self
            .workspace()
            .ui
            .marking_menu
            .as_ref()?
            .submenu_path
            .last()
            .cloned();
        Self::resolve_submenu_key(model, menu_key, open.as_deref(), slot)
    }

    /// Pure resolution: the submenu key a `slot` opens, or None for an inert leaf.
    /// `open_submenu` is the currently-open sub-wheel (None at top level, where
    /// `slot` is a compass direction; otherwise `slot` is `SUB:n`). Kept free of
    /// runtime state so it is unit-testable against the live menu_model.
    pub(super) fn resolve_submenu_key(
        model: &datum_gui_protocol::gui_menu_model::GuiMenuModel,
        menu_key: &str,
        open_submenu: Option<&str>,
        slot: &str,
    ) -> Option<String> {
        let menu = model.marking_menus.get(menu_key)?;
        let item = if open_submenu == Some(MarkingMenuState::OVERFLOW) {
            let index: usize = slot.strip_prefix("MORE:")?.parse().ok()?;
            menu.overflow.get(index)?
        } else if let Some(open) = open_submenu {
            let index: usize = slot.strip_prefix("SUB:")?.parse().ok()?;
            menu.submenus.get(open)?.get(index)?
        } else {
            menu.cardinal
                .get(slot)
                .or_else(|| menu.secondary.get(slot))?
        };
        match item.binding() {
            datum_gui_protocol::GuiMenuBinding::Submenu(key) => Some(key.to_string()),
            _ => None,
        }
    }

    /// Pop one sub-wheel level (back). Returns false at the root (nothing to pop).
    pub(super) fn marking_pop_submenu(&mut self) -> bool {
        let Some(menu) = self.session.workspace_mut().ui.marking_menu.as_mut() else {
            return false;
        };
        if menu.submenu_path.is_empty() {
            return false;
        }
        menu.submenu_path.pop();
        menu.preview_slot = None;
        self.invalidate_frame();
        true
    }

    pub(crate) fn update_marking_menu_preview(&mut self, pos: (f32, f32)) -> bool {
        if self
            .workspace()
            .ui
            .marking_menu
            .as_ref()
            .and_then(|m| m.selection.as_ref())
            .is_some_and(|m| m.open)
        {
            let target = self.presented_hits.hit_test(pos.0, pos.1).cloned();
            let menu = self
                .session
                .workspace_mut()
                .ui
                .marking_menu
                .as_mut()
                .unwrap();
            let changed = runtime_selection_menu::focus_pointer_candidate(menu, target.as_ref());
            if changed {
                self.invalidate_frame();
            }
            return changed;
        }
        let target = self.presented_hits.hit_test(pos.0, pos.1).cloned();
        if self
            .workspace()
            .ui
            .marking_menu
            .as_ref()
            .is_some_and(|m| m.overflow_open())
        {
            if let Some(HitTarget::MarkingMenuItem { menu_key, slot, .. }) = target {
                let menu = self
                    .session
                    .workspace_mut()
                    .ui
                    .marking_menu
                    .as_mut()
                    .unwrap();
                if menu.menu_key == menu_key && menu.preview_slot.as_ref() != Some(&slot) {
                    menu.preview_slot = Some(slot);
                    self.invalidate_frame();
                    return true;
                }
            }
            return false;
        }
        let Some(menu) = self.session.workspace_mut().ui.marking_menu.as_mut() else {
            return false;
        };
        let dx = (pos.0 - menu.anchor_x_px as f32).round() as i32;
        let dy = (pos.1 - menu.anchor_y_px as f32).round() as i32;
        // Inside a sub-wheel the gesture targets a content-sized SUB:n slot; at the
        // top level it targets a compass slot.
        let next_slot = if matches!(target, Some(HitTarget::MarkingMenuItem { ref slot, .. }) if slot == "MORE" || slot == "SELECT")
        {
            match target {
                Some(HitTarget::MarkingMenuItem { slot, .. }) => Some(slot),
                _ => None,
            }
        } else if let Some(open) = menu.submenu_path.last() {
            marking_submenu_slot_for_delta(&menu.menu_key, open, dx, dy)
        } else {
            marking_slot_for_delta(dx, dy)
        };
        if menu.gesture_dx_px == dx && menu.gesture_dy_px == dy && menu.preview_slot == next_slot {
            return false;
        }
        menu.gesture_dx_px = dx;
        menu.gesture_dy_px = dy;
        menu.preview_slot = next_slot;
        self.invalidate_frame();
        true
    }

    pub(super) fn open_marking_overflow(&mut self) -> bool {
        let Ok(model) = datum_gui_protocol::gui_menu_model::default_gui_menu_model() else {
            return false;
        };
        let Some(marking) = self.session.workspace_mut().ui.marking_menu.as_mut() else {
            return false;
        };
        let Some(menu) = model.marking_menus.get(&marking.menu_key) else {
            return false;
        };
        if !marking.open_overflow(menu) {
            return false;
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn handle_marking_overflow_key(&mut self, event: &KeyEvent) -> bool {
        let Some(marking) = self.workspace().ui.marking_menu.as_ref() else {
            return false;
        };
        if marking.selection.as_ref().is_some_and(|s| s.open) {
            return false;
        }
        if !marking.overflow_open() {
            if event.state == ElementState::Pressed
                && marking
                    .submenu_path
                    .iter()
                    .any(|key| key == MarkingMenuState::OVERFLOW)
                && matches!(
                    &event.logical_key,
                    Key::Named(NamedKey::Escape | NamedKey::ArrowLeft)
                )
            {
                self.marking_pop_submenu();
                return true;
            }
            if event.state == ElementState::Pressed
                && matches!(&event.logical_key, Key::Character(c) if c.eq_ignore_ascii_case("m"))
            {
                return self.open_marking_overflow();
            }
            return false;
        }
        if event.state != ElementState::Pressed {
            return true;
        }
        match &event.logical_key {
            Key::Named(NamedKey::Escape | NamedKey::ArrowLeft) => {
                self.marking_pop_submenu();
            }
            Key::Character(c) if c.eq_ignore_ascii_case("s") => {
                self.open_native_select_list();
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.move_marking_overflow_focus(1);
            }
            Key::Named(NamedKey::ArrowUp) => {
                self.move_marking_overflow_focus(-1);
            }
            Key::Named(NamedKey::Enter | NamedKey::ArrowRight) => {
                let menu_key = marking.menu_key.clone();
                if let Some(slot) = marking.preview_slot.clone() {
                    self.activate_application_overlay_hit_target(&HitTarget::MarkingMenuItem {
                        menu_key,
                        slot,
                        label: String::new(),
                    });
                }
            }
            _ => {}
        }
        true
    }
    pub(crate) fn move_marking_overflow_focus(&mut self, step: i32) -> bool {
        let Ok(model) = datum_gui_protocol::gui_menu_model::default_gui_menu_model() else {
            return false;
        };
        let Some(marking) = self.session.workspace_mut().ui.marking_menu.as_mut() else {
            return false;
        };
        if !marking.overflow_open() || marking.selection.as_ref().is_some_and(|s| s.open) {
            return false;
        }
        let Some(menu) = model.marking_menus.get(&marking.menu_key) else {
            return false;
        };
        let slots = marking.overflow_slots(menu);
        if slots.is_empty() {
            return true;
        }
        let focus = slots
            .iter()
            .position(|slot| Some(slot) == marking.preview_slot.as_ref())
            .unwrap_or(0);
        let next = (focus as i64 + i64::from(step)).rem_euclid(slots.len() as i64) as usize;
        marking.preview_slot = Some(slots[next].clone());
        self.invalidate_frame();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marking_overflow_preserves_declared_indexes_and_escape_ownership() {
        let model = datum_gui_protocol::load_default_gui_menu_model().unwrap();
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let mut marking = MarkingMenuState {
            menu_key: "pcb.component".into(),
            target_object_id: None,
            anchor_x_px: 0,
            anchor_y_px: 0,
            preview_slot: None,
            gesture_dx_px: 0,
            gesture_dy_px: 0,
            selection: None,
            submenu_path: vec![],
        };
        let key = "pcb.component";
        let menu = &model.marking_menus[key];
        marking.menu_key = key.into();
        assert!(marking.open_overflow(menu));
        assert!(
            menu_owns_escape_press(None, Some(&marking)),
            "overflow owns Escape press and release before canvas selection"
        );
        for (index, item) in menu.overflow.iter().enumerate() {
            assert_eq!(
                Runtime::resolve_submenu_key(
                    &model,
                    key,
                    Some(MarkingMenuState::OVERFLOW),
                    &format!("MORE:{index}")
                ),
                item.submenu.clone()
            );
            assert_eq!(
                marking.menu_item(menu, &format!("MORE:{index}")),
                Some(item)
            );
        }
        assert!(marking.menu_item(menu, "SUB:0").is_none());
        state.ui.marking_menu = Some(marking);
        let nodes = datum_gui_protocol::gui_menu_model::accessibility::menu_accessibility_nodes(
            &model, &state,
        );
        assert!(
            nodes
                .iter()
                .any(|node| node.name == "Back" && node.key.as_deref() == Some("Escape"))
        );
    }
}
