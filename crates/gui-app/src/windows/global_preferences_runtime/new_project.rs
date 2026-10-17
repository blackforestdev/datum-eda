//! Native New Project controller over the same engine-owned Preferences product service.

use super::*;
use datum_gui_render::HitTarget;

impl GlobalPreferencesCoordinator {
    pub(super) fn open_new_project(
        &mut self,
        ui: &mut WorkspaceUiState,
        invoker: ApplicationFocus,
    ) {
        self.return_focus = invoker;
        ui.active_menu = None;
        ui.active_submenu = None;
        ui.new_project.reset_for_open();
        ui.focus = ApplicationFocus::Overlay;
        self.refresh_new_project_preview(ui);
    }

    pub(super) fn close_new_project(
        &mut self,
        ui: &mut WorkspaceUiState,
    ) -> Option<ApplicationFocus> {
        if !ui.new_project.open {
            return None;
        }
        ui.new_project = NewProjectDialogState::default();
        self.new_project_source = None;
        Some(self.return_focus)
    }

    pub(super) fn refresh_new_project_preview(&mut self, ui: &mut WorkspaceUiState) {
        if ui.new_project.units_choice == NewProjectUnitsChoice::Global {
            self.service.refresh_global_project_units_source();
        }
        let source = match ui.new_project.units_choice {
            NewProjectUnitsChoice::Global => ProjectUnitsSourceV1::Global {
                expected_generation: self.service.status().generation().cloned(),
            },
            NewProjectUnitsChoice::Factory => ProjectUnitsSourceV1::Factory {
                profile_id: "datum.units.factory.v1".to_owned(),
            },
        };
        let result = self
            .service
            .query(PreferenceQueryV1::PreviewProjectUnitsSeed {
                source: source.clone(),
            });
        let preview = match result {
            Ok(PreferenceQueryResultV1::PreviewProjectUnitsSeed(preview)) => preview,
            Ok(_) => {
                self.refuse_preview(
                    ui,
                    "The Units preview returned an unexpected response. Nothing was created."
                        .to_owned(),
                );
                return;
            }
            Err(error) => {
                self.refuse_preview(ui, refusal_message(&error));
                return;
            }
        };
        let rows = match self.project_units_summary(&preview.profile) {
            Ok(rows) => rows,
            Err(message) => {
                self.refuse_preview(ui, message);
                return;
            }
        };
        ui.new_project.units_summary = rows;
        ui.new_project.refusal = None;
        match &source {
            ProjectUnitsSourceV1::Global { .. } => {
                ui.new_project.source_summary = "Source · your Global Units defaults".to_owned();
                ui.new_project.source_detail = preview
                    .pinned_generation
                    .as_ref()
                    .map(|generation| {
                        format!(
                            "preferences · generation {} · manifest {}",
                            generation.generation,
                            abbreviate_digest(&generation.canonical_manifest_digest)
                        )
                    })
                    .unwrap_or_else(|| "factory defaults; no preference file yet".to_owned());
            }
            ProjectUnitsSourceV1::Factory { profile_id } => {
                ui.new_project.source_summary = "Source · Datum factory Units".to_owned();
                ui.new_project.source_detail =
                    format!("{profile_id} · version 1 · no Global repository read");
            }
        }
        self.new_project_source = Some(match source {
            ProjectUnitsSourceV1::Global { .. } => ProjectUnitsSourceV1::Global {
                expected_generation: preview.pinned_generation,
            },
            factory => factory,
        });
    }

    fn refuse_preview(&mut self, ui: &mut WorkspaceUiState, message: String) {
        self.new_project_source = None;
        ui.new_project.units_summary.clear();
        ui.new_project.source_summary = "Source · unavailable".to_owned();
        ui.new_project.source_detail = match ui.new_project.units_choice {
            NewProjectUnitsChoice::Global => {
                "Choose Datum factory Units explicitly, or resolve the Global source and retry."
            }
            NewProjectUnitsChoice::Factory => {
                "The selected factory Units profile could not be resolved."
            }
        }
        .to_owned();
        ui.new_project.refusal = Some(message);
    }

