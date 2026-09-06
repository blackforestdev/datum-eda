//! Application coordinator for the one engine-owned Global Preferences service.
#[path = "global_preferences_product_adapter.rs"]
mod product_adapter;

use std::collections::BTreeMap;

use anyhow::Result;
use datum_gui_protocol::{
    ApplicationFocus, GlobalPreferenceControlUi, GlobalPreferenceRowUi,
    GlobalPreferencesDialogState, GlobalPreferencesDismissal, GlobalPreferencesFocus,
    GlobalPreferencesNoticeUi, WorkspaceUiState,
};
use eda_engine::ir::units::{ACTIVE_UNITS_KEYS, profile_from_descriptor_values};
use eda_engine::preferences::{
    GlobalPreferencesProductService, InstalledPreferenceLocationProvider, PreferenceErrorV1,
    PreferenceKey, PreferenceLiveConsumer, PreferenceMutationRequestV1, new_product_id,
};
use serde_json::Value;

use crate::Runtime;
use crate::console_accessibility::{AccessibilityAnnouncement, AnnouncementPriority};
use crate::global_preferences_projection::{
    apply_live_consumers, bool_consumer_value, control_projection, explanation_lines,
    provenance_label, repository_notice,
};
use product_adapter::{head_expectation, human_gui_actor};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};

pub(super) const SCOPE: &str = "Global · this device";
pub(super) struct GlobalPreferencesCoordinator {
    service: GlobalPreferencesProductService,
    return_focus: ApplicationFocus,
    terminal_theme_before_high_contrast: Option<datum_gui_protocol::TerminalTheme>,
}

impl GlobalPreferencesCoordinator {
    pub(super) fn from_platform() -> Result<Self> {
        let writer_instance = format!("datum-gui-{}", std::process::id());
        let service = GlobalPreferencesProductService::open(
            &InstalledPreferenceLocationProvider,
            &writer_instance,
        )
        .map_err(|error| anyhow::anyhow!("open Global Preferences service: {error:?}"))?;
        Ok(Self {
            service,
            return_focus: ApplicationFocus::default(),
            terminal_theme_before_high_contrast: None,
        })
    }

    pub(super) fn publish_projection(&mut self, ui: &mut WorkspaceUiState) {
        let was_open = ui.global_preferences.open;
        let query = ui.global_preferences.search_query.clone();
        let explanation_key = ui.global_preferences.explanation_key.clone();
        let open_choice_key = ui.global_preferences.open_choice_key.clone();
        let focus = ui.global_preferences.focus.clone();
        let rows = self.service.rows();
        let unit_values: BTreeMap<_, _> = rows
            .iter()
            .filter(|row| ACTIVE_UNITS_KEYS.contains(&row.key.as_str()))
            .filter_map(|row| {
                row.effective_value
                    .clone()
                    .map(|value| (row.key.as_str().to_owned(), value))
            })
            .collect();
        let resolved_units = profile_from_descriptor_values(&unit_values)
            .ok()
            .and_then(|profile| profile.resolve().ok());
        let mut projected = Vec::with_capacity(rows.len());
        for (surface, row) in self.service.surface().entries().iter().zip(&rows) {
            let descriptor = self
                .service
                .registry()
                .get(&row.key)
                .expect("surface construction proves descriptor existence");
            projected.push(GlobalPreferenceRowUi {
                key: row.key.as_str().to_owned(),
                section_id: surface.section.as_str().to_owned(),
                label: descriptor.presentation.label.clone(),
                description: descriptor.presentation.description.clone(),
                aliases: descriptor.retired_aliases.iter().cloned().collect(),
                scope: SCOPE.to_owned(),
                provenance: provenance_label(row, self.service.status()),
                explanation_lines: explanation_lines(row),
                control: control_projection(&surface.control, row, resolved_units.as_ref()),
                changed: row.user_value.is_some(),
                writable: row.writable,
                unavailable_reason: (!row.writable).then(|| {
                    "The preserved Preferences repository could not be read; this control cannot write"
                        .to_owned()
                }),
                reset_description: "Removes the User contribution and resolves again."
                    .to_owned(),
            });
        }
        let reduced_motion = bool_consumer_value(
            self.service.surface().entries(),
            &rows,
            PreferenceLiveConsumer::ReducedMotion,
        );
        let high_contrast_noncolor = bool_consumer_value(
            self.service.surface().entries(),
            &rows,
            PreferenceLiveConsumer::HighContrastNonColor,
        );
        let selected_section = self
            .service
            .surface()
            .sections()
            .iter()
            .find(|section| section.id.as_str() == ui.global_preferences.section_id)
            .unwrap_or(&self.service.surface().sections()[0]);
        ui.global_preferences = GlobalPreferencesDialogState {
            open: was_open,
            title: "Global Preferences — Datum".to_owned(),
            scope: SCOPE.to_owned(),
            sections: self
                .service
                .surface()
                .sections()
                .iter()
                .map(|section| (section.id.as_str().to_owned(), section.label.clone()))
                .collect(),
            section_id: selected_section.id.as_str().to_owned(),
            section_label: selected_section.label.clone(),
            search_query: query,
            scroll_row: ui.global_preferences.scroll_row,
            rows: projected,
            explanation_key,
            open_choice_key,
            focus,
            notice: repository_notice(self.service.status(), self.service.legacy_migration()),
            reduced_motion,
            high_contrast_noncolor,
        };
        apply_live_consumers(
            ui,
            self.service.surface().entries(),
            &rows,
            &mut self.terminal_theme_before_high_contrast,
        );
    }

