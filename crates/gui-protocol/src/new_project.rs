//! Consumer-only state for the native New Project genesis dialog.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewProjectFocus {
    ProjectName,
    Navigation(crate::FolderBrowserFocus),
    UseProjectName,
    FolderName,
    RetryGlobal,
    Cancel,
    Create,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProjectDialogState {
    pub open: bool,
    pub project_name: String,
    pub project_name_editor: crate::single_line_editor::SingleLineEditor,
    pub destination: String,
    pub location_ready: bool,
    pub use_project_name: bool,
    pub folder_name: String,
    pub folder_name_editor: crate::single_line_editor::SingleLineEditor,
    pub host_action_pending: bool,
    /// Eligibility projection of the coordinator's validated, pinned Global snapshot.
    pub global_units_ready: bool,
    pub refusal: Option<String>,
    pub focus: NewProjectFocus,
}

impl Default for NewProjectDialogState {
    fn default() -> Self {
        Self {
            open: false,
            project_name: String::new(),
            project_name_editor: Default::default(),
            destination: String::new(),
            location_ready: false,
            use_project_name: true,
            folder_name: String::new(),
            folder_name_editor: Default::default(),
            host_action_pending: false,
            global_units_ready: false,
            refusal: None,
            focus: NewProjectFocus::ProjectName,
        }
    }
}

impl NewProjectDialogState {
    pub fn effective_folder_name(&self) -> &str {
        if self.use_project_name {
            &self.project_name
        } else {
            &self.folder_name
        }
    }
    pub fn set_use_project_name(&mut self, enabled: bool) {
        if self.use_project_name != enabled {
            self.folder_name = self.project_name.clone();
            self.folder_name_editor =
                crate::single_line_editor::SingleLineEditor::at_end(&self.folder_name);
        }
        self.use_project_name = enabled;
        self.clear_creation_refusal();
    }

    pub fn units_ready(&self) -> bool {
        self.global_units_ready
    }

    pub fn retry_global_available(&self) -> bool {
        !self.units_ready() && self.refusal.is_some()
    }

    /// Editing a destination/name can resolve creation errors, never a Units error.
    pub fn clear_creation_refusal(&mut self) {
        if self.units_ready() {
            self.refusal = None;
        }
    }

    pub fn create_enabled(&self) -> bool {
        !self.host_action_pending
            && !self.project_name.trim().is_empty()
            && !self.destination.trim().is_empty()
            && self.units_ready()
            && self.location_ready
    }

    pub fn reset_for_open(&mut self) {
        *self = Self::default();
        self.open = true;
    }

    pub fn focus_order(&self, expanded: bool) -> Vec<NewProjectFocus> {
        let mut order = vec![NewProjectFocus::ProjectName];
        if !self.use_project_name {
            order.push(NewProjectFocus::FolderName);
        }
        order.push(NewProjectFocus::UseProjectName);
        order.extend(
            crate::folder_navigation_focus_order(expanded)
                .into_iter()
                .map(NewProjectFocus::Navigation),
        );
        if self.retry_global_available() {
            order.push(NewProjectFocus::RetryGlobal);
        }
        order.push(NewProjectFocus::Cancel);
        if self.create_enabled() {
            order.push(NewProjectFocus::Create);
        }
        order
    }

    pub fn move_focus(&mut self, reverse: bool, expanded: bool) {
        if self.use_project_name && self.focus == NewProjectFocus::FolderName {
            self.focus = if reverse {
                NewProjectFocus::ProjectName
            } else {
                NewProjectFocus::UseProjectName
            };
            return;
        }
        let order = self.focus_order(expanded);
        let index = order
            .iter()
            .position(|item| *item == self.focus)
            .unwrap_or(0);
        let next = if reverse {
            index.checked_sub(1).unwrap_or(order.len() - 1)
        } else {
            (index + 1) % order.len()
        };
        self.focus = order[next];
    }

