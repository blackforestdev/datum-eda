//! Menubar and marking-menu interaction ownership, extracted from the runtime
//! monolith under decision 022. Menu dispatch remains session-local; mutation
//! continues through the established verb or GUI-local action boundaries.

use super::*;

impl Runtime {
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
        ui.active_menu = if ui.active_menu.as_deref() == Some(menu) {
            None
        } else {
            Some(menu.to_string())
        };
        self.invalidate_frame();
        true
    }

    pub(super) fn activate_menu_item(&mut self, menu_name: &str, label: &str) -> bool {
        let item = datum_gui_protocol::load_default_gui_menu_model()
            .ok()
            .and_then(|model| {
                model
                    .menubar
                    .into_iter()
                    .find(|menu| menu.menu == menu_name)
                    .and_then(|menu| menu.items.into_iter().find(|item| item.label == label))
            });
        self.session.workspace_mut().ui.active_menu = None;
        let Some(item) = item else {
            self.log_console_refusal(
                ConsoleFeedbackSource::Menu,
                format!("menu item {menu_name}/{label} unavailable"),
            );
            self.invalidate_frame();
            return true;
        };
        if let Some(action) = item.gui_local.as_deref() {
            return self.activate_gui_local_menu_action(action);
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
}
