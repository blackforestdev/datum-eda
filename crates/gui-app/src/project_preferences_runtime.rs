//! Project-owned Units Preferences coordinator.
//!
//! The window is a consumer of the engine Units profile and native-write
//! facade. It never writes Project files directly and never consults Global
//! preferences after the Project seed has been recorded.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use datum_gui_protocol::{
    ApplicationFocus, GlobalPreferenceControlUi, GlobalPreferenceRowUi,
    GlobalPreferencesDialogState, GlobalPreferencesDismissal, GlobalPreferencesFocus,
    GlobalPreferencesNoticeUi,
};
use datum_gui_render::HitTarget;
use eda_engine::api::native_write::project::{
    ProjectUnitsSeedEvidence, build_initialize_project_display_units,
    build_set_project_display_units, project_display_units, project_units_seed_evidence,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::ir::units::{
    ACTIVE_UNITS_KEYS, PreFeatureProjectUnitsMigration, migrate_pre_feature_project_units,
    profile_to_descriptor_values,
};
use eda_engine::preferences::{
    DescriptorRegistry, PreferenceSurfaceCatalog, active_v1_registry, gp_f05_surface_catalog,
};
use eda_engine::substrate::{CommitSource, ModelRevision, ProjectResolver};
use serde_json::Value;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};

use crate::Runtime;
use crate::console_accessibility::{AccessibilityAnnouncement, AnnouncementPriority};
use crate::global_preferences_projection::control_value_projection;

pub(super) struct ProjectPreferencesCoordinator {
    registry: DescriptorRegistry,
    surface: PreferenceSurfaceCatalog,
    project_root: Option<PathBuf>,
    expected_revision: Option<ModelRevision>,
    receipt: Option<ProjectUnitsSeedEvidence>,
    return_focus: ApplicationFocus,
}

impl ProjectPreferencesCoordinator {
    pub(super) fn new() -> Self {
        let registry = active_v1_registry();
        let surface = gp_f05_surface_catalog(&registry)
            .expect("the governed Preferences surface catalog must construct");
        Self {
            registry,
            surface,
            project_root: None,
            expected_revision: None,
            receipt: None,
            return_focus: ApplicationFocus::default(),
        }
    }

    fn open_dialog(
        &mut self,
        workspace: &mut datum_gui_protocol::ReviewWorkspaceState,
        invoker: ApplicationFocus,
    ) -> Result<bool> {
        if workspace.ui.project_preferences.open {
            return Ok(false);
        }
        let root = workspace
            .backing
            .as_ref()
            .map(|backing| backing.request.project_root.clone())
            .context("Project Preferences requires an open Project")?;
        self.return_focus = invoker;
        workspace.ui.project_preferences.reset_transient_view();
        if let Err(error) = self.load_or_migrate(&root, &mut workspace.ui.project_preferences) {
            self.publish_unavailable(&root, &error, &mut workspace.ui.project_preferences);
        }
        workspace.ui.active_menu = None;
        workspace.ui.active_submenu = None;
        workspace.ui.project_preferences.open = true;
        Ok(true)
    }

    fn publish_unavailable(
        &mut self,
        root: &Path,
        error: &anyhow::Error,
        dialog: &mut GlobalPreferencesDialogState,
    ) {
        let project_name = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Project");
        let profile = eda_engine::ir::units::FACTORY_UNITS_PROFILE_V1;
        let values = profile_to_descriptor_values(profile);
        let resolved = profile
            .resolve()
            .expect("the versioned factory Units profile must remain valid");
        let reason = format!(
            "Project Working Units could not be read: {error}. The preserved Project was not changed"
        );
        let rows = self
            .surface
            .entries()
            .iter()
            .filter(|entry| ACTIVE_UNITS_KEYS.contains(&entry.key.as_str()))
            .map(|entry| {
                let key = entry.key.as_str();
                let descriptor = self
                    .registry
                    .get(&entry.key)
                    .expect("surface catalog proves descriptor existence");
                GlobalPreferenceRowUi {
                    key: key.to_owned(),
                    section_id: "units".to_owned(),
                    label: descriptor.presentation.label.clone(),
                    description: descriptor.presentation.description.clone(),
                    aliases: descriptor.retired_aliases.iter().cloned().collect(),
                    scope: format!("Project · {project_name}"),
                    provenance: "⛝ Unavailable · preserved Project value".to_owned(),
                    explanation_lines: vec![
                        format!("Descriptor  {key}"),
                        "Authority   Project Working Units".to_owned(),
                        reason.clone(),
                    ],
                    control: control_value_projection(
                        &entry.control,
                        values.get(key),
                        key,
                        Some(&resolved),
                    ),
                    changed: false,
                    writable: false,
                    unavailable_reason: Some(reason.clone()),
                    reset_description:
                        "Unavailable because the recorded seed or migration receipt cannot be read."
                            .to_owned(),
                }
            })
            .collect();
        *dialog = GlobalPreferencesDialogState {
            open: dialog.open,
            title: "Project Preferences — Datum".to_owned(),
            scope: format!("Project · {project_name}"),
            sections: vec![("units".to_owned(), "Units".to_owned())],
            section_id: "units".to_owned(),
            section_label: "Units".to_owned(),
            search_query: String::new(),
            scroll_row: 0,
            rows,
            explanation_key: None,
            open_choice_key: None,
            focus: GlobalPreferencesFocus::SectionNavigation,
            notice: Some(GlobalPreferencesNoticeUi::PreservedUnreadable(reason)),
            reduced_motion: dialog.reduced_motion,
            high_contrast_noncolor: false,
        };
        self.project_root = None;
        self.expected_revision = None;
        self.receipt = None;
    }

