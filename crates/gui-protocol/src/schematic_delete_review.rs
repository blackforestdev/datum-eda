//! Frozen human review of a complete schematic selection; Proposal owns mutation.
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
        let basis = AuthoringViewBasis::capture(state, state.ui.layout.focused)?;
        if basis.content != PaneContent::Schematic {
            return Err("Focus a schematic selection to delete".into());
        }
        let SelectionTarget::Native(view) = &state.selection else {
            return Err("Select schematic objects first".into());
        };
        if view.stale || view.project != basis.project || view.revision.0 != basis.revision {
            return Err("Selection changed; select the current objects again".into());
        }
        let source = state
            .schematic_authoring
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&basis.source))
            .ok_or("Selected source unavailable")?;
        let targets = targets(
            source,
            &view.members.iter().cloned().collect::<Vec<_>>(),
            &basis
                .document
                .as_ref()
                .ok_or("Qualified sheet unavailable")?
                .instance_path,
        )?;
        let scene = crate::camera_scene_for_pane(state, basis.pane)
            .ok_or("Selected view unavailable")?
            .clone();
        let proposal_id = Uuid::new_v4();
        let action = NativeProposalAction::CreateDelete {
            proposal_id,
            sheet: basis.source,
            instance_path: basis.document.as_ref().unwrap().instance_path.clone(),
            targets: targets.clone(),
        };
        Ok(Self {
            revision: basis.revision.clone(),
            basis,
            source: source.clone(),
            scene,
            targets,
            proposal_id,
            tip: state
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|t| t.parse().ok()),
            response: None,
            pending: Some(action),
            refusal: None,
        })
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
            return Err("This deletion is not ready to accept".into());
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
        let mut scene = self.scene.clone();
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
    if ids.len() < 2 {
        return Err("Batch Delete requires at least two complete selected objects".into());
    }
    ids.iter()
        .map(|id| {
            if id.instance_path != path {
                return Err("Selection spans schematic occurrences; nothing was omitted".into());
            }
            let (class, present) = match id.class {
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
        "symbol" | "symbols" => "Symbols",
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
