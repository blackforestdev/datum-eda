//! Menubar and marking-menu interaction ownership, extracted from the runtime
//! monolith under decision 022. Menu dispatch remains session-local; mutation
//! continues through the established verb or GUI-local action boundaries.

use super::*;
use crate::runtime_view_actions::action_evidence::EntrySurface;

#[derive(Debug, Clone, PartialEq, Eq)]
enum MenuKeyIntent {
    Consume,
    CloseMenu,
    CloseSubmenu(usize),
    Focus(usize),
    OpenSubmenu(String),
    Activate { menu: String, label: String },
}

fn menu_key_intent(
    group: &datum_gui_protocol::GuiMenu,
    active_submenu: Option<&str>,
    focus_index: usize,
    key: &Key,
) -> MenuKeyIntent {
    let (menu_name, items) = if let Some(submenu) = active_submenu {
        (
            submenu,
            group
                .submenus
                .get(submenu)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
        )
    } else {
        (group.menu.as_str(), group.items.as_slice())
    };
    if items.is_empty() {
        return MenuKeyIntent::Consume;
    }
    let parent_index = || {
        group
            .items
            .iter()
            .position(|item| item.submenu.as_deref() == active_submenu)
            .unwrap_or(0)
    };
    match key {
        Key::Named(NamedKey::Escape) if active_submenu.is_some() => {
            MenuKeyIntent::CloseSubmenu(parent_index())
        }
        Key::Named(NamedKey::Escape) => MenuKeyIntent::CloseMenu,
        Key::Named(NamedKey::ArrowLeft) if active_submenu.is_some() => {
            MenuKeyIntent::CloseSubmenu(parent_index())
        }
        Key::Named(NamedKey::ArrowDown) => MenuKeyIntent::Focus((focus_index + 1) % items.len()),
        Key::Named(NamedKey::ArrowUp) => {
            let current = focus_index.min(items.len() - 1);
            MenuKeyIntent::Focus(current.checked_sub(1).unwrap_or(items.len() - 1))
        }
        Key::Named(NamedKey::ArrowRight | NamedKey::Enter) => {
            let item = &items[focus_index.min(items.len() - 1)];
            if let Some(submenu) = &item.submenu {
                MenuKeyIntent::OpenSubmenu(submenu.clone())
            } else if matches!(key, Key::Named(NamedKey::Enter)) {
                MenuKeyIntent::Activate {
                    menu: menu_name.to_owned(),
                    label: item.label.clone(),
                }
            } else {
                MenuKeyIntent::Consume
            }
        }
        _ => MenuKeyIntent::Consume,
    }
}

impl Runtime {
    pub(super) fn activate_application_overlay_hit_target(
        &mut self,
        target: &HitTarget,
    ) -> Option<bool> {
        let handled = match target {
            HitTarget::MenuTitle(menu) => self.toggle_menu(menu),
            HitTarget::MenuItem { menu, label } => {
                self.activate_menu_item(menu, label, EntrySurface::MenuPointer)
            }
            HitTarget::GlobalPreferencesModal => true,
            HitTarget::GlobalPreferencesSection(section_id) => {
                self.session
                    .workspace_mut()
                    .ui
                    .global_preferences
                    .select_section(section_id);
                self.invalidate_frame();
                true
            }
            HitTarget::GlobalPreferencesSearch => {
                self.session.workspace_mut().ui.global_preferences.focus =
                    datum_gui_protocol::GlobalPreferencesFocus::Search;
                self.invalidate_frame();
                true
            }
            HitTarget::GlobalPreferencesSettingName(key) => {
                self.open_global_preference_search_result(key)
            }
            HitTarget::GlobalPreferencesControl(key) => {
                self.activate_global_preference_control(key)
            }
            HitTarget::GlobalPreferencesChoice { key, value } => {
                self.choose_global_preference_value(key, value)
            }
            HitTarget::GlobalPreferencesReset(key) => self.reset_global_preference(key),
            HitTarget::GlobalPreferencesExplanationClose => {
                let ui = &mut self.session.workspace_mut().ui.global_preferences;
                if let Some(key) = ui.explanation_key.take() {
                    ui.focus = datum_gui_protocol::GlobalPreferencesFocus::SettingName(key);
                }
                self.invalidate_frame();
                true
            }
            HitTarget::MarkingMenuItem { .. } => self.dismiss_marking_menu(),
            _ => return None,
        };
        Some(handled)
    }

