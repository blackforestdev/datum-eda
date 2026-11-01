//! Keyboard actions for Project Preferences, using shared control projection.
use super::*;
use crate::global_preferences_projection::{control_endpoint_value, cycle_control_value};
use datum_gui_protocol::GlobalPreferencesDismissal;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};

impl Runtime {
    pub(crate) fn handle_project_preferences_key(
        &mut self,
        event: &KeyEvent,
    ) -> crate::global_preferences_window::DialogInputOutcome {
        use crate::global_preferences_window::DialogInputOutcome as Outcome;
        if !self.workspace().ui.project_preferences.open {
            return Outcome::Unhandled;
        }
        if event.state != ElementState::Pressed {
            return Outcome::Consumed;
        }
        if matches!(event.logical_key, Key::Named(NamedKey::Escape)) {
            let dismissal = self
                .session
                .workspace_mut()
                .ui
                .project_preferences
                .dismiss_innermost();
            if dismissal == GlobalPreferencesDismissal::DialogClosed {
                self.set_application_focus(self.project_preferences.return_focus);
            }
            if dismissal == GlobalPreferencesDismissal::DialogClosed {
                self.invalidate_frame();
            } else {
                self.refresh_dialog_state();
            }
            return if dismissal == GlobalPreferencesDismissal::DialogClosed {
                Outcome::Dependents
            } else {
                Outcome::Dialog
            };
        }
        if matches!(event.logical_key, Key::Named(NamedKey::Tab)) {
            self.session
                .workspace_mut()
                .ui
                .project_preferences
                .advance_focus(self.modifiers.shift_key());
            self.refresh_dialog_state();
            return Outcome::Dialog;
        }
        let focus = self.workspace().ui.project_preferences.focus.clone();
        if let GlobalPreferencesFocus::Control(key) = &focus {
            let next = self
                .workspace()
                .ui
                .project_preferences
                .rows
                .iter()
                .find(|row| row.key == *key && row.writable)
                .and_then(|row| match event.logical_key {
                    Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowUp) => {
                        cycle_control_value(&row.control, -1)
                    }
                    Key::Named(NamedKey::ArrowRight | NamedKey::ArrowDown) => {
                        cycle_control_value(&row.control, 1)
                    }
                    Key::Named(NamedKey::Home) => control_endpoint_value(&row.control, false),
                    Key::Named(NamedKey::End) => control_endpoint_value(&row.control, true),
                    _ => None,
                });
            if let Some(Value::String(value)) = next {
                return Outcome::from_handled(
                    self.choose_project_preference_value(key, &value),
                    Outcome::Dependents,
                );
            }
        }
        if focus == GlobalPreferencesFocus::Search {
            match &event.logical_key {
                Key::Named(NamedKey::Backspace) => {
                    let dialog = &mut self.session.workspace_mut().ui.project_preferences;
                    if dialog.search_query.pop().is_none() {
                        return Outcome::Consumed;
                    }
                    dialog.scroll_row = 0;
                    self.refresh_dialog_state();
                    return Outcome::Dialog;
                }
                key => {
                    if let Some(value) =
                        crate::global_preferences_window::dialog_text_input(key, self.modifiers)
                    {
                        let dialog = &mut self.session.workspace_mut().ui.project_preferences;
                        dialog.search_query.push_str(value);
                        dialog.scroll_row = 0;
                        self.refresh_dialog_state();
                        return Outcome::Dialog;
                    }
                }
            }
        }
        let activate = matches!(
            event.logical_key,
            Key::Named(NamedKey::Enter | NamedKey::Space)
        );
        match focus {
            GlobalPreferencesFocus::SettingName(key) if activate => Outcome::from_handled(
                self.activate_project_preferences_hit_target(
                    &HitTarget::GlobalPreferencesSettingName(key),
                )
                .unwrap_or(false),
                Outcome::Dialog,
            ),
            GlobalPreferencesFocus::Control(key) if activate => {
                let damage =
                    Outcome::control_activation(&self.workspace().ui.project_preferences, &key);
                Outcome::from_handled(self.activate_project_preference_control(&key), damage)
            }
            GlobalPreferencesFocus::Reset(key) if activate => {
                Outcome::from_handled(self.reset_project_preference(&key), Outcome::Dependents)
            }
            GlobalPreferencesFocus::ExplanationClose if activate => {
                let dialog = &mut self.session.workspace_mut().ui.project_preferences;
                dialog.close_explanation();
                self.refresh_dialog_state();
                Outcome::Dialog
            }
            _ => Outcome::Consumed,
        }
    }
}
