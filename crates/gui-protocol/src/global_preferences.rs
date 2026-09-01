//! Consumer-only projection of the engine-owned Global Preferences surface.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalPreferenceControlUi {
    Boolean {
        value: bool,
        off_label: String,
        on_label: String,
    },
    SingleChoice {
        value: String,
        choices: Vec<(String, String)>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPreferenceRowUi {
    pub key: String,
    pub label: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub scope: String,
    pub provenance: String,
    pub explanation_lines: Vec<String>,
    pub control: GlobalPreferenceControlUi,
    pub changed: bool,
    pub writable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalPreferencesNoticeUi {
    Polite(String),
    Assertive(String),
    PreservedUnreadable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalPreferencesFocus {
    SectionNavigation,
    Search,
    SettingName(String),
    Control(String),
    Reset(String),
    ExplanationClose,
    DialogClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalPreferencesDismissal {
    ChoiceClosed,
    ExplanationClosed,
    DialogClosed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPreferencesDialogState {
    pub open: bool,
    pub section_id: String,
    pub section_label: String,
    pub search_query: String,
    pub rows: Vec<GlobalPreferenceRowUi>,
    pub explanation_key: Option<String>,
    pub open_choice_key: Option<String>,
    pub focus: GlobalPreferencesFocus,
    pub notice: Option<GlobalPreferencesNoticeUi>,
    pub reduced_motion: bool,
    pub high_contrast_noncolor: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalPreferencesAccessibleRole {
    Dialog,
    Navigation,
    SearchBox,
    Button,
    Switch,
    ComboBox,
    Status,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPreferencesAccessibleNode {
    pub id: String,
    pub name: String,
    pub role: GlobalPreferencesAccessibleRole,
    pub value: Option<String>,
    pub description: String,
    pub available: bool,
    pub focused: bool,
}

impl Default for GlobalPreferencesDialogState {
    fn default() -> Self {
        Self {
            open: false,
            section_id: "appearance".to_owned(),
            section_label: "Appearance".to_owned(),
            search_query: String::new(),
            rows: Vec::new(),
            explanation_key: None,
            open_choice_key: None,
            focus: GlobalPreferencesFocus::SectionNavigation,
            notice: None,
            reduced_motion: false,
            high_contrast_noncolor: false,
        }
    }
}

impl GlobalPreferencesDialogState {
    pub fn visible_rows(&self) -> impl Iterator<Item = &GlobalPreferenceRowUi> {
        let query = self.search_query.trim().to_ascii_lowercase();
        self.rows.iter().filter(move |row| {
            query.is_empty()
                || row.label.to_ascii_lowercase().contains(&query)
                || row.description.to_ascii_lowercase().contains(&query)
                || row.key.to_ascii_lowercase().contains(&query)
                || row
                    .aliases
                    .iter()
                    .any(|alias| alias.to_ascii_lowercase().contains(&query))
        })
    }

    pub fn accessibility_nodes(&self) -> Vec<GlobalPreferencesAccessibleNode> {
        if !self.open {
            return Vec::new();
        }
        let mut nodes = vec![
            GlobalPreferencesAccessibleNode {
                id: "global-preferences".to_owned(),
                name: "Global Preferences".to_owned(),
                role: GlobalPreferencesAccessibleRole::Dialog,
                value: Some("Global · this device".to_owned()),
                description: "Application preferences for this device.".to_owned(),
                available: true,
                focused: false,
            },
            GlobalPreferencesAccessibleNode {
                id: "appearance-navigation".to_owned(),
                name: self.section_label.clone(),
                role: GlobalPreferencesAccessibleRole::Navigation,
                value: None,
                description: "Global Preferences section navigation.".to_owned(),
                available: true,
                focused: self.focus == GlobalPreferencesFocus::SectionNavigation,
            },
            GlobalPreferencesAccessibleNode {
                id: "appearance-search".to_owned(),
                name: "Search Appearance settings".to_owned(),
                role: GlobalPreferencesAccessibleRole::SearchBox,
                value: Some(self.search_query.clone()),
                description: "Searches labels, descriptions, stable keys, and registered aliases."
                    .to_owned(),
                available: true,
                focused: self.focus == GlobalPreferencesFocus::Search,
            },
        ];
        for row in self.visible_rows() {
            nodes.push(GlobalPreferencesAccessibleNode {
                id: format!("{}-name", row.key),
                name: row.label.clone(),
                role: GlobalPreferencesAccessibleRole::Button,
                value: None,
                description: format!(
                    "{}. {}. Opens resolver explanation.",
                    row.description, row.provenance
                ),
                available: true,
                focused: self.focus == GlobalPreferencesFocus::SettingName(row.key.clone()),
            });
            let (role, value) = match &row.control {
                GlobalPreferenceControlUi::Boolean {
                    value,
                    off_label,
                    on_label,
                } => (
                    GlobalPreferencesAccessibleRole::Switch,
                    if *value {
                        on_label.clone()
                    } else {
                        off_label.clone()
                    },
                ),
                GlobalPreferenceControlUi::SingleChoice { value, choices } => (
                    GlobalPreferencesAccessibleRole::ComboBox,
                    choices
                        .iter()
                        .find(|(candidate, _)| candidate == value)
                        .map(|(_, label)| label.clone())
                        .unwrap_or_else(|| value.clone()),
                ),
            };
            nodes.push(GlobalPreferencesAccessibleNode {
                id: format!("{}-control", row.key),
                name: row.label.clone(),
                role,
                value: Some(value),
                description: format!("{}. {}.", row.scope, row.provenance),
                available: row.writable,
                focused: self.focus == GlobalPreferencesFocus::Control(row.key.clone()),
            });
            if row.changed {
                nodes.push(GlobalPreferencesAccessibleNode {
                    id: format!("{}-reset", row.key),
                    name: format!("Reset {}", row.label),
                    role: GlobalPreferencesAccessibleRole::Button,
                    value: None,
                    description: "Removes the User contribution and resolves again.".to_owned(),
                    available: row.writable,
                    focused: self.focus == GlobalPreferencesFocus::Reset(row.key.clone()),
                });
            }
            if self.explanation_key.as_deref() == Some(row.key.as_str()) {
                nodes.push(GlobalPreferencesAccessibleNode {
                    id: format!("{}-explanation", row.key),
                    name: format!("{} resolver explanation", row.label),
                    role: GlobalPreferencesAccessibleRole::Status,
                    value: Some(row.explanation_lines.join(". ")),
                    description: "Effective contribution, eligible absences, descriptor facts, and resolution reason in reading order.".to_owned(),
                    available: true,
                    focused: false,
                });
                nodes.push(GlobalPreferencesAccessibleNode {
                    id: format!("{}-explanation-close", row.key),
                    name: format!("Close {} explanation", row.label),
                    role: GlobalPreferencesAccessibleRole::Button,
                    value: None,
                    description: "Closes the resolver explanation without changing the value."
                        .to_owned(),
                    available: true,
                    focused: self.focus == GlobalPreferencesFocus::ExplanationClose,
                });
            }
        }
        if let Some(notice) = &self.notice {
            let message = match notice {
                GlobalPreferencesNoticeUi::Polite(message)
                | GlobalPreferencesNoticeUi::Assertive(message)
                | GlobalPreferencesNoticeUi::PreservedUnreadable(message) => message.clone(),
            };
            nodes.push(GlobalPreferencesAccessibleNode {
                id: "global-preferences-status".to_owned(),
                name: "Global Preferences status".to_owned(),
                role: GlobalPreferencesAccessibleRole::Status,
                value: Some(message),
                description: "Preference status announcement.".to_owned(),
                available: true,
                focused: false,
            });
        }
        nodes
    }

    pub fn advance_focus(&mut self, reverse: bool) {
        let mut order = vec![
            GlobalPreferencesFocus::SectionNavigation,
            GlobalPreferencesFocus::Search,
        ];
        for row in self.visible_rows() {
            order.push(GlobalPreferencesFocus::SettingName(row.key.clone()));
            if row.writable {
                order.push(GlobalPreferencesFocus::Control(row.key.clone()));
            }
            if row.writable
                && row.changed
                && self.open_choice_key.as_deref() != Some(row.key.as_str())
            {
                order.push(GlobalPreferencesFocus::Reset(row.key.clone()));
            }
        }
        if self.explanation_key.is_some() {
            order.push(GlobalPreferencesFocus::ExplanationClose);
        }
        order.push(GlobalPreferencesFocus::DialogClose);
        let current = order
            .iter()
            .position(|candidate| candidate == &self.focus)
            .unwrap_or(0);
        let next = if reverse {
            current.checked_sub(1).unwrap_or(order.len() - 1)
        } else {
            (current + 1) % order.len()
        };
        self.focus = order[next].clone();
    }

    pub fn dismiss_innermost(&mut self) -> GlobalPreferencesDismissal {
        if self.open_choice_key.take().is_some() {
            GlobalPreferencesDismissal::ChoiceClosed
        } else if self.explanation_key.take().is_some() {
            self.focus = GlobalPreferencesFocus::DialogClose;
            GlobalPreferencesDismissal::ExplanationClosed
        } else {
            self.open = false;
            GlobalPreferencesDismissal::DialogClosed
        }
    }
}
