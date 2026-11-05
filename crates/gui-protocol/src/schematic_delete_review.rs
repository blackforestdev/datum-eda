//! Shared frozen human review, configured by purpose. Proposal owns mutation.
use crate::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity, AuthoringViewBasis,
    BoardReviewSceneV1, PaneContent, ReviewWorkspaceState, SelectionTarget,
};
use eda_engine::{
    api::native_write::schematic_delete_proposal::{
        DeleteClass, DeleteTarget, NativeProposalAction, NativeProposalResponse,
    },
    schematic::Sheet,
    substrate::ProposalStatus,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteReview {
    pub child_sheet: Option<super::child_sheet::ChildSheetEvidence>,
    pub gate_placement: Option<super::gate_placement::GatePlacementEvidence>,
    pub annotation: Option<super::annotation::AnnotationEvidence>,
    pub fields: Option<super::field_table::FieldTableDraft>,
    pub disposition: Option<super::disposition::DispositionEvidence>,
    pub assignment: Option<super::assignment::AssignmentEvidence>,
    pub basis: AuthoringViewBasis,
    pub source: Sheet,
    pub scene: BoardReviewSceneV1,
    pub targets: Vec<DeleteTarget>,
    pub proposal_id: Uuid,
    pub revision: String,
    pub tip: Option<Uuid>,
    pub response: Option<NativeProposalResponse>,
    pub pending: Option<NativeProposalAction>,
    pub refusal: Option<String>,
}
impl DeleteReview {
    pub fn capture(state: &ReviewWorkspaceState) -> Result<Self, String> {
        let mut review = Self::capture_source(state)?;
        let basis = &review.basis;
        let SelectionTarget::Native(view) = &state.selection else {
            return Err("Select schematic objects first".into());
        };
        if view.stale || view.project != basis.project || view.revision.0 != basis.revision {
            return Err("Selection changed; select the current objects again".into());
        }
        if view.members.len() == 1 {
            let target = view.members.iter().next().unwrap();
            if !Self::singleton_requires_review(state, target)? {
                return Err("Unbound single objects use the ordinary Delete action".into());
            }
        }
        let source = &review.source;
        let targets = targets(
            source,
            &view.members.iter().cloned().collect::<Vec<_>>(),
            &basis
                .document
                .as_ref()
                .ok_or("Qualified sheet unavailable")?
                .instance_path,
        )?;
        let proposal_id = review.proposal_id;
        let action = NativeProposalAction::CreateDelete {
            proposal_id,
            sheet: basis.source,
            instance_path: basis.document.as_ref().unwrap().instance_path.clone(),
            targets: targets.clone(),
        };
        review.targets = targets;
        review.pending = Some(action);
        Ok(review)
    }
    /// Invocation-only canonical ownership capture, never menu or paint readiness.
    /// Unsupported owners refuse here rather than falling through to direct Delete.
    pub fn singleton_requires_review(
        state: &ReviewWorkspaceState,
        target: &AuthoredSelectionIdentity,
    ) -> Result<bool, String> {
        if target.class != Class::Symbol {
            return Ok(false);
        }
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Selected source model unavailable")?;
        if model.project.project_id.to_string() != state.scene.project_uuid
            || model.model_revision.0 != state.scene.source_revision
        {
            return Err("Selected source changed; select the current Symbol again".into());
        }
        eda_engine::api::native_write::schematic_delete_proposal::bound_symbol_owner(
            model,
            target.id,
            &target.instance_path,
        )
        .map(|owner| owner.is_some())
        .map_err(|error| error.to_string())
    }
    pub fn capture_disposition(
        state: &ReviewWorkspaceState,
        kind: eda_engine::api::native_write::schematic_delete_proposal::DispositionKind,
    ) -> Result<Self, String> {
        let evidence = super::disposition::DispositionEvidence::capture(state, kind)?;
        let mut review = Self::capture_source(state)?;
        review.disposition = Some(evidence);
        Ok(review)
    }
    pub fn capture_annotation(state: &ReviewWorkspaceState) -> Result<Option<Self>, String> {
        super::annotation::AnnotationEvidence::admit(state)?;
        let mut review = Self::capture_source(state)?;
        let evidence = super::annotation::AnnotationEvidence::capture(state, &review)?;
        if evidence.summary.changes.is_empty() {
            return Ok(None);
        }
        review.pending = Some(NativeProposalAction::CreateAnnotation {
            proposal_id: review.proposal_id,
            sheet: review.basis.source,
            instance_path: evidence.summary.instance_path.clone(),
        });
        review.annotation = Some(evidence);
        Ok(Some(review))
    }
    pub fn capture_assignment(state: &ReviewWorkspaceState) -> Result<Self, String> {
        let evidence = super::assignment::AssignmentEvidence::selected(state)?;
        let mut review = Self::capture_source(state)?;
        review.assignment = Some(evidence);
        Ok(review)
    }
    pub fn stage_assignment(&mut self, state: &ReviewWorkspaceState) -> Result<(), String> {
        if !self.current(state) {
            return Err("Project or selected source changed; cancel and reopen assignment".into());
        }
        self.pending = Some(
            self.assignment
                .as_ref()
                .ok_or("Assignment choices unavailable")?
                .action(self.proposal_id)?,
        );
        Ok(())
    }
    pub(super) fn capture_source(state: &ReviewWorkspaceState) -> Result<Self, String> {
        let basis = AuthoringViewBasis::capture(state, state.ui.layout.focused)?;
        Self::capture_source_basis(state, basis)
    }
    pub(super) fn capture_source_basis(
        state: &ReviewWorkspaceState,
        basis: AuthoringViewBasis,
    ) -> Result<Self, String> {
        if basis.content != PaneContent::Schematic {
            return Err("Focus a native schematic view for this Proposal".into());
        }
        let source = state
            .schematic_authoring
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&basis.source))
            .ok_or("Source sheet unavailable")?
            .clone();
        let scene = crate::camera_scene_for_pane(state, basis.pane)
            .ok_or("Source view unavailable")?
            .clone();
        Ok(Self {
            gate_placement: None,
            child_sheet: None,
            annotation: None,
            fields: None,
            disposition: None,
            assignment: None,
            revision: basis.revision.clone(),
            basis,
            source,
            scene,
            targets: vec![],
            proposal_id: Uuid::new_v4(),
            response: None,
            pending: None,
            refusal: None,
            tip: state
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|t| t.parse().ok()),
        })
    }
    pub fn stage_disposition(
        &mut self,
        state: &ReviewWorkspaceState,
        rationale: &str,
    ) -> Result<(), String> {
        if !self.current(state) {
            return Err("Project or finding changed; cancel and select the current finding".into());
        }
        let evidence = self
            .disposition
            .as_mut()
            .ok_or("Disposition evidence unavailable")?;
        self.pending = Some(evidence.action(self.proposal_id, rationale)?);
        Ok(())
    }
    /// Metadata revisions update this envelope; selected ids and design source never change.
    pub fn observe(&mut self, response: NativeProposalResponse) -> Result<(), String> {
        if response.proposal_id != self.proposal_id
            || response.prepared_against != self.basis.revision
        {
            return Err("Proposal identity changed".into());
        }
        if self
            .targets
            .iter()
            .any(|target| !response.deleted.iter().any(|d| d.id == target.id))
        {
            return Err("Proposal omitted part of the selected deletion; dismiss and retry".into());
        }
        match (&self.disposition, &response.disposition) {
            (Some(evidence), Some(summary))
                if evidence.matches(summary) && response.deleted.is_empty() => {}
            (None, None) => {}
            _ => return Err("Proposal disposition identity or rationale changed".into()),
        }
        match (&self.assignment, &response.assignment) {
            (Some(evidence), Some(summary))
                if evidence.matches(summary)
                    && response.deleted.is_empty()
                    && self
                        .response
                        .as_ref()
                        .and_then(|r| r.assignment.as_ref())
                        .is_none_or(|previous| {
                            previous.component_instance == summary.component_instance
                        }) => {}
            // A confirmed cancellation can outlive its source Symbol/library.
            // Frozen identity above still qualifies it; missing data never grants apply.
            (Some(_), None)
                if response.status == ProposalStatus::Rejected && response.deleted.is_empty() => {}
            (None, None) => {}
            _ => return Err("Proposal assignment identity changed".into()),
        }
        match (&self.annotation, &response.annotation) {
            (Some(evidence), Some(summary))
                if evidence.summary == *summary && response.deleted.is_empty() => {}
            (Some(_), None)
                if response.status == ProposalStatus::Rejected && response.deleted.is_empty() => {}
            (None, None) => {}
            _ => return Err("Proposal annotation changed; dismiss and review again".into()),
        }
        match (&self.fields, &response.fields) {
            (Some(table), Some(summary))
                if table.summary.as_ref() == Some(summary) && response.deleted.is_empty() => {}
            (Some(table), None)
                if table.project_annotation.is_some() && response.deleted.is_empty() => {}
            (Some(_), None)
                if response.status == ProposalStatus::Rejected && response.deleted.is_empty() => {}
            (None, None) => {}
            _ => return Err("Proposal field changes differ from the reviewed table".into()),
        }
        match (
            self.fields
                .as_ref()
                .and_then(|t| t.project_annotation.as_ref()),
            &response.project_annotation,
        ) {
            (Some(expected), Some(summary))
                if expected == summary && response.deleted.is_empty() => {}
            (Some(_), None)
                if response.status == ProposalStatus::Rejected && response.deleted.is_empty() => {}
            (None, None) => {}
            _ => {
                return Err(
                    "Project annotation differs from the complete reviewed sheet census".into(),
                );
            }
        }
        match (&self.child_sheet, &response.child_sheet) {
            (Some(evidence), Some(summary))
                if evidence.summary == *summary && response.deleted.is_empty() => {}
            (Some(_), None)
                if response.status == ProposalStatus::Rejected && response.deleted.is_empty() => {}
            (None, None) => {}
            _ => {
                return Err(
                    "Child source, definition, parent or placement differs from review".into(),
                );
            }
        }
        match (&self.gate_placement, &response.gate_placement) {
            (Some(evidence), Some(summary))
                if evidence.summary == *summary && response.deleted.is_empty() => {}
            (Some(_), None)
                if response.status == ProposalStatus::Rejected && response.deleted.is_empty() => {}
            (None, None) => {}
            _ => return Err("Proposal Gate placement identity or pose changed".into()),
        }
        self.revision = response.current_model_revision.clone();
        self.tip = response.transaction_tip;
        self.response = Some(response);
        self.pending = None;
        self.refusal = None;
        Ok(())
    }
    pub fn source_current(&self, state: &ReviewWorkspaceState) -> bool {
        state.scene.project_uuid == self.basis.project.to_string()
            && state.scene.source_revision == self.revision
            && state
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|t| t.parse().ok())
                == self.tip
            && state
                .schematic_authoring
                .source
                .as_ref()
                .and_then(|s| s.sheets.get(&self.basis.source))
                == Some(&self.source)
    }
    pub fn current(&self, state: &ReviewWorkspaceState) -> bool {
        self.source_current(state)
            && self.disposition.as_ref().is_none_or(|d| d.current(state))
            && state.ui.layout.focused == self.basis.pane
            && state.schematic_documents.panes.get(&self.basis.pane) == self.basis.document.as_ref()
    }
    pub fn accepted(&self) -> bool {
        self.response
            .as_ref()
            .is_some_and(|r| r.status == ProposalStatus::Accepted)
    }
    pub fn ready(&self) -> bool {
        self.pending.is_none()
            && (self.accepted()
                || (self.refusal.is_none()
                    && self
                        .response
                        .as_ref()
                        .is_some_and(|r| r.status == ProposalStatus::Draft)))
    }
    pub fn closes_locally(&self) -> bool {
        self.response.is_none() || self.accepted() || self.finished()
    }
    pub fn stage(&mut self, accept: bool) -> Result<(), String> {
        if self.pending.is_some() {
            return Err("Wait for the pending Proposal operation".into());
        }
        if !accept && self.closes_locally() {
            return Err("Close this Proposal review locally; it cannot be rejected".into());
        }
        if accept && !self.ready() {
            return Err("This Proposal is not ready to accept".into());
        }
        self.pending = Some(if accept {
            NativeProposalAction::AcceptApply {
                proposal_id: self.proposal_id,
            }
        } else {
            NativeProposalAction::Reject {
                proposal_id: self.proposal_id,
            }
        });
        self.refusal = None;
        Ok(())
    }
    pub fn finished(&self) -> bool {
        self.response
            .as_ref()
            .is_some_and(|r| matches!(r.status, ProposalStatus::Applied | ProposalStatus::Rejected))
    }
    pub fn deleted_sources(&self) -> std::collections::BTreeSet<String> {
        let Some(response) = &self.response else {
            return Default::default();
        };
        let mut ids: std::collections::BTreeSet<_> =
            response.deleted.iter().map(|d| d.id.to_string()).collect();
        for symbol in self
            .source
            .symbols
            .values()
            .filter(|s| response.deleted.iter().any(|d| d.id == s.uuid))
        {
            ids.extend(symbol.pins.iter().map(|pin| pin.uuid.to_string()));
        }
        ids
    }
    /// Display-only replacement; the source Sheet and exact frozen targets remain intact.
    pub fn preview_scene(&self) -> BoardReviewSceneV1 {
        // New Gate geometry is painted by the existing proposed channel, so it
        // cannot acquire authored hit targets or ignore Proposed visibility.
        let mut scene = if let Some(evidence) = &self.annotation {
            evidence.preview.clone()
        } else {
            self.scene.clone()
        };
        let ids = self.deleted_sources();
        let text_keys: std::collections::BTreeSet<_> = scene
            .board_texts
            .iter()
            .filter(|t| {
                t.source_object_uuid
                    .as_ref()
                    .is_some_and(|id| ids.contains(id))
            })
            .map(|t| t.object_id.clone())
            .collect();
        scene
            .board_graphics
            .retain(|g| !ids.contains(&g.source_object_uuid));
        scene
            .board_texts
            .retain(|t| !text_keys.contains(&t.object_id));
        scene
            .board_text_geometries
            .retain(|t| !text_keys.contains(&t.object_id));
        scene.source_revision = self.revision.clone();
        scene
    }
    pub fn summary(&self) -> String {
        if let Some(child) = &self.child_sheet {
            return format!("Create blank child sheet · {}", child.summary.name);
        }

        if let Some(table) = &self.fields {
            if let Some(summary) = &table.project_annotation {
                return format!(
                    "Annotate Project · {} sheets · {} references",
                    summary.sheets.len(),
                    summary.change_count()
                );
            }
            return format!(
                "Symbol Fields Table · {} changes",
                table.summary.as_ref().map_or(0, |s| s.changes.len())
            );
        }
        if let Some(evidence) = &self.gate_placement {
            return format!("Place another Gate · {}", evidence.summary.reference);
        }
        if let Some(evidence) = &self.annotation {
            return format!(
                "Annotate this sheet · {} of {} references change",
                evidence.summary.changes.len(),
                evidence.summary.symbol_count
            );
        }
        if let Some(evidence) = &self.assignment {
            return evidence.title();
        }
        if let Some(evidence) = &self.disposition {
            return format!("{} · {}", evidence.label(), evidence.rule_id);
        }
        let Some(response) = &self.response else {
            return format!(
                "Review deletion of {} selected objects…",
                self.targets.len()
            );
        };
        let mut classes = std::collections::BTreeMap::<String, usize>::new();
        for object in &response.deleted {
            *classes
                .entry(class_label(&object.class).into())
                .or_default() += 1;
        }
        format!(
            "Delete {} selected · {}",
            self.targets.len(),
            classes
                .into_iter()
                .map(|(kind, n)| format!("{n} {kind}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    pub fn names(&self) -> String {
        if let Some(child) = &self.child_sheet {
            return format!(
                "Parent: {} · new source + definition + placed instance",
                child.summary.parent_name
            );
        }

        if let Some(evidence) = &self.gate_placement {
            return format!(
                "{} · {}",
                evidence.summary.choice.gate_name, evidence.summary.choice.symbol_name
            );
        }
        if let Some(evidence) = &self.annotation {
            return evidence
                .summary
                .changes
                .iter()
                .map(|change| format!("{} -> {}", change.previous_reference, change.reference))
                .collect::<Vec<_>>()
                .join(", ");
        }
        if let Some(evidence) = &self.assignment {
            return format!(
                "{} · {} · {}",
                evidence.part_label(),
                evidence.gate_label(),
                evidence
                    .options
                    .members
                    .iter()
                    .map(|member| member.reference.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if let Some(evidence) = &self.disposition {
            return format!("{} · {}", evidence.message, evidence.rationale);
        }
        self.response
            .as_ref()
            .map(|r| {
                r.deleted
                    .iter()
                    .filter(|d| {
                        matches!(
                            d.class.as_str(),
                            "symbol"
                                | "symbols"
                                | "label"
                                | "labels"
                                | "port"
                                | "ports"
                                | "bus"
                                | "buses"
                                | "bus_section"
                                | "schematic_text"
                                | "texts"
                        )
                    })
                    .map(|d| d.name.clone())
                    .filter(|name| !name.is_empty())
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    }
}

fn targets(
    sheet: &Sheet,
    ids: &[AuthoredSelectionIdentity],
    path: &[Uuid],
) -> Result<Vec<DeleteTarget>, String> {
    if ids.is_empty() {
        return Err("Select complete schematic objects to delete".into());
    }
    ids.iter()
        .map(|id| {
            if id.instance_path != path {
                return Err("Selection spans schematic occurrences; nothing was omitted".into());
            }
            let (class, present) = match id.class {
                Class::SchematicText => {
                    (DeleteClass::SchematicText, sheet.texts.contains_key(&id.id))
                }
                Class::Symbol => (DeleteClass::Symbol, sheet.symbols.contains_key(&id.id)),
                Class::Wire => (DeleteClass::Wire, sheet.wires.contains_key(&id.id)),
                Class::Label => (DeleteClass::Label, sheet.labels.contains_key(&id.id)),
                Class::Port => (DeleteClass::Port, sheet.ports.contains_key(&id.id)),
                Class::BusSection => (DeleteClass::BusSection, sheet.buses.contains_key(&id.id)),
                Class::Junction => (DeleteClass::Junction, sheet.junctions.contains_key(&id.id)),
                Class::NoConnect => (
                    DeleteClass::NoConnect,
                    sheet.noconnects.contains_key(&id.id),
                ),
                _ => {
                    return Err(
                        "Selection contains an unsupported object; nothing was omitted".into(),
                    );
                }
            };
            if !present {
                return Err("Selection spans source sheets or changed; nothing was omitted".into());
            }
            Ok(DeleteTarget { class, id: id.id })
        })
        .collect()
}

fn class_label(class: &str) -> &str {
    match class {
        "schematic_text" | "texts" => "Text notes",
        "symbol" | "symbols" => "Symbols",
        "component_instance" => "Component bindings",
        "wire" | "wires" => "Wires",
        "label" | "labels" => "Labels",
        "port" | "ports" => "Ports",
        "bus_section" | "bus" | "buses" => "Buses",
        "junction" | "junctions" => "Junctions",
        "no_connect" | "noconnects" => "No-Connects",
        "bus_entry" | "bus_entries" => "Bus entries",
        _ => "objects",
    }
}