    fn project_units_summary(
        &self,
        profile: &Value,
    ) -> Result<Vec<NewProjectUnitsSummaryRow>, String> {
        let typed = project_profile_from_value(profile)
            .map_err(|_| "The Units preview was not a valid typed profile.".to_owned())?;
        let values = profile_to_descriptor_values(typed);
        let resolved = typed
            .resolve()
            .map_err(|_| "The Units preview could not be resolved.".to_owned())?;
        ACTIVE_UNITS_KEYS
            .iter()
            .map(|key| {
                let entry = self
                    .service
                    .surface()
                    .entries()
                    .iter()
                    .find(|entry| entry.key.as_str() == *key)
                    .ok_or_else(|| "An active Units control is missing.".to_owned())?;
                let descriptor = self
                    .service
                    .registry()
                    .get(&entry.key)
                    .ok_or_else(|| "An active Units descriptor is missing.".to_owned())?;
                let value = values
                    .get(*key)
                    .ok_or_else(|| "An active Units value is missing.".to_owned())?;
                let visible_value = match control_value_projection(
                    &entry.control,
                    Some(value),
                    key,
                    Some(&resolved),
                ) {
                    GlobalPreferenceControlUi::SingleChoice { value, choices } => choices
                        .into_iter()
                        .find(|(candidate, _)| candidate == &value)
                        .map(|(_, label)| label)
                        .unwrap_or(value),
                    _ => value.to_string(),
                };
                Ok(NewProjectUnitsSummaryRow {
                    key: (*key).to_owned(),
                    label: descriptor.presentation.label.clone(),
                    value: visible_value,
                })
            })
            .collect()
    }

    pub(super) fn create_new_project(
        &mut self,
        ui: &mut WorkspaceUiState,
    ) -> Result<eda_engine::preferences::ProjectGenesisResultV1, PreferenceErrorV1> {
        let Some(source) = self.new_project_source.clone() else {
            self.refresh_new_project_preview(ui);
            return Err(PreferenceErrorV1 {
                code: eda_engine::preferences::PreferenceErrorCodeV1::SeedSourceUnavailable,
                message: "Working Units are not resolved. Nothing was created.".to_owned(),
                details: Box::default(),
                current_context: Box::new(self.service.context()),
                preserved_draft: None,
                preserved_proposal: None,
            });
        };
        let destination = destination_from_input(
            &ui.new_project.destination,
            std::env::var_os("HOME").as_deref(),
        )
        .map_err(|message| {
            ui.new_project.refusal = Some(format!("Location could not be resolved. {message}"));
            PreferenceErrorV1 {
                code: eda_engine::preferences::PreferenceErrorCodeV1::InvalidRequest,
                message,
                details: Box::new(BTreeMap::from([(
                    "field".into(),
                    Value::String("destination".into()),
                )])),
                current_context: Box::new(self.service.context()),
                preserved_draft: None,
                preserved_proposal: None,
            }
        })?;
        let request = ProjectGenesisRequestV1 {
            request_id: new_product_id(),
            destination: destination.clone(),
            project_name: ui.new_project.project_name.trim().to_owned(),
            project_id: None,
            units_source: source,
        };
        match self.service.create_project(request, &human_gui_actor()) {
            Ok(result) => Ok(result),
            Err(error) => {
                if units_source_failure(&error.code) {
                    self.refuse_preview(ui, refusal_message(&error));
                } else {
                    let message = refusal_message(&error);
                    ui.new_project.refusal = Some(
                        if error.code
                            == eda_engine::preferences::PreferenceErrorCodeV1::RepositoryIo
                            && destination.exists()
                        {
                            format!("Location exists. Use a new folder. {message}")
                        } else {
                            message
                        },
                    );
                }
                Err(error)
            }
        }
    }
}

impl Runtime {
    pub(crate) fn open_new_project(&mut self) -> bool {
        if self.workspace().ui.project_open.open {
            self.close_project_open();
        }
        if self.workspace().ui.global_preferences.open {
            self.close_global_preferences();
        }
        if self.workspace().ui.project_preferences.open {
            self.close_project_preferences();
        }
        let invoker = self.application_focus();
        self.global_preferences
            .open_new_project(&mut self.session.workspace_mut().ui, invoker);
        self.set_application_focus(ApplicationFocus::Overlay);
        self.invalidate_frame();
        true
    }

