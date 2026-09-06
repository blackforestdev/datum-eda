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
        ui.new_project.source_detail =
            "Choose Datum factory Units explicitly, or resolve the Global source and retry."
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
    ) -> Result<String, PreferenceErrorV1> {
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
        let request = ProjectGenesisRequestV1 {
            request_id: new_product_id(),
            destination: std::path::PathBuf::from(ui.new_project.destination.trim()),
            project_name: ui.new_project.project_name.trim().to_owned(),
            project_id: None,
            units_source: source,
        };
        match self.service.create_project(request, &human_gui_actor()) {
            Ok(result) => Ok(format!(
                "Project created with eight Working Units and immutable receipt: {}",
                result.project_root_identity
            )),
            Err(error) => {
                self.new_project_source = None;
                ui.new_project.units_summary.clear();
                ui.new_project.refusal = Some(refusal_message(&error));
                Err(error)
            }
        }
    }
}

impl Runtime {
    pub(crate) fn open_new_project(&mut self) -> bool {
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
            HitTarget::NewProjectModal => {}
            _ => return false,
        }
        self.invalidate_frame();
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
            Ok(message) => {
                self.log_console_echo(ConsoleFeedbackSource::Viewport, message);
                self.close_new_project();
            }
            Err(_) => self.invalidate_frame(),
        }
        true
    }

    pub(crate) fn handle_new_project_key(&mut self, event: &KeyEvent) -> bool {
        if !self.workspace().ui.new_project.open {
            return false;
        }
        if event.state != ElementState::Pressed {
            return true;
        }
        if matches!(event.logical_key, Key::Named(NamedKey::Escape)) {
            self.close_new_project();
            return true;
        }
        if matches!(event.logical_key, Key::Named(NamedKey::Tab)) {
            self.session
                .workspace_mut()
                .ui
                .new_project
                .move_focus(self.modifiers.shift_key());
            self.invalidate_frame();
            return true;
        }
        let focus = self.workspace().ui.new_project.focus;
        if matches!(
            focus,
            NewProjectFocus::ProjectName | NewProjectFocus::Destination
        ) {
            match &event.logical_key {
                Key::Named(NamedKey::Backspace) => {
                    let dialog = &mut self.session.workspace_mut().ui.new_project;
                    if focus == NewProjectFocus::ProjectName {
                        dialog.project_name.pop();
                    } else {
                        dialog.destination.pop();
                    }
                    self.invalidate_frame();
                    return true;
                }
                Key::Character(value)
                    if !self.modifiers.control_key()
                        && !self.modifiers.alt_key()
                        && !value.chars().any(char::is_control) =>
                {
                    let dialog = &mut self.session.workspace_mut().ui.new_project;
                    if focus == NewProjectFocus::ProjectName {
                        dialog.project_name.push_str(value);
                    } else {
                        dialog.destination.push_str(value);
                    }
                    self.invalidate_frame();
                    return true;
                }
                _ => {}
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
            self.invalidate_frame();
            return true;
        }
        if matches!(
            event.logical_key,
            Key::Named(NamedKey::Enter | NamedKey::Space)
        ) {
            return match focus {
                NewProjectFocus::RetryGlobal => {
                    self.global_preferences
                        .refresh_new_project_preview(&mut self.session.workspace_mut().ui);
                    self.invalidate_frame();
                    true
                }
                NewProjectFocus::Cancel => self.close_new_project(),
                NewProjectFocus::Create => self.submit_new_project(),
                _ => true,
            };
        }
        true
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
        _ => format!("Nothing was created. {}", error.message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eda_engine::preferences::{FixedPreferenceLocationProvider, PreferenceLocations};

    fn fixture() -> (
        std::path::PathBuf,
        GlobalPreferencesCoordinator,
        WorkspaceUiState,
    ) {
        let root = std::env::temp_dir().join(format!(
            "datum-new-project-gui-{}-{}",
            std::process::id(),
            new_product_id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let service = GlobalPreferencesProductService::open(
            &FixedPreferenceLocationProvider(PreferenceLocations {
                configuration_base: root.clone(),
                repository_root: root.join("preferences"),
                legacy_console_path: root.join("legacy.json"),
            }),
            "new-project-gui-test",
        )
        .unwrap();
        let coordinator = GlobalPreferencesCoordinator {
            service,
            return_focus: ApplicationFocus::default(),
            new_project_source: None,
            terminal_theme_before_high_contrast: None,
        };
        let filters = datum_gui_protocol::WorkspaceFilterState {
            show_authored: true,
            show_proposed: true,
            show_unrouted: true,
            dim_unrelated: false,
            active_layer_id: None,
            layer_visibility: Default::default(),
            layer_scroll_offset: 0,
        };
        (root, coordinator, WorkspaceUiState::new(filters))
    }

    #[test]
    fn opening_defaults_to_global_and_projects_exactly_eight_resolved_values() {
        let (root, mut coordinator, mut ui) = fixture();
        coordinator.open_new_project(&mut ui, ApplicationFocus::default());
        assert_eq!(ui.new_project.units_choice, NewProjectUnitsChoice::Global);
        assert_eq!(ui.new_project.units_summary.len(), 8);
        assert_eq!(
            ui.new_project
                .units_summary
                .iter()
                .map(|row| row.key.as_str())
                .collect::<Vec<_>>(),
            ACTIVE_UNITS_KEYS
        );
        assert_eq!(
            ui.new_project.source_detail,
            "factory defaults; no preference file yet"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn factory_requires_explicit_choice_and_never_creates_a_preference_repository() {
        let (root, mut coordinator, mut ui) = fixture();
        coordinator.open_new_project(&mut ui, ApplicationFocus::default());
        ui.new_project.units_choice = NewProjectUnitsChoice::Factory;
        coordinator.refresh_new_project_preview(&mut ui);
        assert!(
            ui.new_project
                .source_detail
                .contains("datum.units.factory.v1")
        );
        assert!(!root.join("preferences").exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn create_publishes_one_real_project_with_embedded_eight_item_receipt() {
        let (root, mut coordinator, mut ui) = fixture();
        coordinator.open_new_project(&mut ui, ApplicationFocus::default());
        let destination = root.join("created-project");
        ui.new_project.project_name = "created-project".to_owned();
        ui.new_project.destination = destination.to_string_lossy().into_owned();
        coordinator.create_new_project(&mut ui).unwrap();
        let project: Value =
            serde_json::from_slice(&std::fs::read(destination.join("project.json")).unwrap())
                .unwrap();
        assert_eq!(
            project["project_units_seed_receipt"]["items"]
                .as_array()
                .map(Vec::len),
            Some(8)
        );
        assert!(!project.to_string().contains("revision"));
        let _ = std::fs::remove_dir_all(root);
    }
}