    fn open_dialog(&mut self, ui: &mut WorkspaceUiState, invoker: ApplicationFocus) -> bool {
        if ui.global_preferences.open {
            return false;
        }
        self.return_focus = invoker;
        ui.active_menu = None;
        ui.active_submenu = None;
        ui.global_preferences.reset_transient_view();
        ui.global_preferences.notice =
            repository_notice(self.service.status(), self.service.legacy_migration());
        ui.global_preferences.open = true;
        ui.focus = ApplicationFocus::Overlay;
        true
    }

    fn close_dialog(&mut self, ui: &mut WorkspaceUiState) -> Option<ApplicationFocus> {
        if !ui.global_preferences.open {
            return None;
        }
        ui.global_preferences.open = false;
        ui.global_preferences.reset_transient_view();
        Some(self.return_focus)
    }

    fn set_value(
        &mut self,
        key: &str,
        value: Value,
        ui: &mut WorkspaceUiState,
    ) -> Result<(), PreferenceErrorV1> {
        let typed_key = PreferenceKey::parse(key).expect("UI key came from typed surface catalog");
        let request = PreferenceMutationRequestV1::SetUser {
            key: key.to_owned(),
            value,
            expected: head_expectation(self.service.status()),
            request_id: new_product_id(),
            reason: "Global Preferences control activation".to_owned(),
        };
        match self.service.mutate(request, &human_gui_actor()) {
            Ok(_) => {
                self.publish_projection(ui);
                ui.global_preferences.notice = Some(GlobalPreferencesNoticeUi::Polite(format!(
                    "{} changed for this device.",
                    self.service
                        .registry()
                        .get(&typed_key)
                        .unwrap()
                        .presentation
                        .label
                )));
                Ok(())
            }
            Err(refusal) => {
                self.publish_projection(ui);
                ui.global_preferences.notice = Some(GlobalPreferencesNoticeUi::Assertive(
                    refusal.message.clone(),
                ));
                Err(refusal)
            }
        }
    }