    pub fn accessibility_nodes(&self) -> Vec<crate::GlobalPreferencesAccessibleNode> {
        use crate::{
            GlobalPreferencesAccessibleNode as Node, GlobalPreferencesAccessibleRole as Role,
        };
        if !self.open {
            return Vec::new();
        }
        let mut nodes = vec![
            Node {
                id: "new-project-dialog".to_owned(),
                name: "New Project".to_owned(),
                role: Role::Dialog,
                value: None,
                description: "Creates a Project in the folder shown in the preview.".to_owned(),
                available: true,
                focused: false,
            },
            Node {
                id: "new-project-name".to_owned(),
                name: "Project name".to_owned(),
                role: Role::TextBox,
                value: Some(self.project_name.clone()),
                description: "Name preserved when creation is refused.".to_owned(),
                available: true,
                focused: self.focus == NewProjectFocus::ProjectName,
            },
            Node {
                id: "new-project-folder-name".into(),
                name: "Folder name".into(),
                role: Role::TextBox,
                value: Some(self.effective_folder_name().into()),
                description: if self.use_project_name {
                    "Read-only, from Project name. Uncheck Use project name to edit an override."
                } else {
                    "Editable folder name override."
                }
                .into(),
                available: true,
                focused: self.focus == NewProjectFocus::FolderName,
            },
        ];
        nodes.push(Node { id: "new-project-use-name".into(), name: "Use project name".into(), role: Role::CheckBox, value: Some(self.use_project_name.to_string()), description: "Checked derives Folder name exactly from Project name. Uncheck for an override; checking restores derivation.".into(), available: true, focused: self.focus == NewProjectFocus::UseProjectName });
        if let Some(refusal) = &self.refusal {
            nodes.push(Node {
                id: "new-project-refusal".to_owned(),
                name: "Project not created".to_owned(),
                role: Role::Status,
                value: Some(refusal.clone()),
                description: if self.units_ready() {
                    "The draft is preserved. Correct the destination and try Create again."
                } else {
                    "The draft is preserved. Resolve the reported Global Preferences problem, then explicitly re-read the defaults; no Project was created."
                }.to_owned(),
                available: true,
                focused: false,
            });
            if self.retry_global_available() {
                nodes.push(Node {
                    id: "new-project-retry".to_owned(),
                    name: "Re-read Global defaults".to_owned(),
                    role: Role::Button,
                    value: None,
                    description:
                        "Retries the Global Units preview without changing the selected source."
                            .to_owned(),
                    available: true,
                    focused: self.focus == NewProjectFocus::RetryGlobal,
                });
            }
        }
        nodes.push(Node {
            id: "new-project-cancel".to_owned(),
            name: "Cancel".to_owned(),
            role: Role::Button,
            value: None,
            description: "Closes without creating a Project.".to_owned(),
            available: true,
            focused: self.focus == NewProjectFocus::Cancel,
        });
        nodes.push(Node {
            id: "new-project-create".to_owned(),
            name: "Create".to_owned(),
            role: Role::Button,
            value: None,
            description: if self.host_action_pending {
                crate::FolderBrowserMode::New.host_action_wait_reason()
            } else {
                "Creates the Project in the previewed folder."
            }
            .to_owned(),
            available: self.create_enabled(),
            focused: self.focus == NewProjectFocus::Create,
        });
        if self.host_action_pending {
            nodes.push(crate::FolderBrowserMode::New.host_action_wait_node());
        }
        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_requires_a_validated_global_snapshot_without_units_controls() {
        let mut state = NewProjectDialogState {
            open: true,
            project_name: "fixture".into(),
            destination: "/fixture".into(),
            location_ready: true,
            ..Default::default()
        };
        assert!(!state.create_enabled());
        state.global_units_ready = true;
        assert!(state.create_enabled());
        state.refusal = Some("Destination refused".into());
        assert!(state.create_enabled() && !state.retry_global_available());
        assert!(
            state
                .accessibility_nodes()
                .iter()
                .find(|n| n.id == "new-project-refusal")
                .unwrap()
                .description
                .contains("Correct the destination")
        );
        state.clear_creation_refusal();
        assert!(state.refusal.is_none());
        let nodes = state.accessibility_nodes();
        assert!(
            !nodes
                .iter()
                .any(|n| n.id.contains("units") || n.name.contains("Factory"))
        );
        assert_eq!(
            state.focus_order(false),
            vec![
                NewProjectFocus::ProjectName,
                NewProjectFocus::UseProjectName,
                NewProjectFocus::Navigation(crate::FolderBrowserFocus::Path),
                NewProjectFocus::Navigation(crate::FolderBrowserFocus::Browse),
                NewProjectFocus::Navigation(crate::FolderBrowserFocus::Preview),
                NewProjectFocus::Cancel,
                NewProjectFocus::Create,
            ]
        );
    }

    #[test]
    fn source_refusal_requires_explicit_reread_and_preserves_draft() {
        let mut state = NewProjectDialogState {
            open: true,
            project_name: "preserved".into(),
            destination: "/preserved".into(),
            refusal: Some("Global source unavailable".into()),
            ..Default::default()
        };
        state.clear_creation_refusal();
        assert!(state.refusal.is_some() && state.retry_global_available());
        assert!(!state.focus_order(false).contains(&NewProjectFocus::Create));
        assert!(
            state
                .focus_order(false)
                .contains(&NewProjectFocus::RetryGlobal)
        );
        assert_eq!(state.project_name, "preserved");
        assert_eq!(state.destination, "/preserved");
        let nodes = state.accessibility_nodes();
        assert!(nodes.iter().any(|n| n.name == "Re-read Global defaults"));
        assert!(!nodes.iter().any(|n| n.description.contains("Factory")));
    }
}