    pub(super) fn handle_menu_key(&mut self, event: &KeyEvent) -> bool {
        let Some(active_menu) = self.workspace().ui.active_menu.clone() else {
            return false;
        };
        if event.state != ElementState::Pressed {
            return true;
        }
        let model = match datum_gui_protocol::gui_menu_model::default_gui_menu_model() {
            Ok(model) => model,
            Err(_) => return true,
        };
        let Some(group) = model.menubar.iter().find(|group| group.menu == active_menu) else {
            return true;
        };
        let active_submenu = self.workspace().ui.active_submenu.clone();
        match menu_key_intent(
            group,
            active_submenu.as_deref(),
            self.workspace().ui.menu_focus_index,
            &event.logical_key,
        ) {
            MenuKeyIntent::CloseMenu => {
                self.session.workspace_mut().ui.active_menu = None;
                let pane = self.workspace().ui.layout.focused;
                self.set_application_focus(ApplicationFocus::Editor(pane));
                self.invalidate_frame();
            }
            MenuKeyIntent::CloseSubmenu(parent_index) => {
                self.session.workspace_mut().ui.active_submenu = None;
                self.session.workspace_mut().ui.menu_focus_index = parent_index;
                self.invalidate_frame();
            }
            MenuKeyIntent::Focus(index) => {
                self.session.workspace_mut().ui.menu_focus_index = index;
                self.invalidate_frame();
            }
            MenuKeyIntent::OpenSubmenu(submenu) => {
                self.session.workspace_mut().ui.active_submenu = Some(submenu);
                self.session.workspace_mut().ui.menu_focus_index = 0;
                self.invalidate_frame();
            }
            MenuKeyIntent::Activate { menu, label } => {
                self.activate_menu_item(&menu, &label, EntrySurface::MenuKeyboard);
            }
            MenuKeyIntent::Consume => {}
        }
        true
    }

    pub(super) fn marking_menu_active(&self) -> bool {
        self.workspace().ui.marking_menu.is_some()
    }

    pub(super) fn update_marking_menu_preview(&mut self, pos: (f32, f32)) -> bool {
        let Some(menu) = self.session.workspace_mut().ui.marking_menu.as_mut() else {
            return false;
        };
        let dx = (pos.0 - menu.anchor_x_px as f32).round() as i32;
        let dy = (pos.1 - menu.anchor_y_px as f32).round() as i32;
        let next_slot = marking_slot_for_delta(dx, dy);
        if menu.gesture_dx_px == dx && menu.gesture_dy_px == dy && menu.preview_slot == next_slot {
            return false;
        }
        menu.gesture_dx_px = dx;
        menu.gesture_dy_px = dy;
        menu.preview_slot = next_slot;
        self.invalidate_frame();
        true
    }

    pub(super) fn dismiss_marking_menu(&mut self) -> bool {
        if self.session.workspace().ui.marking_menu.is_none() {
            return false;
        }
        self.session.workspace_mut().ui.marking_menu = None;
        // TF-01: the marking menu is a transient Overlay key owner; dismissing
        // it restores keyboard ownership to the editor.
        let pane = self.workspace().ui.layout.focused;
        self.set_application_focus(ApplicationFocus::Editor(pane));
        self.invalidate_frame();
        true
    }

