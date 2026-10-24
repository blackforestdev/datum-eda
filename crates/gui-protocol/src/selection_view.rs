//! Complete, revision-bound shared output. Never an authored group or writer.
mod component_relations;
mod inspection;
mod inspector_labels;
mod inventory;
pub use inspector_labels::field_label;
pub mod inspector_state;
mod net_coverage;
mod placed_pad_inspection;
mod presentation;
mod projection_inspection;
mod properties;
mod quantities;
use crate::selection_resolution::{
    NativeSelectionResolution, SelectionAuthority, SelectionModelRevision,
};
use crate::{AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, SelectionSubject};
pub use inspection::{InspectionDetailKind, ScopeInspection};
pub use quantities::QuantityPresentation;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum InspectionValue {
    Common(Value),
    Mixed,
    Unavailable(String),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionState {
    Common,
    Mixed,
    Unavailable,
}
mod display_preview;
impl InspectionValue {
    pub fn state(&self) -> InspectionState {
        match self {
            Self::Common(_) => InspectionState::Common,
            Self::Mixed => InspectionState::Mixed,
            Self::Unavailable(_) => InspectionState::Unavailable,
        }
    }
    /// Explicit width-limited presentation; exact data remains available to all
    /// read-only query/context/accessibility consumers.
    pub fn display_preview(&self, columns: usize) -> String {
        display_preview::value(self, columns)
    }
}
/// A presentation belongs to one source occurrence, even when primitive UUIDs repeat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchematicSelectionPresentation {
    pub document: crate::schematic_documents::SchematicDocumentKey,
    pub scene_id: String,
    pub keys: BTreeSet<String>,
    pub related: BTreeSet<String>,
    pub owners: BTreeMap<String, Id>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionView {
    pub project: crate::SelectionProjectId,
    pub revision: SelectionModelRevision,
    pub subject: SelectionSubject,
    pub members: BTreeSet<Id>,
    pub related: BTreeSet<Id>,
    pub scope: Option<Class>,
    pub values: BTreeMap<Id, BTreeMap<String, Value>>,
    #[serde(default)]
    pub authored_anchors: BTreeMap<Id, crate::PointNm>,
    pub inspection: Vec<ScopeInspection>,
    pub zone_regions: BTreeMap<String, Vec<Vec<crate::PointNm>>>,
    pub board_keys: BTreeSet<String>,
    pub schematic_keys: BTreeSet<String>,
    #[serde(default)]
    pub schematic_documents: Vec<SchematicSelectionPresentation>,
    #[serde(default)]
    pub board_presentations: BTreeMap<String, Id>,
    #[serde(default)]
    pub schematic_presentations: BTreeMap<String, Id>,
    pub related_board_keys: BTreeSet<String>,
    pub related_schematic_keys: BTreeSet<String>,
    pub hidden: BTreeSet<Id>,
    pub unprojected: BTreeSet<Id>,
    pub locked: BTreeSet<Id>,
    pub notice: Option<String>,
    pub dropped: BTreeSet<Id>,
    pub stale: bool,
}
impl SelectionView {
    pub fn focus(&self) -> Option<&Id> {
        match &self.subject {
            SelectionSubject::Object(id) => Some(id),
            SelectionSubject::Compound(c) => c.focus.as_ref(),
            _ => None,
        }
    }
    pub fn scoped_members(&self) -> impl Iterator<Item = &Id> {
        self.members
            .iter()
            .filter(|id| self.scope.is_none_or(|class| class == id.class))
    }
    /// Borrow the exact compiled value; display consumers need not clone large
    /// complete arrays merely to render a bounded preview.
    pub fn field_ref(&self, name: &str) -> Option<&InspectionValue> {
        self.inspection
            .iter()
            .find(|scope| scope.scope == self.scope)
            .and_then(|scope| scope.fields.get(name))
    }
    pub(super) fn field_detail_kind(&self, name: &str) -> InspectionDetailKind {
        self.inspection
            .iter()
            .find(|scope| scope.scope == self.scope)
            .and_then(|scope| scope.field_details.get(name))
            .copied()
            .unwrap_or_default()
    }
    pub fn field_quantity_presentation(&self, name: &str) -> Option<&QuantityPresentation> {
        self.inspection
            .iter()
            .find(|scope| scope.scope == self.scope)
            .and_then(|scope| scope.quantities.get(name))
    }
    /// Read-only Project quantity formatting; canonical aggregate values stay exact.
    pub fn field_display_preview(&self, name: &str, columns: usize) -> String {
        if let Some(label) = self
            .inspection
            .iter()
            .find(|scope| scope.scope == self.scope)
            .and_then(|scope| scope.display_labels.get(name))
        {
            return display_preview::elide(label, columns);
        }
        let presentation = self.field_quantity_presentation(name);
        match presentation {
            Some(QuantityPresentation::Formatted { rendered, rounded }) => display_preview::elide(
                &format!("{rendered}{}", if *rounded { " (rounded)" } else { "" }),
                columns,
            ),
            Some(QuantityPresentation::Unavailable { reason }) => {
                display_preview::elide(&format!("Unavailable — {reason}"), columns)
            }
            None => self
                .field_ref(name)
                .map(|value| value.display_preview(columns))
                .unwrap_or_else(|| display_preview::elide("Unavailable", columns)),
        }
    }
    pub fn field(&self, name: &str) -> InspectionValue {
        self.field_ref(name).cloned().unwrap_or_else(|| {
            InspectionValue::Unavailable(
                "Property is absent or incompatible in the complete scope".into(),
            )
        })
    }
    pub fn fields(&self) -> BTreeSet<&str> {
        self.inspection
            .iter()
            .find(|scope| scope.scope == self.scope)
            .into_iter()
            .flat_map(|scope| scope.fields.keys().map(String::as_str))
            .collect()
    }
    pub fn summary(&self) -> String {
        if let SelectionSubject::Proposal(id)
        | SelectionSubject::Review(id)
        | SelectionSubject::Diagnostic(id) = &self.subject
        {
            let display_id = if matches!(self.subject, SelectionSubject::Proposal(_)) {
                serde_json::from_str::<crate::selection_artifacts::ProductionProposalActionIdentity>(id)
                    .map(|identity| identity.proposal_id.to_string())
                    .unwrap_or_else(|_| id.clone())
            } else {
                id.clone()
            };
            return format!(
                "{} — {}{}",
                self.subject.display_name(),
                display_id,
                self.notice
                    .as_ref()
                    .map(|n| format!(". {n}"))
                    .unwrap_or_default()
            );
        }
        format!(
            "{} — {} objects; {} hidden; {} locked{}",
            self.subject.display_name(),
            self.members.len(),
            self.hidden.len(),
            self.locked.len(),
            self.notice
                .as_ref()
                .map(|n| format!(". {n}"))
                .unwrap_or_default()
        )
    }
    /// Uncapped projection configured by the pane, never scene naming or context.
    pub fn projection_keys(
        &self,
        scene: &crate::BoardReviewSceneV1,
        surface: crate::PaneContent,
    ) -> Option<&BTreeSet<String>> {
        if self.stale
            || scene.project_uuid != self.project.to_string()
            || scene.source_revision != self.revision.0
        {
            return None;
        }
        match surface {
            crate::PaneContent::Board => Some(&self.board_keys),
            crate::PaneContent::Schematic => match &scene.schematic_document {
                Some(_) => self.document_presentation(scene).map(|p| &p.keys),
                None => Some(&self.schematic_keys),
            },
            crate::PaneContent::Library | crate::PaneContent::Revision(_) => None,
        }
    }
    pub fn selected_key(
        &self,
        scene: &crate::BoardReviewSceneV1,
        surface: crate::PaneContent,
        key: &str,
    ) -> bool {
        self.projection_keys(scene, surface)
            .is_some_and(|keys| keys.contains(key))
    }
}

pub fn build_selection_view(
    model: &eda_engine::substrate::DesignModel,
    authority: &NativeSelectionResolution,
    subject: SelectionSubject,
    board: &crate::BoardReviewSceneV1,
    schematic: Option<&crate::BoardReviewSceneV1>,
    visible: impl Fn(&str) -> bool,
) -> Result<SelectionView, crate::selection_resolution::SelectionResolutionError> {
    build_selection_view_for_documents(
        model,
        authority,
        subject,
        board,
        &schematic.into_iter().collect::<Vec<_>>(),
        visible,
    )
}

pub fn build_selection_view_for_documents(
    model: &eda_engine::substrate::DesignModel,
    authority: &NativeSelectionResolution,
    subject: SelectionSubject,
    board: &crate::BoardReviewSceneV1,
    schematics: &[&crate::BoardReviewSceneV1],
    visible: impl Fn(&str) -> bool,
) -> Result<SelectionView, crate::selection_resolution::SelectionResolutionError> {
    let projection = authority.resolve(&subject)?;
    let mut result = SelectionView {
        project: authority.project(),
        revision: authority.revision().clone(),
        subject,
        members: projection.members,
        related: projection.related,
        scope: None,
        values: BTreeMap::new(),
        authored_anchors: BTreeMap::new(),
        inspection: Vec::new(),
        zone_regions: BTreeMap::new(),
        board_keys: BTreeSet::new(),
        schematic_keys: BTreeSet::new(),
        schematic_documents: Vec::new(),
        board_presentations: BTreeMap::new(),
        schematic_presentations: BTreeMap::new(),
        related_board_keys: BTreeSet::new(),
        related_schematic_keys: BTreeSet::new(),
        hidden: BTreeSet::new(),
        unprojected: BTreeSet::new(),
        locked: BTreeSet::new(),
        notice: None,
        dropped: BTreeSet::new(),
        stale: false,
    };
    // Construction chooses the initial inspection scope. Publication and
    // reconciliation restore deliberate All/per-type navigation separately.
    result.scope = result.authored_singleton().map(|id| id.class);
    let zone_fills: BTreeMap<_, _> = result
        .members
        .iter()
        .filter(|id| id.class == Class::Zone)
        .map(|id| (id.clone(), crate::ZoneFillProjection::capture(model, id.id)))
        .collect();
    let zone_statuses: BTreeMap<_, _> = zone_fills
        .iter()
        .map(|(id, fill)| {
            let status = match fill {
                crate::ZoneFillProjection::Current(polygons) => {
                    InspectionValue::Common(serde_json::json!(if polygons.is_empty() {
                        "Current: no occupied copper"
                    } else {
                        "Current occupied copper"
                    }))
                }
                crate::ZoneFillProjection::Unavailable(reason) => {
                    InspectionValue::Unavailable(reason.clone())
                }
                crate::ZoneFillProjection::LegacyNominal => {
                    unreachable!("native current-fill producer")
                }
            };
            (id.clone(), status)
        })
        .collect();
    for zone in &board.zones {
        let Some(id) =
            crate::selection_scene::projection_candidate(board, &zone.object_id, authority)
        else {
            continue;
        };
        if !result.members.contains(&id) {
            continue;
        }
        let fill = zone_fills.get(&id).expect("complete native zone facts");
        let polygons = if let Some(run) = &projection.board_run {
            run.zone_copper
                .iter()
                .filter(|c| c.zone_id == id.id)
                .flat_map(|c| c.polygons.iter())
                .map(|p| {
                    p.vertices
                        .iter()
                        .map(|p| crate::PointNm { x: p.x, y: p.y })
                        .collect()
                })
                .collect()
        } else if let crate::ZoneFillProjection::Current(polygons) = fill {
            polygons.clone()
        } else {
            continue;
        };
        result.zone_regions.insert(zone.object_id.clone(), polygons);
    }
    let needed: BTreeSet<_> = result
        .members
        .iter()
        .filter_map(|id| model.objects.get(&id.id).map(|o| o.source_shard_id))
        .collect();
    let mut shards = BTreeMap::new();
    for shard in &model.source_shards {
        if needed.contains(&shard.shard_id)
            && let Ok(value) = authority
                .snapshot()
                .source_shard_value(authority.revision(), shard.shard_id)
        {
            shards.insert(shard.shard_id, value);
        }
    }
    let board_source = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == eda_engine::substrate::SourceShardKind::BoardRoot)
        .and_then(|shard| {
            authority
                .snapshot()
                .source_shard_value(authority.revision(), shard.shard_id)
                .ok()
        });
    let mut index = BTreeMap::new();
    for (shard, value) in &shards {
        index_sources(*shard, value, &mut index);
    }
    for id in &result.members {
        if let Some(object) = model.objects.get(&id.id)
            && let Some(value) = index.get(&(object.source_shard_id, id.id))
        {
            if let Some(anchor) = properties::anchor(id.class, value) {
                result.authored_anchors.insert(id.clone(), anchor);
            }
            if value.get("locked").and_then(Value::as_bool) == Some(true) {
                result.locked.insert(id.clone());
            }
            result
                .values
                .insert(id.clone(), properties::values(id.class, value));
        }
    }
    let (represented, rendered) = presentation::compile(
        &mut result,
        authority,
        projection.electrical.as_ref(),
        board,
        schematics,
        &visible,
    );
    result.hidden = result
        .members
        .intersection(&represented)
        .filter(|id| !rendered.contains(*id))
        .cloned()
        .collect();
    result.unprojected = result.members.difference(&represented).cloned().collect();
    placed_pad_inspection::append(&mut result, model, board_source);
    result.inspection = inspection::compile(&result);
    component_relations::append(&mut result, &projection.component_context);
    if let Some(scope) = result
        .inspection
        .iter_mut()
        .find(|s| s.scope == Some(Class::Zone))
    {
        let selected: Vec<_> = result
            .members
            .iter()
            .filter(|id| id.class == Class::Zone)
            .collect();
        let unavailable: Vec<_> = selected
            .iter()
            .filter_map(|id| match zone_statuses.get(*id) {
                Some(InspectionValue::Unavailable(reason)) => Some((*id, reason.as_str())),
                None => Some((*id, "no current surface projection")),
                _ => None,
            })
            .collect();
        let detail = if unavailable.is_empty() {
            InspectionDetailKind::Ordinary
        } else {
            InspectionDetailKind::Members
        };
        let status = if !unavailable.is_empty() {
            InspectionValue::Unavailable(
                unavailable
                    .iter()
                    .map(|(id, reason)| format!("{}: {reason}", id.id))
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        } else {
            let values: Vec<_> = selected
                .iter()
                .map(|id| zone_statuses.get(*id).expect("complete zone states"))
                .collect();
            if values.windows(2).all(|w| w[0] == w[1]) {
                values[0].clone()
            } else {
                InspectionValue::Mixed
            }
        };
        scope.insert_field("occupied_fill_status", status, detail);
    }

    projection_inspection::append(
        &mut result,
        authority,
        projection.electrical.as_ref(),
        projection.board_run.as_ref(),
        projection.bus_run.as_ref(),
        board,
        schematics,
    );
    net_coverage::append(&mut result, authority, board_source, model);
    quantities::append(model, &mut result.inspection);
    inspector_labels::append(&mut result.inspection, &shards);
    Ok(result)
}
pub fn scene_keys(scene: &crate::BoardReviewSceneV1) -> impl Iterator<Item = &str> {
    scene
        .components
        .iter()
        .map(|v| v.object_id.as_str())
        .chain(scene.pads.iter().map(|v| v.object_id.as_str()))
        .chain(scene.tracks.iter().map(|v| v.object_id.as_str()))
        .chain(scene.vias.iter().map(|v| v.object_id.as_str()))
        .chain(scene.zones.iter().map(|v| v.object_id.as_str()))
        .chain(scene.outline.iter().map(|v| v.object_id.as_str()))
        .chain(scene.board_graphics.iter().map(|v| v.object_id.as_str()))
        .chain(scene.board_texts.iter().map(|v| v.object_id.as_str()))
}
fn index_sources<'a>(
    shard: crate::SelectionProjectId,
    value: &'a Value,
    index: &mut BTreeMap<(crate::SelectionProjectId, crate::SelectionProjectId), &'a Value>,
) {
    match value {
        Value::Object(map) => {
            if let Some(id) = map
                .get("uuid")
                .and_then(Value::as_str)
                .and_then(|s| s.parse().ok())
            {
                index.insert((shard, id), value);
            }
            for child in map.values() {
                index_sources(shard, child, index);
            }
        }
        Value::Array(values) => {
            for child in values {
                index_sources(shard, child, index);
            }
        }
        _ => {}
    }
}
