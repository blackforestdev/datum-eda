//! Disposable Library review intent; the native service remains the write owner.
use super::{LibraryBrowser, revision_view};
use crate::native_edit_client::{NativeEditAction, NativeEditIntent, NativeLibraryEdit};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[path = "library_provenance_form.rs"]
pub mod provenance;
#[path = "library_revision_summary.rs"]
mod summary_text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionControl {
    AdoptProfile(RevisionEntryBasis),
    RecordSource(RevisionEntryBasis),
    EditProvenance(RevisionEntryBasis),
    Provenance {
        session: Uuid,
        generation: Uuid,
        action: provenance::ProvenanceAction,
    },
    SaveCandidate,
    SelectUpdate {
        basis: RevisionEntryBasis,
        index: usize,
    },
    Purpose(Uuid),
    Preview {
        session: Uuid,
        request_id: Uuid,
    },
    Apply {
        session: Uuid,
        request_id: Uuid,
    },
    RecordDetails {
        session: Uuid,
        request_id: Uuid,
        show: bool,
    },
    Cancel(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevisionEntryBasis {
    inspection_id: Uuid,
    selected_asset: Option<Uuid>,
}

impl LibraryBrowser {
    fn revision_entry_basis(
        &self,
        selection: &crate::SelectionTarget,
    ) -> Option<RevisionEntryBasis> {
        if self.revision_inspection_id.is_nil() {
            return None;
        }
        let model = self.basis.as_ref()?;
        self.revision_inspection
            .as_ref()
            .filter(|description| revision_view::matches_model(description, model))?;
        Some(RevisionEntryBasis {
            inspection_id: self.revision_inspection_id,
            selected_asset: self.selected(selection).map(|entry| entry.id),
        })
    }

    /// Validate the actual displayed view before interpreting a row index.
    /// A newly qualified current snapshot cannot retarget an older visible hit.
    pub fn validate_revision_entry(
        &self,
        selection: &crate::SelectionTarget,
        control: RevisionControl,
    ) -> Result<(), String> {
        let basis = match control {
            RevisionControl::AdoptProfile(basis)
            | RevisionControl::RecordSource(basis)
            | RevisionControl::EditProvenance(basis)
            | RevisionControl::SelectUpdate { basis, .. } => basis,
            _ => return Ok(()),
        };
        if self.revision_entry_basis(selection) != Some(basis) {
            return Err(
                "Library view or selection changed; select the displayed revision again".into(),
            );
        }
        if let RevisionControl::SelectUpdate { index, .. } = control {
            let record = self.revision_inspection.as_ref().unwrap()["library"]["available_assets"]
                .as_array()
                .unwrap()
                .get(index)
                .ok_or("Displayed revision is no longer available")?;
            if record["revision"]["asset_id"] != json!(basis.selected_asset) {
                return Err("Displayed revision belongs to another definition".into());
            }
        }
        Ok(())
    }

    /// Shared entry availability for pointer rendering and keyboard navigation.
    pub fn revision_entry_controls(
        &self,
        selection: &crate::SelectionTarget,
    ) -> Vec<(String, RevisionControl)> {
        let Some(basis) = self.revision_entry_basis(selection) else {
            return vec![];
        };
        let model = self.basis.as_ref().unwrap();
        let inspection = self.revision_inspection.as_ref().unwrap();
        if model.project.schema_version != Some(2) {
            return vec![(
                "Review library format change".into(),
                RevisionControl::AdoptProfile(basis),
            )];
        }
        let Some(entry) = self.selected(selection).filter(|entry| entry.editable()) else {
            return vec![];
        };
        let records = inspection["library"]["available_assets"]
            .as_array()
            .unwrap();
        let mut controls = Vec::new();
        if records.iter().any(|r| {
            r["revision"]["asset_id"] == json!(entry.id)
                && r["active"] == true
                && r["active_source_path"] == json!(entry.source_path)
        }) {
            controls.push((
                "Edit source and license".into(),
                RevisionControl::EditProvenance(basis),
            ));
        }
        if !records
            .iter()
            .any(|record| record["revision"]["asset_id"] == json!(entry.id))
        {
            controls.push((
                "Record local source revision".into(),
                RevisionControl::RecordSource(basis),
            ));
        }
        for (index, record) in records.iter().enumerate().filter(|(_, record)| {
            record["revision"]["asset_id"] == json!(entry.id) && record["active"] == false
        }) {
            if let Some(revision) = record["revision"]["revision_id"].as_str() {
                controls.push((
                    format!("Review local revision {revision}"),
                    RevisionControl::SelectUpdate { basis, index },
                ));
            }
        }
        controls
    }
}

/// Constructed only by the client after qualifying the actual dry-run response.
/// This conveys neither permission to publish nor confirmation of a write.
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedLibraryPreview {
    pub(crate) identity: LibraryPreviewIdentity,
    pub(crate) summary: Value,
    /// Exact service-previewed request, including qualified same-pad intent.
    /// The identity above still binds the designer's original selected inputs.
    pub(crate) prepared_request: Value,
}

/// Bind a verified response to all authored inputs, not just a correlation ID.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LibraryPreviewIdentity {
    root: PathBuf,
    project_id: Uuid,
    revision: String,
    tip: Option<Uuid>,
    request_id: Uuid,
    request: Value,
    purpose: String,
}

impl LibraryPreviewIdentity {
    pub(crate) fn from_intent(intent: &NativeEditIntent) -> Option<Self> {
        if !matches!(intent.action, NativeEditAction::Library) {
            return None;
        }
        let NativeLibraryEdit::Revision { request, reason } = intent.library.as_ref()? else {
            return None;
        };
        Some(Self {
            root: intent.project_root.clone(),
            project_id: intent.project_id,
            revision: intent.expected_revision.clone(),
            tip: intent.expected_tip,
            request_id: intent.request_id,
            request: request.clone(),
            purpose: reason.trim().into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LibraryRevisionWorkflow {
    session_id: Uuid,
    purpose_editor: crate::single_line_editor::SingleLineEditor,
    root: PathBuf,
    project_id: Uuid,
    revision: String,
    tip: Option<Uuid>,
    catalog: Value,
    managed: bool,
    request: Value,
    purpose: String,
    request_id: Uuid,
    preview: Option<QualifiedLibraryPreview>,
    show_record_details: bool,
    submitted: bool,
    provenance: Option<provenance::ProvenanceForm>,
}

impl LibraryRevisionWorkflow {
    pub fn purpose_control(&self) -> RevisionControl {
        RevisionControl::Purpose(self.session_id)
    }

    pub fn preview_control(&self) -> RevisionControl {
        RevisionControl::Preview {
            session: self.session_id,
            request_id: self.request_id,
        }
    }

    /// The same displayed controls drive pointer hits and keyboard navigation.
    pub fn review_controls(&self) -> Vec<RevisionControl> {
        if self.submitted {
            return vec![];
        }
        let mut controls = vec![self.purpose_control(), self.preview_control()];
        if let Some(form) = &self.provenance {
            let actions = if self.preview.is_some() {
                vec![provenance::ProvenanceAction::EditFields]
            } else {
                form.actions()
            };
            controls.splice(
                0..0,
                actions
                    .into_iter()
                    .map(|action| self.provenance_control(action)),
            );
        }
        if self.preview.is_some() {
            if self.summary().is_some_and(|summary| {
                summary["asset_changes"]
                    .as_array()
                    .is_some_and(|assets| !assets.is_empty())
            }) {
                controls.insert(
                    1,
                    RevisionControl::RecordDetails {
                        session: self.session_id,
                        request_id: self.request_id,
                        show: !self.show_record_details,
                    },
                );
            }
            controls.push(RevisionControl::Apply {
                session: self.session_id,
                request_id: self.request_id,
            });
        }
        controls.push(RevisionControl::Cancel(self.session_id));
        controls
    }

    pub fn validate_review_control(&self, control: RevisionControl) -> Result<(), String> {
        let matches = match control {
            RevisionControl::Provenance {
                session,
                generation,
                action,
            } => {
                session == self.session_id
                    && generation == self.request_id
                    && !self.submitted
                    && self.provenance.as_ref().is_some_and(|form| {
                        if action == provenance::ProvenanceAction::EditFields {
                            self.preview.is_some()
                        } else {
                            self.preview.is_none() && form.actions().contains(&action)
                        }
                    })
            }
            RevisionControl::Purpose(session) | RevisionControl::Cancel(session) => {
                session == self.session_id
            }
            RevisionControl::Preview {
                session,
                request_id,
            }
            | RevisionControl::Apply {
                session,
                request_id,
            } => session == self.session_id && request_id == self.request_id,
            RevisionControl::RecordDetails {
                session,
                request_id,
                show,
            } => {
                session == self.session_id
                    && request_id == self.request_id
                    && show != self.show_record_details
                    && self.preview.is_some()
                    && !self.submitted
            }
            _ => false,
        };
        if !matches {
            return Err("Revision review changed; use the currently displayed controls".into());
        }
        Ok(())
    }

    fn capture(browser: &LibraryBrowser, root: &Path, request: Value) -> Result<Self, String> {
        let model = browser.basis.as_ref().ok_or("Library source unavailable")?;
        let description = browser
            .revision_inspection
            .as_ref()
            .filter(|d| revision_view::matches_model(d, model))
            .ok_or("Refresh the Library before preparing a revision")?;
        Ok(Self {
            session_id: Uuid::new_v4(),
            purpose_editor: Default::default(),
            root: root.to_owned(),
            project_id: model.project.project_id,
            revision: model.model_revision.0.clone(),
            tip: eda_engine::revision::transaction_tip(&model.journal),
            catalog: description["library"]["catalog_manifest"].clone(),
            managed: model.project.schema_version == Some(2),
            request,
            purpose: String::new(),
            request_id: Uuid::new_v4(),
            preview: None,
            show_record_details: false,
            submitted: false,
            provenance: None,
        })
    }

    /// The separate format change records no historical asset provenance.
    pub fn adopt_profile(browser: &LibraryBrowser, root: &Path) -> Result<Self, String> {
        let workflow = Self::capture(browser, root, json!({"action":"adopt_profile"}))?;
        if workflow.managed {
            return Err("This project already uses the managed library format".into());
        }
        Ok(workflow)
    }

    pub fn record_local_source(
        browser: &LibraryBrowser,
        root: &Path,
        entry_index: usize,
    ) -> Result<Self, String> {
        let entry = browser
            .entries
            .get(entry_index)
            .ok_or("Select a local definition")?;
        if !entry.editable() || entry.warning.is_some() {
            return Err("This definition is not available for local revision authoring".into());
        }
        let mut workflow = Self::capture(browser, root, Value::Null)?;
        let library = &browser.revision_inspection.as_ref().unwrap()["library"];
        if library["available_assets"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["revision"]["asset_id"] == json!(entry.id))
        {
            return Err(
                "This definition already has revision history; edit its exact revision".into(),
            );
        }
        workflow.request = workflow.candidate_request(json!({
            "action":"capture_native", "asset_id":entry.id
        }))?;
        Ok(workflow)
    }

    /// The existing kind editor supplies its engineering payload as raw JSON
    /// text. Parsing/projection/identity checks stay with the bounded service.
    pub fn save_revision(
        browser: &LibraryBrowser,
        root: &Path,
        entry_index: usize,
        payload_json: String,
    ) -> Result<Self, String> {
        let entry = browser
            .entries
            .get(entry_index)
            .ok_or("Select a local definition")?;
        if !entry.editable() || entry.warning.is_some() {
            return Err("This definition is not available for local revision authoring".into());
        }
        let mut workflow = Self::capture(browser, root, Value::Null)?;
        let records = browser.revision_inspection.as_ref().unwrap()["library"]["available_assets"]
            .as_array()
            .unwrap();
        let active: Vec<_> = records
            .iter()
            .filter(|record| {
                record["revision"]["asset_id"] == json!(entry.id) && record["active"] == true
            })
            .collect();
        if active.len() != 1 || active[0]["active_source_path"] != json!(entry.source_path) {
            return Err(
                "An exact active revision from this source is required before editing".into(),
            );
        }
        workflow.request = workflow.candidate_request(json!({
            "action":"revise", "base":active[0]["revision"], "payload_json":payload_json
        }))?;
        Ok(workflow)
    }

    /// Select one displayed exact revision. Complete retained roots are composed
    /// by the shared service, never assembled as a designer's input task.
    pub fn review_selection(
        browser: &LibraryBrowser,
        root: &Path,
        record_index: usize,
    ) -> Result<Self, String> {
        let mut workflow = Self::capture(browser, root, Value::Null)?;
        let catalog = workflow.managed_catalog()?;
        let record = browser.revision_inspection.as_ref().unwrap()["library"]["available_assets"]
            .as_array()
            .unwrap()
            .get(record_index)
            .ok_or("Select an available exact revision")?;
        if record["active"] != false {
            return Err("This exact revision is already active or unavailable".into());
        }
        workflow.request = json!({"action":"adopt_selection","request":{
            "expected_catalog":catalog,"selected":record["revision"]
        }});
        Ok(workflow)
    }

    fn managed_catalog(&self) -> Result<String, String> {
        if !self.managed {
            return Err("Review the separate project format change first".into());
        }
        self.catalog
            .as_str()
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| "Refresh the exact local catalog before continuing".into())
    }

    fn candidate_request(&self, change: Value) -> Result<Value, String> {
        Ok(json!({"action":"save_candidate", "request":{
            "expected_catalog":self.managed_catalog()?, "change":change,
            "changed_at":null, "provenance_override":null
        }}))
    }

    pub fn purpose(&self) -> &str {
        &self.purpose
    }

    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    pub fn purpose_editor(&self) -> crate::single_line_editor::SingleLineEditor {
        self.purpose_editor
    }

    pub fn set_purpose_editor(&mut self, editor: crate::single_line_editor::SingleLineEditor) {
        self.purpose_editor = editor;
    }

    pub fn action(&self) -> &str {
        self.request["action"].as_str().unwrap_or("unavailable")
    }

    /// Editing the purpose creates a new preview identity, without changing
    /// the exact selected revision or silently refreshing the captured basis.
    pub fn set_purpose(&mut self, purpose: String) -> Result<(), String> {
        if self.submitted {
            return Err("Reconcile the submitted change before continuing".into());
        }
        if self.purpose != purpose {
            self.purpose = purpose;
            self.purpose_editor =
                crate::single_line_editor::SingleLineEditor::at_end(&self.purpose);
            self.preview = None;
            self.show_record_details = false;
            self.request_id = Uuid::new_v4();
        }
        Ok(())
    }

    pub fn summary(&self) -> Option<&Value> {
        self.preview.as_ref().map(|preview| &preview.summary)
    }

    pub fn summary_lines(&self) -> Vec<String> {
        self.summary()
            .map(|summary| {
                if self.show_record_details {
                    summary_text::lines(summary)
                } else {
                    summary_text::overview(summary)
                }
            })
            .unwrap_or_default()
    }

    /// Disposable presentation only; never alters the qualified write intent.
    pub fn set_record_details(&mut self, control: RevisionControl) -> Result<(), String> {
        self.validate_review_control(control)?;
        let RevisionControl::RecordDetails { show, .. } = control else {
            return Err("Use the current recorded-details control".into());
        };
        if !self.review_controls().contains(&control) {
            return Err("Recorded details are unavailable for this preview".into());
        }
        self.show_record_details = show;
        Ok(())
    }

    pub fn is_submitted(&self) -> bool {
        self.submitted
    }

    fn qualify(&self, browser: &LibraryBrowser, root: &Path) -> Result<(), String> {
        self.qualify_source(browser, root)?;
        if self.purpose.trim().is_empty() {
            return Err("Record the purpose of this change".into());
        }
        Ok(())
    }

    fn qualify_source(&self, browser: &LibraryBrowser, root: &Path) -> Result<(), String> {
        let model = browser.basis.as_ref().ok_or("Library source unavailable")?;
        let description = browser
            .revision_inspection
            .as_ref()
            .filter(|description| revision_view::matches_model(description, model))
            .ok_or("Refresh the Library; this review no longer matches the project")?;
        if self.root != root
            || self.project_id != model.project.project_id
            || self.revision != model.model_revision.0
            || self.tip != eda_engine::revision::transaction_tip(&model.journal)
            || self.managed != (model.project.schema_version == Some(2))
            || self.catalog != description["library"]["catalog_manifest"]
        {
            return Err(
                "Project or Library changed; close this review and inspect the current revision"
                    .into(),
            );
        }
        if self.submitted {
            return Err("Reconcile the submitted change before continuing".into());
        }
        Ok(())
    }

    pub fn preview_intent(
        &self,
        browser: &LibraryBrowser,
        root: &Path,
    ) -> Result<NativeEditIntent, String> {
        self.qualify(browser, root)?;
        let mut intent = NativeEditIntent::new(
            self.root.clone(),
            self.project_id,
            self.revision.clone(),
            self.tip,
            NativeEditAction::Library,
        );
        intent.request_id = self.request_id;
        intent.library = Some(NativeLibraryEdit::Revision {
            request: self.effective_request()?,
            reason: self.purpose.trim().into(),
        });
        Ok(intent)
    }

    pub fn accept_preview(
        &mut self,
        browser: &LibraryBrowser,
        root: &Path,
        preview: QualifiedLibraryPreview,
    ) -> Result<(), String> {
        self.qualify(browser, root)?;
        let intent = self.preview_intent(browser, root)?;
        if Some(&preview.identity) != LibraryPreviewIdentity::from_intent(&intent).as_ref() {
            return Err(
                "This preview describes different inputs; preview this exact change".into(),
            );
        }
        self.verify_declaration_preview(&preview.summary)?;
        let request = self.effective_request()?;
        let original = &request;
        if preview.prepared_request != *original {
            let mut selected = preview.prepared_request.clone();
            let geometry = selected
                .get_mut("request")
                .and_then(Value::as_object_mut)
                .and_then(|request| request.remove("physical_geometry"));
            if original["action"] != "adopt_selection"
                || original["request"].get("physical_geometry").is_some()
                || selected != *original
                || !geometry.as_ref().is_some_and(|geometry| {
                    geometry["placements"]
                        .as_array()
                        .is_some_and(|v| !v.is_empty())
                })
            {
                return Err("Prepared update differs from the selected revision".into());
            }
        }
        self.preview = Some(preview);
        self.show_record_details = false;
        Ok(())
    }

    /// Mark submission before queueing. Uncertain transport and completed
    /// writes require reconciliation, never a reset or automatic retry.
    pub fn submit(
        &mut self,
        browser: &LibraryBrowser,
        root: &Path,
    ) -> Result<NativeEditIntent, String> {
        let mut intent = self.preview_intent(browser, root)?;
        let preview = self
            .preview
            .as_ref()
            .ok_or("Preview this exact change before applying")?;
        if Some(&preview.identity) != LibraryPreviewIdentity::from_intent(&intent).as_ref() {
            return Err("Preview this exact change before applying".into());
        }
        intent.library = Some(NativeLibraryEdit::Revision {
            request: preview.prepared_request.clone(),
            reason: self.purpose.trim().into(),
        });
        self.submitted = true;
        Ok(intent)
    }

    /// Only the writer owner's synchronous failure to start a worker may call
    /// this. No request reached transport. Discard the old preview so even this
    /// confirmed non-submission requires a fresh, explicit review before Apply.
    /// Worker/transport errors must never use this path.
    pub fn reject_unqueued_submission(&mut self, request_id: Uuid) -> bool {
        if !self.submitted || self.request_id != request_id {
            return false;
        }
        self.submitted = false;
        self.preview = None;
        self.show_record_details = false;
        self.request_id = Uuid::new_v4();
        true
    }
}

#[cfg(test)]
#[path = "library_revision_workflow/tests.rs"]
mod tests;