    pub(crate) fn close_new_project(&mut self) -> bool {
        let Some(return_focus) = self
            .global_preferences
            .close_new_project(&mut self.session.workspace_mut().ui)
        else {
            return false;
        };
        self.set_application_focus(return_focus);
        self.invalidate_frame();
        true
    }

    pub(crate) fn activate_new_project_hit(&mut self, target: &HitTarget) -> bool {
        match target {
            HitTarget::NewProjectName => {
                self.session.workspace_mut().ui.new_project.focus = NewProjectFocus::ProjectName
            }
            HitTarget::NewProjectDestination => {
                self.session.workspace_mut().ui.new_project.focus = NewProjectFocus::Destination
            }
            HitTarget::NewProjectUnitsChoice(choice) => {
                let dialog = &mut self.session.workspace_mut().ui.new_project;
                dialog.units_choice = *choice;
                dialog.focus = NewProjectFocus::UnitsChoice;
                self.global_preferences
                    .refresh_new_project_preview(&mut self.session.workspace_mut().ui);
            }
            HitTarget::NewProjectUnitsSummary => {
                self.session.workspace_mut().ui.new_project.focus = NewProjectFocus::UnitsSummary
            }
            HitTarget::NewProjectRetryGlobal => {
                self.session.workspace_mut().ui.new_project.focus = NewProjectFocus::RetryGlobal;
                self.global_preferences
                    .refresh_new_project_preview(&mut self.session.workspace_mut().ui);
            }
            HitTarget::NewProjectCancel => return self.close_new_project(),
            HitTarget::NewProjectCreate => return self.submit_new_project(),
            HitTarget::NewProjectModal => return true,
            _ => return false,
        }
        self.refresh_dialog_state();
        true
    }

    fn submit_new_project(&mut self) -> bool {
        if !self.workspace().ui.new_project.create_enabled() {
            return true;
        }
        match self
            .global_preferences
            .create_new_project(&mut self.session.workspace_mut().ui)
        {
            Ok(created) => {
                self.log_console_echo(ConsoleFeedbackSource::Viewport, format!(
                    "Project created with eight Working Units and immutable receipt: {}. Opening Project.",
                    created.project_root_identity));
                self.close_new_project();
                self.open_created_project(std::path::PathBuf::from(created.project_root_identity));
            }
            Err(_) => self.invalidate_frame(),
        }
        true
    }