    fn load_or_migrate(
        &mut self,
        root: &Path,
        dialog: &mut GlobalPreferencesDialogState,
    ) -> Result<()> {
        let mut model = ProjectResolver::new(root).resolve()?;
        if model.project.project_display_units.is_none() {
            let PreFeatureProjectUnitsMigration::Create { profile, receipt } =
                migrate_pre_feature_project_units(None)
                    .map_err(|refusal| anyhow::anyhow!("Units migration refused: {refusal:?}"))?
            else {
                unreachable!("an absent profile always produces a migration")
            };
            let prepared = build_initialize_project_display_units(
                &model,
                WriteProvenance::new(
                    "datum-gui",
                    CommitSource::Manual,
                    "Migrate pre-feature Project Working Units from factory profile",
                ),
                profile,
                &receipt,
            )?;
            commit_prepared(&mut model, root, prepared)?;
            model = ProjectResolver::new(root).resolve()?;
        }
        self.publish_model(root, &model, dialog)
    }

    fn publish_model(
        &mut self,
        root: &Path,
        model: &eda_engine::substrate::DesignModel,
        dialog: &mut GlobalPreferencesDialogState,
    ) -> Result<()> {
        let profile = project_display_units(model)?;
        let resolved = profile
            .resolve()
            .map_err(|reason| anyhow::anyhow!("Project Working Units refused: {reason:?}"))?;
        let receipt = project_units_seed_evidence(model)?;
        let values = profile_to_descriptor_values(profile);
        let seed_values = &receipt.copied_values;
        let rows = self
            .surface
            .entries()
            .iter()
            .filter(|entry| ACTIVE_UNITS_KEYS.contains(&entry.key.as_str()))
            .map(|entry| {
                let key = entry.key.as_str();
                let descriptor = self
                    .registry
                    .get(&entry.key)
                    .expect("surface catalog proves descriptor existence");
                let value = values.get(key).expect("complete Units profile");
                let changed = seed_values.get(key) != Some(value);
                GlobalPreferenceRowUi {
                    key: key.to_owned(),
                    section_id: "units".to_owned(),
                    label: descriptor.presentation.label.clone(),
                    description: descriptor.presentation.description.replace(
                        "default copied into newly created Projects",
                        "working value used by this Project",
                    ),
                    aliases: descriptor.retired_aliases.iter().cloned().collect(),
                    scope: format!("Project · {}", model.project.name),
                    provenance: if changed {
                        format!(
                            "Set in this Project · Project · {} · Undo available",
                            model.project.name
                        )
                    } else {
                        format!(
                            "Recorded seed or migration value · Project · {}",
                            model.project.name
                        )
                    },
                    explanation_lines: vec![
                        format!("Descriptor  {key}"),
                        "Authority   Project Working Units".to_owned(),
                        format!("Stored      {value}"),
                        format!("Receipt     {}", receipt.source_summary),
                    ],
                    control: control_value_projection(
                        &entry.control,
                        Some(value),
                        key,
                        Some(&resolved),
                    ),
                    changed,
                    writable: true,
                    unavailable_reason: None,
                    reset_description: "Restores this field to the Project's recorded seed or migration value through one undoable journal mutation; it does not read Global defaults."
                        .to_owned(),
                }
            })
            .collect();
        let was_open = dialog.open;
        let search_query = dialog.search_query.clone();
        let scroll_row = dialog.scroll_row;
        let explanation_key = dialog.explanation_key.clone();
        let open_choice_key = dialog.open_choice_key.clone();
        let focus = dialog.focus.clone();
        *dialog = GlobalPreferencesDialogState {
            open: was_open,
            title: "Project Preferences — Datum".to_owned(),
            scope: format!("Project · {}", model.project.name),
            sections: vec![("units".to_owned(), "Units".to_owned())],
            section_id: "units".to_owned(),
            section_label: "Units".to_owned(),
            search_query,
            scroll_row,
            rows,
            explanation_key,
            open_choice_key,
            focus,
            notice: None,
            reduced_motion: dialog.reduced_motion,
            high_contrast_noncolor: false,
        };
        self.project_root = Some(root.to_path_buf());
        self.expected_revision = Some(model.model_revision.clone());
        self.receipt = Some(receipt);
        Ok(())
    }