    fn reset(&mut self, key: &str, ui: &mut WorkspaceUiState) -> Result<(), PreferenceErrorV1> {
        let typed_key = PreferenceKey::parse(key).expect("UI key came from typed surface catalog");
        let request = PreferenceMutationRequestV1::ResetUser {
            key: key.to_owned(),
            expected: head_expectation(self.service.status()),
            request_id: new_product_id(),
            reason: "Global Preferences Reset activation".to_owned(),
        };
        match self.service.mutate(request, &human_gui_actor()) {
            Ok(_) => {
                self.publish_projection(ui);
                ui.global_preferences.notice = Some(GlobalPreferencesNoticeUi::Polite(format!(
                    "{} reset to its effective default.",
                    self.service
                        .registry()
                        .get(&typed_key)
                        .unwrap()
                        .presentation
                        .label
                )));
                Ok(())
            }
            Err(refusal) => {
                self.publish_projection(ui);
                ui.global_preferences.notice = Some(GlobalPreferencesNoticeUi::Assertive(
                    refusal.message.clone(),
                ));
                Err(refusal)
            }
        }
    }
}

impl Runtime {
    pub(super) fn open_global_preferences(&mut self) -> bool {
        if self.workspace().ui.project_preferences.open {
            self.close_project_preferences();
        }
        let invoker = self.application_focus();
        let opened = self
            .global_preferences
            .open_dialog(&mut self.session.workspace_mut().ui, invoker);
        self.set_application_focus(ApplicationFocus::Overlay);
        self.global_preferences_raise_requested = true;
        if opened {
            self.announce_global_preferences(
                "Global Preferences opened. Appearance and Units, eleven settings. Units are defaults for new Projects; open Projects are unaffected.",
                AnnouncementPriority::Medium,
            );
            self.announce_current_global_preferences_notice();
        }
        self.invalidate_frame();
        true
    }

    pub(super) fn take_global_preferences_raise_request(&mut self) -> bool {
        std::mem::take(&mut self.global_preferences_raise_requested)
    }

    pub(super) fn close_global_preferences(&mut self) -> bool {
        let Some(return_focus) = self
            .global_preferences
            .close_dialog(&mut self.session.workspace_mut().ui)
        else {
            return false;
        };
        self.set_application_focus(return_focus);
        self.invalidate_frame();
        true
    }

    pub(super) fn activate_global_preference_control(&mut self, key: &str) -> bool {
        let control = self
            .workspace()
            .ui
            .global_preferences
            .rows
            .iter()
            .find(|row| row.key == key)
            .map(|row| row.control.clone());
        let Some(control) = control else {
            return false;
        };
        match control {
            GlobalPreferenceControlUi::Boolean { value, .. } => {
                let _ = self.global_preferences.set_value(
                    key,
                    Value::Bool(!value),
                    &mut self.session.workspace_mut().ui,
                );
                self.announce_current_global_preferences_notice();
            }
            GlobalPreferenceControlUi::SingleChoice { .. } => {
                let ui = &mut self.session.workspace_mut().ui.global_preferences;
                ui.explanation_key = None;
                ui.open_choice_key = if ui.open_choice_key.as_deref() == Some(key) {
                    None
                } else {
                    Some(key.to_owned())
                };
                if ui.open_choice_key.is_some() {
                    ui.scroll_to_row(key);
                }
            }
            GlobalPreferenceControlUi::Integer {
                value,
                min,
                max,
                step,
                ..
            } => {
                let next = value.saturating_add(step as i64).min(max).max(min);
                let _ = self.global_preferences.set_value(
                    key,
                    Value::from(next),
                    &mut self.session.workspace_mut().ui,
                );
                self.announce_current_global_preferences_notice();
            }
            GlobalPreferenceControlUi::Identity { .. }
            | GlobalPreferenceControlUi::Structured { .. } => {
                self.explain_global_preference(key);
                return true;
            }
        }
        self.invalidate_frame();
        true
    }

    pub(super) fn choose_global_preference_value(&mut self, key: &str, value: &str) -> bool {
        let _ = self.global_preferences.set_value(
            key,
            Value::String(value.to_owned()),
            &mut self.session.workspace_mut().ui,
        );
        self.session
            .workspace_mut()
            .ui
            .global_preferences
            .open_choice_key = None;
        self.announce_current_global_preferences_notice();
        self.invalidate_frame();
        true
    }

    pub(super) fn reset_global_preference(&mut self, key: &str) -> bool {
        let _ = self
            .global_preferences
            .reset(key, &mut self.session.workspace_mut().ui);
        self.announce_current_global_preferences_notice();
        self.invalidate_frame();
        true
    }