    pub(crate) fn handle_new_project_key(
        &mut self,
        event: &KeyEvent,
    ) -> crate::global_preferences_window::DialogInputOutcome {
        use crate::global_preferences_window::DialogInputOutcome as Outcome;
        if !self.workspace().ui.new_project.open {
            return Outcome::Unhandled;
        }
        if event.state != ElementState::Pressed {
            return Outcome::Consumed;
        }
        if matches!(event.logical_key, Key::Named(NamedKey::Escape)) {
            self.close_new_project();
            return Outcome::Dependents;
        }
        if matches!(event.logical_key, Key::Named(NamedKey::Tab)) {
            self.session
                .workspace_mut()
                .ui
                .new_project
                .move_focus(self.modifiers.shift_key());
            self.refresh_dialog_state();
            return Outcome::Dialog;
        }
        let focus = self.workspace().ui.new_project.focus;
        if matches!(
            focus,
            NewProjectFocus::ProjectName | NewProjectFocus::Destination
        ) {
            match &event.logical_key {
                Key::Named(NamedKey::Backspace) => {
                    let dialog = &mut self.session.workspace_mut().ui.new_project;
                    let removed = if focus == NewProjectFocus::ProjectName {
                        dialog.project_name.pop()
                    } else {
                        dialog.destination.pop()
                    };
                    if removed.is_none() {
                        return Outcome::Consumed;
                    }
                    dialog.clear_creation_refusal();
                    self.refresh_dialog_state();
                    return Outcome::Dialog;
                }
                key => {
                    if let Some(value) =
                        crate::global_preferences_window::dialog_text_input(key, self.modifiers)
                    {
                        let dialog = &mut self.session.workspace_mut().ui.new_project;
                        if focus == NewProjectFocus::ProjectName {
                            dialog.project_name.push_str(value);
                        } else {
                            dialog.destination.push_str(value);
                        }
                        dialog.clear_creation_refusal();
                        self.refresh_dialog_state();
                        return Outcome::Dialog;
                    }
                }
            }
        }
        if focus == NewProjectFocus::UnitsChoice
            && matches!(
                event.logical_key,
                Key::Named(
                    NamedKey::ArrowUp
                        | NamedKey::ArrowDown
                        | NamedKey::ArrowLeft
                        | NamedKey::ArrowRight
                        | NamedKey::Space
                )
            )
        {
            let dialog = &mut self.session.workspace_mut().ui.new_project;
            dialog.units_choice = match dialog.units_choice {
                NewProjectUnitsChoice::Global => NewProjectUnitsChoice::Factory,
                NewProjectUnitsChoice::Factory => NewProjectUnitsChoice::Global,
            };
            self.global_preferences
                .refresh_new_project_preview(&mut self.session.workspace_mut().ui);
            self.refresh_dialog_state();
            return Outcome::Dialog;
        }
        if matches!(
            event.logical_key,
            Key::Named(NamedKey::Enter | NamedKey::Space)
        ) {
            return match focus {
                NewProjectFocus::RetryGlobal => {
                    self.global_preferences
                        .refresh_new_project_preview(&mut self.session.workspace_mut().ui);
                    self.refresh_dialog_state();
                    Outcome::Dialog
                }
                NewProjectFocus::Cancel => {
                    Outcome::from_handled(self.close_new_project(), Outcome::Dependents)
                }
                NewProjectFocus::Create => {
                    Outcome::from_handled(self.submit_new_project(), Outcome::Dependents)
                }
                _ => Outcome::Consumed,
            };
        }
        Outcome::Consumed
    }
}

fn abbreviate_digest(value: &str) -> String {
    value.chars().take(12).collect::<String>() + "…"
}

fn refusal_message(error: &PreferenceErrorV1) -> String {
    use eda_engine::preferences::PreferenceErrorCodeV1;
    match error.code {
        PreferenceErrorCodeV1::StaleGeneration => format!(
            "Your Global Units changed while this dialog was open. Nothing was created. {}",
            error.message
        ),
        PreferenceErrorCodeV1::SeedSourceUnavailable => format!(
            "Your Global Units could not be resolved. Nothing was created. {}",
            error.message
        ),
        _ => match error.details.get("reason").and_then(Value::as_str) {
            Some(reason) => format!("Nothing was created. {}: {reason}", error.message),
            None => format!("Nothing was created. {}", error.message),
        },
    }
}

fn units_source_failure(code: &eda_engine::preferences::PreferenceErrorCodeV1) -> bool {
    use eda_engine::preferences::PreferenceErrorCodeV1::*;
    matches!(
        code,
        StaleGeneration
            | SeedSourceUnavailable
            | SeedIncomplete
            | RepositoryUnreadable
            | MigrationRequired
    )
}

/// Interpret human home shorthand only; the engine retains full destination authority.
fn destination_from_input(
    input: &str,
    home: Option<&std::ffi::OsStr>,
) -> Result<std::path::PathBuf, String> {
    let input = input.trim();
    let suffix = if input == "~" {
        Some("")
    } else {
        input.strip_prefix("~/")
    };
    if let Some(suffix) = suffix {
        let home = home
            .filter(|home| std::path::Path::new(home).is_absolute())
            .ok_or(
                "Home directory is unavailable or not absolute; enter a full destination path",
            )?;
        return Ok(std::path::Path::new(home).join(suffix.trim_start_matches('/')));
    }
    Ok(std::path::PathBuf::from(input))
}

#[cfg(test)]
#[path = "new_project/tests.rs"]
mod tests;