    fn set_value(
        &mut self,
        key: &str,
        value: Value,
        dialog: &mut GlobalPreferencesDialogState,
    ) -> Result<()> {
        let root = self
            .project_root
            .clone()
            .context("Project Preferences has no Project root")?;
        let mut model = ProjectResolver::new(&root).resolve()?;
        if self.expected_revision.as_ref() != Some(&model.model_revision) {
            self.publish_model(&root, &model, dialog)?;
            anyhow::bail!(
                "Project changed outside this window; values were reloaded and the stale edit was not applied"
            )
        }
        let profile = project_display_units(&model)?;
        let mut values = profile_to_descriptor_values(profile);
        let target = values
            .get_mut(key)
            .with_context(|| format!("unknown Project Units key {key}"))?;
        *target = value;
        let profile = eda_engine::ir::units::profile_from_descriptor_values(&values)
            .map_err(|reason| anyhow::anyhow!("Project Working Units refused: {reason:?}"))?;
        let prepared = build_set_project_display_units(
            &model,
            WriteProvenance::new(
                "datum-gui",
                CommitSource::Manual,
                format!("Set Project Working Units field {key}"),
            ),
            profile,
        )?;
        commit_prepared(&mut model, &root, prepared)?;
        let model = ProjectResolver::new(&root).resolve()?;
        self.publish_model(&root, &model, dialog)?;
        dialog.notice = Some(GlobalPreferencesNoticeUi::Polite(
            "Project Working Units changed. The Project journal can undo this change; geometry was not rescaled."
                .to_owned(),
        ));
        Ok(())
    }

    fn reset(&mut self, key: &str, dialog: &mut GlobalPreferencesDialogState) -> Result<()> {
        let value = self
            .receipt
            .as_ref()
            .and_then(|receipt| receipt.copied_values.get(key))
            .cloned()
            .with_context(|| {
                format!("recorded seed or migration value for {key} is unavailable")
            })?;
        self.set_value(key, value, dialog)
    }
}

impl Runtime {
    pub(super) fn activate_project_preferences_hit_target(
        &mut self,
        target: &HitTarget,
    ) -> Option<bool> {
        let handled = match target {
            HitTarget::GlobalPreferencesModal => true,
            HitTarget::GlobalPreferencesSection(section_id) => {
                self.session
                    .workspace_mut()
                    .ui
                    .project_preferences
                    .select_section(section_id);
                self.refresh_dialog_state();
                true
            }
            HitTarget::GlobalPreferencesSearch => {
                self.session.workspace_mut().ui.project_preferences.focus =
                    GlobalPreferencesFocus::Search;
                self.refresh_dialog_state();
                true
            }
            HitTarget::GlobalPreferencesSettingName(key) => {
                let dialog = &mut self.session.workspace_mut().ui.project_preferences;
                dialog.open_choice_key = None;
                dialog.explanation_key = Some(key.clone());
                dialog.focus = GlobalPreferencesFocus::ExplanationClose;
                self.refresh_dialog_state();
                true
            }
            HitTarget::GlobalPreferencesControl(key) => {
                self.activate_project_preference_control(key)
            }
            HitTarget::GlobalPreferencesChoice { key, value } => {
                self.choose_project_preference_value(key, value)
            }
            HitTarget::GlobalPreferencesReset(key) => self.reset_project_preference(key),
            HitTarget::GlobalPreferencesExplanationClose => {
                let dialog = &mut self.session.workspace_mut().ui.project_preferences;
                if let Some(key) = dialog.explanation_key.take() {
                    dialog.focus = GlobalPreferencesFocus::SettingName(key);
                }
                self.refresh_dialog_state();
                true
            }
            _ => return None,
        };
        Some(handled)
    }

