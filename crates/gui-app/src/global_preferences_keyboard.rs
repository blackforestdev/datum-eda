//! Global Preferences keyboard consumption and damage outcomes.
use super::*;

impl Runtime {
    pub(crate) fn handle_global_preferences_key(
        &mut self,
        event: &KeyEvent,
    ) -> crate::global_preferences_window::DialogInputOutcome {
        use crate::global_preferences_window::DialogInputOutcome as Outcome;
        if !self.workspace().ui.global_preferences.open {
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
                .global_preferences
                .dismiss_innermost();
            match dismissal {
                GlobalPreferencesDismissal::SearchCleared => {
                    self.announce_global_preferences(
                        "Preference search cleared.",
                        AnnouncementPriority::Medium,
                    );
                }
                GlobalPreferencesDismissal::DialogClosed => {
                    self.set_application_focus(self.global_preferences.return_focus);
                }
                GlobalPreferencesDismissal::ChoiceClosed
                | GlobalPreferencesDismissal::ExplanationClosed => {}
            }
            self.invalidate_frame();
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
                .global_preferences
                .advance_focus(self.modifiers.shift_key());
            self.invalidate_frame();
            return Outcome::Dialog;
        }

        let focus = self.workspace().ui.global_preferences.focus.clone();
        if focus == GlobalPreferencesFocus::Search {
            match &event.logical_key {
                Key::Named(NamedKey::Backspace) => {
                    let dialog = &mut self.session.workspace_mut().ui.global_preferences;
                    dialog.search_query.pop();
                    dialog.scroll_row = 0;
                    self.invalidate_frame();
                    self.announce_global_preferences_search_count();
                    return Outcome::Dialog;
                }
                Key::Named(NamedKey::Enter) => {
                    let key = {
                        self.workspace()
                            .ui
                            .global_preferences
                            .visible_rows()
                            .next()
                            .map(|row| row.key.clone())
                    };
                    if let Some(key) = key {
                        self.open_global_preference_search_result(&key);
                        return Outcome::Dialog;
                    }
                    return Outcome::Consumed;
                }
                Key::Character(value)
                    if !self.modifiers.control_key()
                        && !self.modifiers.alt_key()
                        && !value.chars().any(char::is_control) =>
                {
                    let dialog = &mut self.session.workspace_mut().ui.global_preferences;
                    dialog.search_query.push_str(value);
                    dialog.scroll_row = 0;
                    self.invalidate_frame();
                    self.announce_global_preferences_search_count();
                    return Outcome::Dialog;
                }
                _ => {}
            }
        }

        let activate = matches!(
            event.logical_key,
            Key::Named(NamedKey::Enter | NamedKey::Space)
        );
        match focus {
            GlobalPreferencesFocus::SectionNavigation
                if matches!(
                    event.logical_key,
                    Key::Named(NamedKey::ArrowUp | NamedKey::ArrowLeft)
                ) =>
            {
                Outcome::from_handled(self.cycle_global_preferences_section(-1), Outcome::Dialog)
            }
            GlobalPreferencesFocus::SectionNavigation
                if matches!(
                    event.logical_key,
                    Key::Named(NamedKey::ArrowDown | NamedKey::ArrowRight)
                ) =>
            {
                Outcome::from_handled(self.cycle_global_preferences_section(1), Outcome::Dialog)
            }
            GlobalPreferencesFocus::SettingName(key) if activate => {
                Outcome::from_handled(self.explain_global_preference(&key), Outcome::Dialog)
            }
            GlobalPreferencesFocus::Control(key) if activate => Outcome::from_handled(
                self.activate_global_preference_control(&key),
                Outcome::Dependents,
            ),
            GlobalPreferencesFocus::Control(key)
                if matches!(
                    event.logical_key,
                    Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowUp)
                ) =>
            {
                Outcome::from_handled(
                    self.cycle_global_preference_control(&key, -1),
                    Outcome::Dependents,
                )
            }
            GlobalPreferencesFocus::Control(key)
                if matches!(
                    event.logical_key,
                    Key::Named(NamedKey::ArrowRight | NamedKey::ArrowDown)
                ) =>
            {
                Outcome::from_handled(
                    self.cycle_global_preference_control(&key, 1),
                    Outcome::Dependents,
                )
            }
            GlobalPreferencesFocus::Control(key)
                if matches!(event.logical_key, Key::Named(NamedKey::Home)) =>
            {
                Outcome::from_handled(
                    self.choose_global_preference_endpoint(&key, false),
                    Outcome::Dependents,
                )
            }
            GlobalPreferencesFocus::Control(key)
                if matches!(event.logical_key, Key::Named(NamedKey::End)) =>
            {
                Outcome::from_handled(
                    self.choose_global_preference_endpoint(&key, true),
                    Outcome::Dependents,
                )
            }
            GlobalPreferencesFocus::Reset(key) if activate => {
                Outcome::from_handled(self.reset_global_preference(&key), Outcome::Dependents)
            }
            GlobalPreferencesFocus::ExplanationClose if activate => {
                let ui = &mut self.session.workspace_mut().ui.global_preferences;
                if let Some(key) = ui.explanation_key.take() {
                    ui.focus = GlobalPreferencesFocus::SettingName(key);
                }
                self.invalidate_frame();
                Outcome::Dialog
            }
            _ => Outcome::Consumed,
        }
    }
}