    pub(super) fn toggle_menu(&mut self, menu: &str) -> bool {
        let ui = &mut self.session.workspace_mut().ui;
        ui.terminal_clipboard_menu = None;
        ui.active_submenu = None;
        ui.active_menu = if ui.active_menu.as_deref() == Some(menu) {
            None
        } else {
            Some(menu.to_string())
        };
        ui.menu_focus_index = 0;
        if ui.active_menu.is_some() {
            self.set_application_focus(ApplicationFocus::Overlay);
        } else {
            let pane = self.workspace().ui.layout.focused;
            self.set_application_focus(ApplicationFocus::Editor(pane));
        }
        self.invalidate_frame();
        true
    }

    pub(super) fn activate_menu_item(
        &mut self,
        menu_name: &str,
        label: &str,
        surface: EntrySurface,
    ) -> bool {
        let item = datum_gui_protocol::gui_menu_model::default_gui_menu_model()
            .ok()
            .and_then(|model| {
                model.menubar.iter().find_map(|menu| {
                    if menu.menu == menu_name {
                        return menu.items.iter().find(|item| item.label == label);
                    }
                    menu.submenus
                        .get(menu_name)
                        .and_then(|items| items.iter().find(|item| item.label == label))
                })
            });
        let Some(item) = item else {
            self.log_console_refusal(
                ConsoleFeedbackSource::Menu,
                format!("menu item {menu_name}/{label} unavailable"),
            );
            self.invalidate_frame();
            return true;
        };
        if let Some(submenu) = item.submenu.as_deref() {
            self.session.workspace_mut().ui.active_submenu = Some(submenu.to_owned());
            self.session.workspace_mut().ui.menu_focus_index = 0;
            self.set_application_focus(ApplicationFocus::Overlay);
            self.invalidate_frame();
            return true;
        }
        if let Some(reason) = item.unavailable_reason(self.workspace()) {
            if let Some(action) = item.gui_local.as_deref() {
                self.trace_action_attempt(action, surface, false);
            }
            self.session.workspace_mut().ui.active_menu = None;
            self.session.workspace_mut().ui.active_submenu = None;
            let pane = self.workspace().ui.layout.focused;
            self.set_application_focus(ApplicationFocus::Editor(pane));
            self.log_console_refusal(
                ConsoleFeedbackSource::Menu,
                format!("{menu_name} / {label}: {reason}"),
            );
            self.invalidate_frame();
            return true;
        }
        self.session.workspace_mut().ui.active_menu = None;
        self.session.workspace_mut().ui.active_submenu = None;
        let pane = self.workspace().ui.layout.focused;
        self.set_application_focus(ApplicationFocus::Editor(pane));
        if let Some(action) = item.gui_local.as_deref() {
            return self.activate_gui_local_menu_action(action, surface);
        }
        let reason = item
            .not_built
            .as_deref()
            .unwrap_or("not available in this build");
        let message = format!("{menu_name} / {label} is unavailable: {reason}");
        if let Some(action_id) = item.verb.as_deref() {
            self.log_console_refusal_for_action(ConsoleFeedbackSource::Menu, action_id, message);
        } else {
            self.log_console_refusal(ConsoleFeedbackSource::Menu, message);
        }
        self.invalidate_frame();
        true
    }

    pub(super) fn update_menu_hover(&mut self, pos: (f32, f32)) -> bool {
        if self.workspace().ui.active_menu.is_none() {
            return false;
        }
        let target = self.presented_hits.hit_test(pos.0, pos.1).cloned();
        let Some(HitTarget::MenuItem { menu, label }) = target else {
            return false;
        };
        let model = match datum_gui_protocol::gui_menu_model::default_gui_menu_model() {
            Ok(model) => model,
            Err(_) => return false,
        };
        let Some((index, item)) = model.menubar.iter().find_map(|group| {
            if group.menu == menu {
                group
                    .items
                    .iter()
                    .enumerate()
                    .find(|(_, item)| item.label == label)
            } else {
                group.submenus.get(&menu).and_then(|items| {
                    items
                        .iter()
                        .enumerate()
                        .find(|(_, item)| item.label == label)
                })
            }
        }) else {
            return false;
        };
        let next_submenu = item.submenu.clone().or_else(|| {
            (self.workspace().ui.active_submenu.as_deref() == Some(menu.as_str()))
                .then(|| menu.clone())
        });
        let ui = &mut self.session.workspace_mut().ui;
        let changed = ui.menu_focus_index != index || ui.active_submenu != next_submenu;
        ui.menu_focus_index = index;
        ui.active_submenu = next_submenu;
        if changed {
            self.invalidate_frame();
        }
        changed
    }
}