    pub(super) fn explain_global_preference(&mut self, key: &str) -> bool {
        let ui = &mut self.session.workspace_mut().ui.global_preferences;
        ui.open_choice_key = None;
        ui.explanation_key = Some(key.to_owned());
        ui.focus = GlobalPreferencesFocus::ExplanationClose;
        ui.notice = Some(GlobalPreferencesNoticeUi::Polite(
            "Preference explanation opened.".to_owned(),
        ));
        self.announce_global_preferences(
            "Preference explanation opened.",
            AnnouncementPriority::Medium,
        );
        self.invalidate_frame();
        true
    }

    pub(super) fn open_global_preference_search_result(&mut self, key: &str) -> bool {
        let section = self
            .workspace()
            .ui
            .global_preferences
            .rows
            .iter()
            .find(|row| row.key == key)
            .map(|row| row.section_id.clone());
        if self
            .workspace()
            .ui
            .global_preferences
            .search_query
            .is_empty()
        {
            return self.explain_global_preference(key);
        }
        let Some(section) = section else {
            return false;
        };
        let dialog = &mut self.session.workspace_mut().ui.global_preferences;
        dialog.select_section(&section);
        dialog.scroll_to_row(key);
        self.explain_global_preference(key)
    }

    pub(super) fn handle_global_preferences_key(&mut self, event: &KeyEvent) -> bool {
        if !self.workspace().ui.global_preferences.open {
            return false;
        }
        if event.state != ElementState::Pressed {
            return true;
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
            return true;
        }
        if matches!(event.logical_key, Key::Named(NamedKey::Tab)) {
            self.session
                .workspace_mut()
                .ui
                .global_preferences
                .advance_focus(self.modifiers.shift_key());
            self.invalidate_frame();
            return true;
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
                    return true;
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
                    }
                    return true;
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
                    return true;
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
                self.cycle_global_preferences_section(-1)
            }
            GlobalPreferencesFocus::SectionNavigation
                if matches!(
                    event.logical_key,
                    Key::Named(NamedKey::ArrowDown | NamedKey::ArrowRight)
                ) =>
            {
                self.cycle_global_preferences_section(1)
            }
            GlobalPreferencesFocus::SettingName(key) if activate => {
                self.explain_global_preference(&key)
            }
            GlobalPreferencesFocus::Control(key) if activate => {
                self.activate_global_preference_control(&key)
            }
            GlobalPreferencesFocus::Control(key)
                if matches!(
                    event.logical_key,
                    Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowUp)
                ) =>
            {
                self.cycle_global_preference_control(&key, -1)
            }
            GlobalPreferencesFocus::Control(key)
                if matches!(
                    event.logical_key,
                    Key::Named(NamedKey::ArrowRight | NamedKey::ArrowDown)
                ) =>
            {
                self.cycle_global_preference_control(&key, 1)
            }
            GlobalPreferencesFocus::Control(key)
                if matches!(event.logical_key, Key::Named(NamedKey::Home)) =>
            {
                self.choose_global_preference_endpoint(&key, false)
            }
            GlobalPreferencesFocus::Control(key)
                if matches!(event.logical_key, Key::Named(NamedKey::End)) =>
            {
                self.choose_global_preference_endpoint(&key, true)
            }
            GlobalPreferencesFocus::Reset(key) if activate => self.reset_global_preference(&key),
            GlobalPreferencesFocus::ExplanationClose if activate => {
                let ui = &mut self.session.workspace_mut().ui.global_preferences;
                if let Some(key) = ui.explanation_key.take() {
                    ui.focus = GlobalPreferencesFocus::SettingName(key);
                }
                self.invalidate_frame();
                true
            }
            _ => true,
        }
    }

    fn cycle_global_preferences_section(&mut self, delta: isize) -> bool {
        let dialog = &mut self.session.workspace_mut().ui.global_preferences;
        let current = dialog
            .sections
            .iter()
            .position(|(id, _)| id == &dialog.section_id)
            .unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(dialog.sections.len() as isize) as usize;
        let section_id = dialog.sections[next].0.clone();
        dialog.select_section(&section_id);
        self.invalidate_frame();
        true
    }

    fn cycle_global_preference_control(&mut self, key: &str, delta: isize) -> bool {
        let next = self
            .workspace()
            .ui
            .global_preferences
            .rows
            .iter()
            .find(|row| row.key == key)
            .and_then(|row| match &row.control {
                GlobalPreferenceControlUi::Boolean { value, .. } => Some(Value::Bool(!value)),
                GlobalPreferenceControlUi::SingleChoice { value, choices } => {
                    let index = choices
                        .iter()
                        .position(|(candidate, _)| candidate == value)?
                        as isize;
                    let next = (index + delta).rem_euclid(choices.len() as isize) as usize;
                    Some(Value::String(choices[next].0.clone()))
                }
                GlobalPreferenceControlUi::Integer {
                    value,
                    min,
                    max,
                    step,
                    ..
                } => {
                    let step = *step as i64;
                    Some(Value::from(
                        (*value + delta.signum() as i64 * step).clamp(*min, *max),
                    ))
                }
                GlobalPreferenceControlUi::Identity { .. }
                | GlobalPreferenceControlUi::Structured { .. } => None,
            });
        self.commit_projected_value(key, next)
    }

    fn choose_global_preference_endpoint(&mut self, key: &str, last: bool) -> bool {
        let next = self
            .workspace()
            .ui
            .global_preferences
            .rows
            .iter()
            .find(|row| row.key == key)
            .and_then(|row| match &row.control {
                GlobalPreferenceControlUi::Boolean { .. } => Some(Value::Bool(last)),
                GlobalPreferenceControlUi::SingleChoice { choices, .. } => choices
                    .get(if last {
                        choices.len().saturating_sub(1)
                    } else {
                        0
                    })
                    .map(|(value, _)| Value::String(value.clone())),
                GlobalPreferenceControlUi::Integer { min, max, .. } => {
                    Some(Value::from(if last { *max } else { *min }))
                }
                GlobalPreferenceControlUi::Identity { .. }
                | GlobalPreferenceControlUi::Structured { .. } => None,
            });
        self.commit_projected_value(key, next)
    }

    fn commit_projected_value(&mut self, key: &str, next: Option<Value>) -> bool {
        let Some(next) = next else {
            return false;
        };
        let _ = self
            .global_preferences
            .set_value(key, next, &mut self.session.workspace_mut().ui);
        self.announce_current_global_preferences_notice();
        self.invalidate_frame();
        true
    }

    fn announce_global_preferences_search_count(&mut self) {
        let count = self
            .workspace()
            .ui
            .global_preferences
            .visible_rows()
            .count();
        self.announce_global_preferences(
            &format!("{count} preference search results."),
            AnnouncementPriority::Medium,
        );
    }

    fn announce_current_global_preferences_notice(&mut self) {
        let notice = self.workspace().ui.global_preferences.notice.clone();
        if let Some(notice) = notice {
            let (message, priority) = global_preferences_notice_announcement(notice);
            self.announce_global_preferences(&message, priority);
        }
    }

    fn announce_global_preferences(&mut self, message: &str, priority: AnnouncementPriority) {
        self.terminal_accessibility
            .announce_console(AccessibilityAnnouncement {
                text: message.to_owned(),
                priority,
            });
    }
}

fn global_preferences_notice_announcement(
    notice: GlobalPreferencesNoticeUi,
) -> (String, AnnouncementPriority) {
    match notice {
        GlobalPreferencesNoticeUi::Polite(message) => (message, AnnouncementPriority::Medium),
        GlobalPreferencesNoticeUi::Assertive(message)
        | GlobalPreferencesNoticeUi::PreservedUnreadable(message) => {
            (message, AnnouncementPriority::High)
        }
    }
}

#[cfg(test)]
#[path = "global_preferences_runtime_tests.rs"]
mod tests;
