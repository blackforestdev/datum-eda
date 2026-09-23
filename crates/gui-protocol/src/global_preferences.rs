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
    Integer {
        value: i64,
        min: i64,
        max: i64,
        step: u64,
        suffix: String,
    },
    Identity {
        value: Option<String>,
        placeholder: String,
    },
    Structured {
        value_summary: String,
        action_label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPreferenceRowUi {
    pub key: String,
    pub section_id: String,
    pub label: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub scope: String,
    pub provenance: String,
    pub explanation_lines: Vec<String>,
    pub control: GlobalPreferenceControlUi,
    pub changed: bool,
    pub writable: bool,
    pub unavailable_reason: Option<String>,
    pub reset_description: String,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalPreferencesDismissal {
    ChoiceClosed,
    ExplanationClosed,
    SearchCleared,
    DialogClosed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPreferencesDialogState {
    pub open: bool,
    pub title: String,
    pub scope: String,
    pub sections: Vec<(String, String)>,
    pub section_id: String,
    pub section_label: String,
    pub search_query: String,
    pub scroll_row: usize,
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
    SpinButton,
    TextBox,
    RadioGroup,
    RadioButton,
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
            title: "Global Preferences — Datum".to_owned(),
            scope: "Global · this device".to_owned(),
            sections: vec![("appearance".to_owned(), "Appearance".to_owned())],
            section_id: "appearance".to_owned(),
            section_label: "Appearance".to_owned(),
            search_query: String::new(),
            scroll_row: 0,
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
    pub fn project_units_default() -> Self {
        Self {
            title: "Project Preferences — Datum".to_owned(),
            scope: "Project".to_owned(),
            sections: vec![("units".to_owned(), "Units".to_owned())],
            section_id: "units".to_owned(),
            section_label: "Units".to_owned(),
            ..Self::default()
        }
    }

    pub fn visible_rows(&self) -> impl Iterator<Item = &GlobalPreferenceRowUi> {
        let query = self.search_query.trim();
        self.rows.iter().filter(move |row| {
            (query.is_empty() && row.section_id == self.section_id)
                || (!query.is_empty()
                    && (contains_ascii_case_insensitive(&row.label, query)
                        || contains_ascii_case_insensitive(&row.description, query)
                        || contains_ascii_case_insensitive(&row.key, query)
                        || row
                            .aliases
                            .iter()
                            .any(|alias| contains_ascii_case_insensitive(alias, query))
                        || match &row.control {
                            GlobalPreferenceControlUi::Boolean {
                                off_label,
                                on_label,
                                ..
                            } => [off_label, on_label]
                                .iter()
                                .any(|label| contains_ascii_case_insensitive(label, query)),
                            GlobalPreferenceControlUi::SingleChoice { choices, .. } => {
                                choices.iter().any(|(value, label)| {
                                    contains_ascii_case_insensitive(value, query)
                                        || contains_ascii_case_insensitive(label, query)
                                })
                            }
                            GlobalPreferenceControlUi::Integer { value, suffix, .. } => {
                                contains_ascii_case_insensitive(&format!("{value}{suffix}"), query)
                            }
                            GlobalPreferenceControlUi::Identity { value, .. } => value
                                .as_deref()
                                .is_some_and(|value| contains_ascii_case_insensitive(value, query)),
                            GlobalPreferenceControlUi::Structured { value_summary, .. } => {
                                contains_ascii_case_insensitive(value_summary, query)
                            }
                        }))
        })
    }

    pub fn accessibility_nodes(&self) -> Vec<GlobalPreferencesAccessibleNode> {
        if !self.open {
            return Vec::new();
        }
        let mut nodes = vec![
            GlobalPreferencesAccessibleNode {
                id: format!("{}-dialog", self.section_id),
                name: self.title.trim_end_matches(" — Datum").to_owned(),
                role: GlobalPreferencesAccessibleRole::Dialog,
                value: Some(self.scope.clone()),
                description: format!("{}. Changes save immediately.", self.scope),
                available: true,
                focused: false,
            },
            GlobalPreferencesAccessibleNode {
                id: format!("{}-navigation", self.section_id),
                name: self.section_label.clone(),
                role: GlobalPreferencesAccessibleRole::Navigation,
                value: None,
                description: format!("{} section navigation.", self.title),
                available: true,
                focused: self.focus == GlobalPreferencesFocus::SectionNavigation,
            },
            GlobalPreferencesAccessibleNode {
                id: format!("{}-search", self.section_id),
                name: format!("Search {} settings", self.section_label),
                role: GlobalPreferencesAccessibleRole::SearchBox,
                value: Some(self.search_query.clone()),
                description: "Searches labels, descriptions, stable keys, and registered aliases. Escape clears a nonempty search before a second Escape closes the window."
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
            let (role, mut value) = match &row.control {
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
                GlobalPreferenceControlUi::Integer { value, suffix, .. } => (
                    GlobalPreferencesAccessibleRole::SpinButton,
                    format!("{value}{suffix}"),
                ),
                GlobalPreferenceControlUi::Identity { value, placeholder } => (
                    GlobalPreferencesAccessibleRole::TextBox,
                    value.clone().unwrap_or_else(|| placeholder.clone()),
                ),
                GlobalPreferenceControlUi::Structured {
                    value_summary,
                    action_label: _,
                } => (
                    GlobalPreferencesAccessibleRole::Button,
                    value_summary.clone(),
                ),
            };
            if !row.writable {
                value = "Unavailable".to_owned();
            }
            nodes.push(GlobalPreferencesAccessibleNode {
                id: format!("{}-control", row.key),
                name: row.label.clone(),
                role,
                value: Some(value),
                description: format!(
                    "{}. {}.{}",
                    row.scope,
                    row.provenance,
                    row.unavailable_reason
                        .as_deref()
                        .map(|reason| format!(" {reason}."))
                        .unwrap_or_default()
                ),
                available: row.writable,
                focused: self.focus == GlobalPreferencesFocus::Control(row.key.clone()),
            });
            if row.changed {
                nodes.push(GlobalPreferencesAccessibleNode {
                    id: format!("{}-reset", row.key),
                    name: format!("Reset {}", row.label),
                    role: GlobalPreferencesAccessibleRole::Button,
                    value: None,
                    description: row.reset_description.clone(),
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
                id: format!("{}-preferences-status", self.section_id),
                name: format!("{} status", self.title.trim_end_matches(" — Datum")),
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
        let focused_key = match &self.focus {
            GlobalPreferencesFocus::SettingName(key)
            | GlobalPreferencesFocus::Control(key)
            | GlobalPreferencesFocus::Reset(key) => Some(key.clone()),
            _ => None,
        };
        if let Some(key) = focused_key {
            self.scroll_to_row(&key);
        }
    }

    pub fn reset_transient_view(&mut self) {
        self.search_query.clear();
        self.scroll_row = 0;
        self.explanation_key = None;
        self.open_choice_key = None;
        self.focus = GlobalPreferencesFocus::SectionNavigation;
        self.notice = None;
    }

    pub fn select_section(&mut self, section_id: &str) -> bool {
        let Some((id, label)) = self.sections.iter().find(|(id, _)| id == section_id) else {
            return false;
        };
        self.section_id.clone_from(id);
        self.section_label.clone_from(label);
        self.search_query.clear();
        self.scroll_row = 0;
        self.explanation_key = None;
        self.open_choice_key = None;
        self.focus = GlobalPreferencesFocus::SectionNavigation;
        true
    }

    pub fn scroll_rows(&mut self, delta: i32) -> bool {
        let maximum = self.visible_rows().count().saturating_sub(1);
        let next = if delta < 0 {
            self.scroll_row
                .saturating_add(delta.unsigned_abs() as usize)
                .min(maximum)
        } else {
            self.scroll_row.saturating_sub(delta as usize)
        };
        let changed = next != self.scroll_row;
        self.scroll_row = next;
        changed
    }

    /// Places a row at the top of the scrolling content viewport. This keeps
    /// keyboard-focused controls and expanded choices fully visible without
    /// coupling protocol state to renderer pixel dimensions.
    pub fn scroll_to_row(&mut self, key: &str) -> bool {
        let Some(index) = self.visible_rows().position(|row| row.key == key) else {
            return false;
        };
        let changed = self.scroll_row != index;
        self.scroll_row = index;
        changed
    }

    /// Toggle a choice through the shared dialog state without copying its options.
    pub fn toggle_choice(&mut self, key: &str) -> bool {
        if !self.rows.iter().any(|row| {
            row.key == key && matches!(row.control, GlobalPreferenceControlUi::SingleChoice { .. })
        }) {
            return false;
        }
        self.explanation_key = None;
        self.open_choice_key =
            (self.open_choice_key.as_deref() != Some(key)).then(|| key.to_owned());
        if self.open_choice_key.is_some() {
            self.scroll_to_row(key);
        }
        true
    }

    pub fn dismiss_innermost(&mut self) -> GlobalPreferencesDismissal {
        if self.open_choice_key.take().is_some() {
            GlobalPreferencesDismissal::ChoiceClosed
        } else if let Some(key) = self.explanation_key.take() {
            self.focus = GlobalPreferencesFocus::SettingName(key);
            GlobalPreferencesDismissal::ExplanationClosed
        } else if self.focus == GlobalPreferencesFocus::Search && !self.search_query.is_empty() {
            self.search_query.clear();
            GlobalPreferencesDismissal::SearchCleared
        } else {
            self.open = false;
            self.reset_transient_view();
            GlobalPreferencesDismissal::DialogClosed
        }
    }
}

// Preserve ASCII-only case folding without copying every searchable field.
fn contains_ascii_case_insensitive(text: &str, query: &str) -> bool {
    match query.as_bytes() {
        [] => true,
        // First-character searches should not pay for a slice comparison per byte.
        [byte] => text
            .as_bytes()
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(byte)),
        query => text
            .as_bytes()
            .windows(query.len())
            .any(|candidate| candidate.eq_ignore_ascii_case(query)),
    }
}

#[cfg(test)]
mod search_tests {
    use super::contains_ascii_case_insensitive;

    #[test]
    fn borrowed_search_preserves_ascii_and_utf8_substring_rules() {
        for (text, query, expected) in [
            ("Board Precision", "pReCiSiOn", true),
            ("Board Precision", "P", true),
            ("Board Precision", "z", false),
            ("µm display", "µM", true),
            ("ÄBC", "äbc", false),
            ("ÄBC", "Äbc", true),
            ("a", "longer", false),
            ("", "", true),
            ("µm", "m", true),
        ] {
            assert_eq!(contains_ascii_case_insensitive(text, query), expected);
        }
    }
}
