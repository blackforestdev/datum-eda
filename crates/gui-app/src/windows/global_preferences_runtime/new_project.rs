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
        self.service.refresh_global_project_units_source();
        let source = ProjectUnitsSourceV1::Global {
            expected_generation: self.service.status().generation().cloned(),
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
        if project_profile_from_value(&preview.profile)
            .ok()
            .and_then(|profile| profile.resolve().ok())
            .is_none()
        {
            self.refuse_preview(ui, "Global Units defaults did not resolve to a complete typed profile. Nothing was created. Resolve the reported Global Preferences problem, then re-read the defaults.".into());
            return;
        }
        ui.new_project.global_units_ready = true;
        ui.new_project.refusal = None;
        self.new_project_source = Some(ProjectUnitsSourceV1::Global {
            expected_generation: preview.pinned_generation,
        });
        if ui.new_project.focus == NewProjectFocus::RetryGlobal {
            ui.new_project.focus = if ui.new_project.create_enabled() {
                NewProjectFocus::Create
            } else {
                NewProjectFocus::ProjectName
            };
        }
    }

    fn refuse_preview(&mut self, ui: &mut WorkspaceUiState, message: String) {
        self.new_project_source = None;
        ui.new_project.global_units_ready = false;
        ui.new_project.refusal = Some(message);
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
        self.folder_browser.cancel();
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
        self.init_project_navigation(datum_gui_protocol::FolderBrowserMode::New, "");
        self.invalidate_frame();
        true
    }

    pub(crate) fn close_new_project(&mut self) -> bool {
        self.folder_browser.cancel();
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
            HitTarget::NewProjectFolderName => {
                let dialog = &mut self.session.workspace_mut().ui.new_project;
                dialog.focus = NewProjectFocus::FolderName;
            }
            HitTarget::NewProjectUseName => {
                let dialog = &mut self.session.workspace_mut().ui.new_project;
                dialog.focus = NewProjectFocus::UseProjectName;
                dialog.set_use_project_name(!dialog.use_project_name);
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
        self.refresh_folder_browser();
        true
    }

    fn submit_new_project(&mut self) -> bool {
        self.refresh_folder_browser();
        if self.folder_host_actions_pending() {
            self.session.workspace_mut().ui.new_project.refusal = Some(
                "Wait for the submitted folder action to finish before creating a Project".into(),
            );
            self.refresh_folder_browser();
            return true;
        }
        if self
            .folder_browser
            .state
            .as_ref()
            .is_none_or(|s| s.mode != datum_gui_protocol::FolderBrowserMode::New)
        {
            return true;
        }
        if !self.workspace().ui.new_project.create_enabled() {
            return true;
        }
        match self
            .global_preferences
            .create_new_project(&mut self.session.workspace_mut().ui)
        {
            Ok(created) => {
                self.log_console_echo(
                    ConsoleFeedbackSource::Viewport,
                    format!(
                        "Project created: {}. Opening Project.",
                        created.project_root_identity
                    ),
                );
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
        if self.handle_folder_actions_key(event)
            || self.handle_shared_editor_key(event)
            || self.handle_folder_browser_key(event)
        {
            return Outcome::Dialog;
        }
        if matches!(event.logical_key, Key::Named(NamedKey::Escape)) {
            self.close_new_project();
            return Outcome::Dependents;
        }
        if matches!(event.logical_key, Key::Named(NamedKey::Tab)) {
            let expanded = self
                .folder_browser
                .state
                .as_ref()
                .is_some_and(|s| s.expanded);
            self.session
                .workspace_mut()
                .ui
                .new_project
                .move_focus(self.modifiers.shift_key(), expanded);
            self.refresh_folder_browser();
            return Outcome::Dialog;
        }
        let focus = self.workspace().ui.new_project.focus;
        if focus == NewProjectFocus::FolderName && self.workspace().ui.new_project.use_project_name
        {
            return Outcome::Consumed;
        }
        if matches!(
            event.logical_key,
            Key::Named(NamedKey::Enter | NamedKey::Space)
        ) {
            return match focus {
                NewProjectFocus::RetryGlobal => {
                    self.global_preferences
                        .refresh_new_project_preview(&mut self.session.workspace_mut().ui);
                    self.refresh_folder_browser();
                    Outcome::Dialog
                }
                NewProjectFocus::UseProjectName => {
                    let dialog = &mut self.session.workspace_mut().ui.new_project;
                    dialog.set_use_project_name(!dialog.use_project_name);
                    self.refresh_folder_browser();
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

fn refusal_message(error: &PreferenceErrorV1) -> String {
    use eda_engine::preferences::PreferenceErrorCodeV1;
    match error.code {
        PreferenceErrorCodeV1::StaleGeneration => format!(
            "Your Global Units changed while this dialog was open. Nothing was created. {}",
            error.message
        ),
        PreferenceErrorCodeV1::SeedSourceUnavailable => format!(
            "Your Global Units could not be resolved. Nothing was created. {}",
            error.message.replace(
                "re-read the source or explicitly choose Factory",
                "resolve the reported Global Preferences problem, then re-read the defaults"
            )
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