    pub(super) fn open_project_preferences(&mut self) -> bool {
        if self.workspace().ui.global_preferences.open {
            self.close_global_preferences();
        }
        let invoker = self.application_focus();
        let mut coordinator = std::mem::replace(
            &mut self.project_preferences,
            ProjectPreferencesCoordinator::new(),
        );
        let result = coordinator.open_dialog(self.session.workspace_mut(), invoker);
        self.project_preferences = coordinator;
        match result {
            Ok(opened) => {
                self.set_application_focus(ApplicationFocus::Overlay);
                self.project_preferences_raise_requested = true;
                if opened {
                    self.announce_project_preferences(
                        "Project Preferences opened. Units, eight settings. Changes save immediately to the Project journal and can be undone.",
                        AnnouncementPriority::Medium,
                    );
                }
            }
            Err(error) => self.log_console_refusal(
                datum_gui_protocol::ConsoleFeedbackSource::Menu,
                format!("Project Preferences unavailable: {error}"),
            ),
        }
        self.invalidate_frame();
        true
    }

    pub(super) fn take_project_preferences_raise_request(&mut self) -> bool {
        std::mem::take(&mut self.project_preferences_raise_requested)
    }

    pub(super) fn close_project_preferences(&mut self) -> bool {
        if !self.workspace().ui.project_preferences.open {
            return false;
        }
        self.session.workspace_mut().ui.project_preferences.open = false;
        self.session
            .workspace_mut()
            .ui
            .project_preferences
            .reset_transient_view();
        self.set_application_focus(self.project_preferences.return_focus);
        self.invalidate_frame();
        true
    }

    pub(super) fn activate_project_preference_control(&mut self, key: &str) -> bool {
        let control = self
            .workspace()
            .ui
            .project_preferences
            .rows
            .iter()
            .find(|row| row.key == key)
            .map(|row| row.control.clone());
        let Some(GlobalPreferenceControlUi::SingleChoice { .. }) = control else {
            return false;
        };
        let dialog = &mut self.session.workspace_mut().ui.project_preferences;
        dialog.explanation_key = None;
        dialog.open_choice_key =
            (dialog.open_choice_key.as_deref() != Some(key)).then(|| key.to_owned());
        if dialog.open_choice_key.is_some() {
            dialog.scroll_to_row(key);
        }
        self.refresh_dialog_state();
        true
    }

    pub(super) fn choose_project_preference_value(&mut self, key: &str, value: &str) -> bool {
        let mut coordinator = std::mem::replace(
            &mut self.project_preferences,
            ProjectPreferencesCoordinator::new(),
        );
        let result = coordinator.set_value(
            key,
            Value::String(value.to_owned()),
            &mut self.session.workspace_mut().ui.project_preferences,
        );
        self.project_preferences = coordinator;
        self.finish_project_preference_mutation(result)
    }

    pub(super) fn reset_project_preference(&mut self, key: &str) -> bool {
        let mut coordinator = std::mem::replace(
            &mut self.project_preferences,
            ProjectPreferencesCoordinator::new(),
        );
        let result = coordinator.reset(
            key,
            &mut self.session.workspace_mut().ui.project_preferences,
        );
        self.project_preferences = coordinator;
        self.finish_project_preference_mutation(result)
    }

    fn finish_project_preference_mutation(&mut self, result: Result<()>) -> bool {
        if let Err(error) = result {
            self.session.workspace_mut().ui.project_preferences.notice =
                Some(GlobalPreferencesNoticeUi::Assertive(error.to_string()));
        }
        self.session
            .workspace_mut()
            .ui
            .project_preferences
            .open_choice_key = None;
        self.invalidate_frame();
        true
    }

    pub(super) fn handle_project_preferences_key(
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
                if let Some(key) = dialog.explanation_key.take() {
                    dialog.focus = GlobalPreferencesFocus::SettingName(key);
                }
                self.refresh_dialog_state();
                Outcome::Dialog
            }
            _ => Outcome::Consumed,
        }
    }

    fn announce_project_preferences(&mut self, message: &str, priority: AnnouncementPriority) {
        self.terminal_accessibility
            .announce_console(AccessibilityAnnouncement {
                text: message.to_owned(),
                priority,
            });
    }
}

#[cfg(test)]
#[path = "project_preferences_runtime_tests.rs"]
mod tests;