#[cfg(test)]
mod menu_keyboard_tests {
    use super::*;

    #[test]
    fn unavailable_pilot_rows_remain_keyboard_inspectable_and_dismissible() {
        let model = datum_gui_protocol::load_default_gui_menu_model().unwrap();
        for (menu_name, key) in [
            ("Help", "help.about"),
            ("Window", "window.documents"),
            ("View", "view.layers"),
        ] {
            let group = model
                .menubar
                .iter()
                .find(|group| group.menu == menu_name)
                .unwrap();
            let index = group
                .items
                .iter()
                .position(|item| item.gui_local.as_deref() == Some(key))
                .unwrap();
            assert!(!group.items[index].is_phase_one_enabled());
            let previous = (index + group.items.len() - 1) % group.items.len();
            assert_eq!(
                menu_key_intent(group, None, previous, &Key::Named(NamedKey::ArrowDown)),
                MenuKeyIntent::Focus(index),
            );
            // Enter reaches the production activation/refusal boundary; disabled
            // rows are not skipped and cannot acquire a synthetic handler.
            assert_eq!(
                menu_key_intent(group, None, index, &Key::Named(NamedKey::Enter)),
                MenuKeyIntent::Activate {
                    menu: menu_name.to_owned(),
                    label: group.items[index].label.clone()
                },
            );
            assert_eq!(
                menu_key_intent(group, None, index, &Key::Named(NamedKey::Escape)),
                MenuKeyIntent::CloseMenu,
            );
        }
    }

    fn edit_menu() -> datum_gui_protocol::GuiMenu {
        datum_gui_protocol::load_default_gui_menu_model()
            .unwrap()
            .menubar
            .into_iter()
            .find(|menu| menu.menu == "Edit")
            .unwrap()
    }

    #[test]
    fn right_and_enter_open_and_activate_the_real_preferences_submenu() {
        let edit = edit_menu();
        let parent = edit
            .items
            .iter()
            .position(|item| item.label == "Preferences")
            .unwrap();
        assert_eq!(
            menu_key_intent(&edit, None, parent, &Key::Named(NamedKey::ArrowRight)),
            MenuKeyIntent::OpenSubmenu("edit.preferences".to_owned())
        );
        assert_eq!(
            menu_key_intent(
                &edit,
                Some("edit.preferences"),
                0,
                &Key::Named(NamedKey::Enter),
            ),
            MenuKeyIntent::Activate {
                menu: "edit.preferences".to_owned(),
                label: "Global Preferences…".to_owned(),
            }
        );
    }

    #[test]
    fn left_and_escape_close_only_the_innermost_menu_level() {
        let edit = edit_menu();
        let parent = edit
            .items
            .iter()
            .position(|item| item.label == "Preferences")
            .unwrap();
        assert_eq!(
            menu_key_intent(
                &edit,
                Some("edit.preferences"),
                0,
                &Key::Named(NamedKey::ArrowLeft),
            ),
            MenuKeyIntent::CloseSubmenu(parent)
        );
        assert_eq!(
            menu_key_intent(&edit, None, parent, &Key::Named(NamedKey::Escape)),
            MenuKeyIntent::CloseMenu
        );
    }
}
