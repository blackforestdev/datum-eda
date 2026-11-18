//! Selected-asset declarations over retained common provenance, never clearance.
use super::*;
use crate::{library_authoring::LibraryField, single_line_editor::SingleLineEditor};

const CITATION_FIELDS: [(&str, &str); 5] = [
    ("title", "Title"),
    ("issuer", "Issuer · optional"),
    ("document_revision", "Source revision · optional"),
    ("locator", "Reference or location · optional"),
    ("access_note", "Access or fingerprint note · optional"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProvenanceField {
    pub row: Uuid,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvenanceAction {
    EditFields,
    Field(ProvenanceField),
    AddAttribution,
    RemoveAttribution(Uuid),
    AddCitation,
    RemoveCitation(Uuid),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributionRow {
    pub id: Uuid,
    pub field: LibraryField,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CitationRow {
    pub id: Uuid,
    pub fields: Vec<LibraryField>,
    original: Value,
}

impl CitationRow {
    pub fn retained_file(&self) -> Option<&str> {
        self.original["retained_blob"].as_str()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProvenanceForm {
    pub id: Uuid,
    pub asset_name: String,
    pub license: LibraryField,
    pub attributions: Vec<AttributionRow>,
    pub citations: Vec<CitationRow>,
    original: Value,
    selected: Value,
}

fn field(key: &'static str, label: &'static str, value: &Value) -> LibraryField {
    let text = value.as_str().unwrap_or_default().to_owned();
    LibraryField {
        key,
        label,
        editor: SingleLineEditor::at_end(&text),
        text,
        choices: vec![],
        selector: false,
    }
}

fn citation(original: Value) -> CitationRow {
    CitationRow {
        id: Uuid::new_v4(),
        fields: CITATION_FIELDS
            .iter()
            .map(|(key, label)| field(key, label, &original[key]))
            .collect(),
        original,
    }
}

impl ProvenanceForm {
    fn new(record: &Value) -> Result<Self, String> {
        let original = record["provenance"].clone();
        let credits = original["rights"]["attribution"]
            .as_array()
            .ok_or("Recorded attributions are unavailable")?;
        let sources = original["sources"]
            .as_array()
            .ok_or("Recorded citations are unavailable")?;
        Ok(Self {
            id: Uuid::new_v4(),
            asset_name: record["name"].as_str().unwrap_or("Selected asset").into(),
            license: field(
                "license_expression",
                "License declaration · blank means unknown",
                &original["rights"]["license_expression"],
            ),
            attributions: credits
                .iter()
                .map(|value| AttributionRow {
                    id: Uuid::new_v4(),
                    field: field("attribution", "Attribution", value),
                })
                .collect(),
            citations: sources.iter().cloned().map(citation).collect(),
            original,
            selected: record["revision"].clone(),
        })
    }

    pub fn field(&self, target: ProvenanceField) -> Option<&LibraryField> {
        if target.row == self.id && target.index == 0 {
            return Some(&self.license);
        }
        if target.index == 0
            && let Some(row) = self.attributions.iter().find(|r| r.id == target.row)
        {
            return Some(&row.field);
        }
        self.citations
            .iter()
            .find(|r| r.id == target.row)?
            .fields
            .get(target.index)
    }

    fn field_mut(&mut self, target: ProvenanceField) -> Option<&mut LibraryField> {
        if target.row == self.id && target.index == 0 {
            return Some(&mut self.license);
        }
        if target.index == 0
            && let Some(row) = self.attributions.iter_mut().find(|r| r.id == target.row)
        {
            return Some(&mut row.field);
        }
        self.citations
            .iter_mut()
            .find(|r| r.id == target.row)?
            .fields
            .get_mut(target.index)
    }

    pub fn actions(&self) -> Vec<ProvenanceAction> {
        use ProvenanceAction::*;
        let mut actions = vec![Field(ProvenanceField {
            row: self.id,
            index: 0,
        })];
        for row in &self.attributions {
            actions.extend([
                Field(ProvenanceField {
                    row: row.id,
                    index: 0,
                }),
                RemoveAttribution(row.id),
            ]);
        }
        actions.push(AddAttribution);
        for row in &self.citations {
            actions.extend(
                (0..row.fields.len()).map(|index| Field(ProvenanceField { row: row.id, index })),
            );
            if row.retained_file().is_none() {
                actions.push(RemoveCitation(row.id));
            }
        }
        actions.push(AddCitation);
        actions
    }

    fn edit_list(&mut self, action: ProvenanceAction) -> Result<ProvenanceAction, String> {
        if !self.actions().contains(&action) {
            return Err("Declaration row changed; use the current controls".into());
        }
        let focus_field = |row| ProvenanceAction::Field(ProvenanceField { row, index: 0 });
        Ok(match action {
            ProvenanceAction::AddAttribution => {
                let id = Uuid::new_v4();
                self.attributions.push(AttributionRow {
                    id,
                    field: field("attribution", "Attribution", &Value::Null),
                });
                focus_field(id)
            }
            ProvenanceAction::RemoveAttribution(id) => {
                let index = self
                    .attributions
                    .iter()
                    .position(|row| row.id == id)
                    .unwrap();
                self.attributions.remove(index);
                self.attributions
                    .get(index)
                    .or_else(|| self.attributions.last())
                    .map(|row| focus_field(row.id))
                    .unwrap_or(ProvenanceAction::AddAttribution)
            }
            ProvenanceAction::AddCitation => {
                let row = citation(json!({"title":"", "issuer":null, "document_revision":null,
                    "locator":null, "access_note":null, "retained_blob":null}));
                let focus = focus_field(row.id);
                self.citations.push(row);
                focus
            }
            ProvenanceAction::RemoveCitation(id) => {
                let index = self.citations.iter().position(|row| row.id == id).unwrap();
                self.citations.remove(index);
                self.citations
                    .get(index)
                    .or_else(|| self.citations.last())
                    .map(|row| focus_field(row.id))
                    .unwrap_or(ProvenanceAction::AddCitation)
            }
            _ => return Err("Select a declaration list action".into()),
        })
    }

    /// Only exposed fields change; opaque evidence and retained references stay put.
    fn value(&self) -> Result<Value, String> {
        let mut value = self.original.clone();
        let optional = |text: &str| {
            if text.trim().is_empty() {
                Value::Null
            } else {
                json!(text)
            }
        };
        value["rights"]["license_expression"] = optional(&self.license.text);
        let mut credits = Vec::new();
        for (index, row) in self.attributions.iter().enumerate() {
            if row.field.text.trim().is_empty() {
                return Err(format!("Attribution {} needs text or Remove", index + 1));
            }
            credits.push(json!(row.field.text));
        }
        value["rights"]["attribution"] = json!(credits);
        let mut sources = Vec::new();
        for (index, row) in self.citations.iter().enumerate() {
            if row.fields[0].text.trim().is_empty() {
                return Err(format!("Source citation {} needs a title", index + 1));
            }
            let mut source = row.original.clone();
            for (field_index, (key, _)) in CITATION_FIELDS.iter().enumerate() {
                source[key] = if field_index == 0 {
                    json!(row.fields[field_index].text)
                } else {
                    optional(&row.fields[field_index].text)
                };
            }
            sources.push(source);
        }
        value["sources"] = json!(sources);
        Ok(value)
    }
}

impl LibraryRevisionWorkflow {
    pub fn edit_provenance(
        browser: &LibraryBrowser,
        root: &Path,
        entry_index: usize,
    ) -> Result<Self, String> {
        let entry = browser
            .entries
            .get(entry_index)
            .filter(|e| e.editable() && e.warning.is_none())
            .ok_or("Select an available managed definition")?;
        let mut workflow = Self::capture(browser, root, Value::Null)?;
        if !workflow.managed {
            return Err("Record a managed revision before editing declarations".into());
        }
        let records = browser.revision_inspection.as_ref().unwrap()["library"]["available_assets"]
            .as_array()
            .unwrap();
        let active: Vec<_> = records
            .iter()
            .filter(|r| r["revision"]["asset_id"] == json!(entry.id) && r["active"] == true)
            .collect();
        if active.len() != 1 || active[0]["active_source_path"] != json!(entry.source_path) {
            return Err("An exact active revision from this source is required".into());
        }
        workflow.request =
            workflow.candidate_request(json!({"action":"revise", "base":active[0]["revision"]}))?;
        workflow.provenance = Some(ProvenanceForm::new(active[0])?);
        Ok(workflow)
    }

    pub fn provenance(&self) -> Option<&ProvenanceForm> {
        self.provenance.as_ref()
    }

    pub fn provenance_control(&self, action: ProvenanceAction) -> RevisionControl {
        RevisionControl::Provenance {
            session: self.session_id,
            generation: self.request_id,
            action,
        }
    }

    pub fn initial_focus_control(&self) -> RevisionControl {
        self.provenance
            .as_ref()
            .map(|form| {
                self.provenance_control(ProvenanceAction::Field(ProvenanceField {
                    row: form.id,
                    index: 0,
                }))
            })
            .unwrap_or_else(|| self.purpose_control())
    }

    pub fn provenance_field(&self, target: ProvenanceField) -> Option<&LibraryField> {
        self.provenance.as_ref()?.field(target)
    }

    pub fn provenance_edit_available(&self, browser: &LibraryBrowser, root: &Path) -> bool {
        self.preview.is_none() && self.provenance_basis_current(browser, root)
    }

    pub fn provenance_basis_current(&self, browser: &LibraryBrowser, root: &Path) -> bool {
        self.provenance.is_some() && self.qualify_source(browser, root).is_ok()
    }

    pub fn edit_provenance_list(
        &mut self,
        control: RevisionControl,
    ) -> Result<RevisionControl, String> {
        self.validate_review_control(control)?;
        if self.submitted {
            return Err("Reconcile the submitted change before editing".into());
        }
        let RevisionControl::Provenance { action, .. } = control else {
            return Err("Use declaration controls".into());
        };
        let focus = if action != ProvenanceAction::EditFields {
            self.provenance
                .as_mut()
                .ok_or("Declaration editor is closed")?
                .edit_list(action)?
        } else {
            ProvenanceAction::Field(ProvenanceField {
                row: self
                    .provenance
                    .as_ref()
                    .ok_or("Declaration editor is closed")?
                    .id,
                index: 0,
            })
        };
        self.invalidate_declaration_preview();
        Ok(self.provenance_control(focus))
    }

    pub fn set_provenance_field(
        &mut self,
        target: ProvenanceField,
        text: String,
        editor: SingleLineEditor,
    ) -> Result<(), String> {
        if self.submitted || self.preview.is_some() {
            return Err("Reopen the declaration fields before editing".into());
        }
        let field = self
            .provenance
            .as_mut()
            .and_then(|form| form.field_mut(target))
            .ok_or("Declaration row is no longer available")?;
        let changed = field.text != text;
        field.text = text;
        field.editor = editor;
        if changed {
            self.invalidate_declaration_preview();
        }
        Ok(())
    }

    fn invalidate_declaration_preview(&mut self) {
        self.preview = None;
        self.show_record_details = false;
        self.request_id = Uuid::new_v4();
    }

    pub(super) fn effective_request(&self) -> Result<Value, String> {
        let mut request = self.request.clone();
        if let Some(form) = &self.provenance {
            request["request"]["provenance_override"] = form.value()?;
        }
        Ok(request)
    }

    pub(super) fn verify_declaration_preview(&self, summary: &Value) -> Result<(), String> {
        let Some(form) = &self.provenance else {
            return Ok(());
        };
        let row = summary["asset_changes"]
            .as_array()
            .and_then(|rows| {
                rows.iter()
                    .find(|row| row["asset_id"] == form.selected["asset_id"])
            })
            .ok_or("Declaration preview lacks the selected exact asset")?;
        let review = &row["review"];
        if review["before"] != form.selected
            || review["payload_bytes_changed"] != false
            || review["payload_semantics_changed"] != false
            || !review["dependencies"].as_array().is_some_and(Vec::is_empty)
        {
            return Err("Declaration preview changed engineering content or dependencies".into());
        }
        Ok(())
    }
}

impl RevisionControl {
    pub fn is_text_field(self) -> bool {
        matches!(
            self,
            Self::Purpose(_)
                | Self::Provenance {
                    action: ProvenanceAction::Field(_),
                    ..
                }
        )
    }
}

#[cfg(test)]
#[path = "library_provenance_form_tests.rs"]
mod tests;
