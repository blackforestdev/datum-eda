//! Consumer-only state for the native New Project genesis dialog.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewProjectUnitsChoice {
    Global,
    Factory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewProjectFocus {
    ProjectName,
    Destination,
    UnitsChoice,
    UnitsSummary,
    RetryGlobal,
    Cancel,
    Create,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProjectUnitsSummaryRow {
    pub key: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProjectDialogState {
    pub open: bool,
    pub project_name: String,
    pub destination: String,
    pub units_choice: NewProjectUnitsChoice,
    pub units_summary: Vec<NewProjectUnitsSummaryRow>,
    pub source_summary: String,
    pub source_detail: String,
    pub refusal: Option<String>,
    pub focus: NewProjectFocus,
}

impl Default for NewProjectDialogState {
    fn default() -> Self {
        Self {
            open: false,
            project_name: String::new(),
            destination: String::new(),
            units_choice: NewProjectUnitsChoice::Global,
            units_summary: Vec::new(),
            source_summary: String::new(),
            source_detail: String::new(),
            refusal: None,
            focus: NewProjectFocus::ProjectName,
        }
    }
}

impl NewProjectDialogState {
    pub fn create_enabled(&self) -> bool {
        !self.project_name.trim().is_empty()
            && !self.destination.trim().is_empty()
            && self.refusal.is_none()
            && self.units_summary.len() == 8
    }

    pub fn reset_for_open(&mut self) {
        *self = Self::default();
        self.open = true;
    }

    pub fn focus_order(&self) -> Vec<NewProjectFocus> {
        let mut order = vec![
            NewProjectFocus::ProjectName,
            NewProjectFocus::Destination,
            NewProjectFocus::UnitsChoice,
            NewProjectFocus::UnitsSummary,
        ];
        if self.refusal.is_some() && self.units_choice == NewProjectUnitsChoice::Global {
            order.push(NewProjectFocus::RetryGlobal);
        }
        order.push(NewProjectFocus::Cancel);
        if self.create_enabled() {
            order.push(NewProjectFocus::Create);
        }
        order
    }

    pub fn move_focus(&mut self, reverse: bool) {
        let order = self.focus_order();
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
        let selected = match self.units_choice {
            NewProjectUnitsChoice::Global => "Use my Global Units defaults",
            NewProjectUnitsChoice::Factory => "Use Datum factory Units",
        };
        let mut nodes = vec![
            Node { id: "new-project-dialog".to_owned(), name: "New Project".to_owned(), role: Role::Dialog, value: None, description: "Creates one Project through the engine-owned atomic genesis service.".to_owned(), available: true, focused: false },
            Node { id: "new-project-name".to_owned(), name: "Project name".to_owned(), role: Role::TextBox, value: Some(self.project_name.clone()), description: "Name preserved when creation is refused.".to_owned(), available: true, focused: self.focus == NewProjectFocus::ProjectName },
            Node { id: "new-project-destination".to_owned(), name: "Location".to_owned(), role: Role::TextBox, value: Some(self.destination.clone()), description: "Full destination path preserved when creation is refused.".to_owned(), available: true, focused: self.focus == NewProjectFocus::Destination },
            Node { id: "new-project-units".to_owned(), name: "Working units for this new Project".to_owned(), role: Role::RadioGroup, value: Some(selected.to_owned()), description: "Two choices. Arrow keys change the explicit selection; Datum never selects factory as a fallback.".to_owned(), available: true, focused: self.focus == NewProjectFocus::UnitsChoice },
        ];
        for (choice, label) in [
            (
                NewProjectUnitsChoice::Global,
                "Use my Global Units defaults",
            ),
            (NewProjectUnitsChoice::Factory, "Use Datum factory Units"),
        ] {
            nodes.push(Node {
                id: format!("new-project-units-{choice:?}").to_ascii_lowercase(),
                name: label.to_owned(),
                role: Role::RadioButton,
                value: Some(
                    if self.units_choice == choice {
                        "selected"
                    } else {
                        "not selected"
                    }
                    .to_owned(),
                ),
                description: "One of two Working Units sources.".to_owned(),
                available: true,
                focused: false,
            });
        }
        let values = self
            .units_summary
            .iter()
            .map(|row| format!("{}: {}", row.label, row.value))
            .collect::<Vec<_>>()
            .join("; ");
        nodes.push(Node {
            id: "new-project-units-summary".to_owned(),
            name: "Working units this Project will be created with".to_owned(),
            role: Role::Navigation,
            value: Some(values),
            description: format!("{}. {}", self.source_summary, self.source_detail),
            available: self.refusal.is_none(),
            focused: self.focus == NewProjectFocus::UnitsSummary,
        });
        if let Some(refusal) = &self.refusal {
            nodes.push(Node {
                id: "new-project-refusal".to_owned(),
                name: "Project not created".to_owned(),
                role: Role::Status,
                value: Some(refusal.clone()),
                description: "Factory remains available but is not selected automatically."
                    .to_owned(),
                available: true,
                focused: false,
            });
            if self.units_choice == NewProjectUnitsChoice::Global {
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
            description: "The sole action that atomically publishes the Project.".to_owned(),
            available: self.create_enabled(),
            focused: self.focus == NewProjectFocus::Create,
        });
        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_requires_complete_input_and_exact_eight_item_snapshot() {
        let mut state = NewProjectDialogState {
            project_name: "fixture".to_owned(),
            destination: "/tmp/fixture".to_owned(),
            units_summary: (0..8)
                .map(|index| NewProjectUnitsSummaryRow {
                    key: format!("key-{index}"),
                    label: format!("label-{index}"),
                    value: format!("value-{index}"),
                })
                .collect(),
            ..Default::default()
        };
        assert!(state.create_enabled());
        state.refusal = Some("refused".to_owned());
        assert!(!state.create_enabled());
    }

    #[test]
    fn refusal_adds_retry_without_making_unavailable_create_focusable() {
        let state = NewProjectDialogState {
            refusal: Some("refused".to_owned()),
            ..Default::default()
        };
        assert!(state.focus_order().contains(&NewProjectFocus::RetryGlobal));
        assert!(!state.focus_order().contains(&NewProjectFocus::Create));
    }
}
