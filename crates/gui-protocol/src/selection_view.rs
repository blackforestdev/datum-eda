//! Complete, revision-bound shared output. Never an authored group or writer.
mod inspection;
pub mod inspector_state;
mod net_coverage;
mod projection_inspection;
mod properties;
use crate::selection_resolution::{
    NativeSelectionResolution, SelectionAuthority, SelectionModelRevision,
};
use crate::{AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, SelectionSubject};
pub use inspection::ScopeInspection;
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionView {
    pub project: crate::SelectionProjectId,
    pub revision: SelectionModelRevision,
    pub subject: SelectionSubject,
    pub members: BTreeSet<Id>,
    pub related: BTreeSet<Id>,
    pub scope: Option<Class>,
    pub values: BTreeMap<Id, BTreeMap<String, Value>>,
    pub inspection: Vec<ScopeInspection>,
    pub zone_regions: BTreeMap<String, Vec<Vec<crate::PointNm>>>,
    pub board_keys: BTreeSet<String>,
    pub schematic_keys: BTreeSet<String>,
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
    pub fn field(&self, name: &str) -> InspectionValue {
        self.inspection
            .iter()
            .find(|scope| scope.scope == self.scope)
            .and_then(|scope| scope.fields.get(name))
            .cloned()
            .unwrap_or_else(|| {
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
    /// Exact per-member blocker detail for complete inventory/accessibility.
    pub fn unavailable_fields(&self, id: &Id) -> Vec<&'static str> {
        properties::fields(id.class)
            .iter()
            .filter_map(|(field, _)| {
                (!self
                    .values
                    .get(id)
                    .is_some_and(|values| values.contains_key(*field)))
                .then_some(*field)
            })
            .collect()
    }
    pub fn summary(&self) -> String {
        format!(
            "{} — {} objects; {} hidden; {} locked{}",
            self.subject.kind(),
            self.members.len(),
            self.hidden.len(),
            self.locked.len(),
            self.notice
                .as_ref()
                .map(|n| format!(". {n}"))
                .unwrap_or_default()
        )
    }
    /// Rendering uses uncapped projection sets, never context transport.
    pub fn selected_key(&self, scene: &crate::BoardReviewSceneV1, key: &str) -> bool {
        if scene.project_uuid != self.project.to_string() {
            return false;
        }
        if scene.scene_id.starts_with("schematic-") {
            self.schematic_keys.contains(key)
        } else {
            self.board_keys.contains(key)
        }
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
    let projection = authority.resolve(&subject)?;
    let scope = if let SelectionSubject::Object(id) = &subject {
        Some(id.class)
    } else {
        None
    };
    let mut result = SelectionView {
        project: authority.project(),
        revision: authority.revision().clone(),
        subject,
        members: projection.members,
        related: projection.related,
        scope,
        values: BTreeMap::new(),
        inspection: Vec::new(),
        zone_regions: BTreeMap::new(),
        board_keys: BTreeSet::new(),
        schematic_keys: BTreeSet::new(),
        related_board_keys: BTreeSet::new(),
        related_schematic_keys: BTreeSet::new(),
        hidden: BTreeSet::new(),
        unprojected: BTreeSet::new(),
        locked: BTreeSet::new(),
        notice: None,
        dropped: BTreeSet::new(),
        stale: false,
    };
    for zone in &board.zones {
        let Some(id) =
            crate::selection_scene::projection_candidate(board, &zone.object_id, authority)
        else {
            continue;
        };
        if !result.members.contains(&id) {
            continue;
        }
        let polygons = if let Some(run) = &projection.board_run {
            run.zone_copper
                .iter()
                .filter(|c| c.zone_id == id.id)
                .flat_map(|c| c.polygons.iter())
                .collect::<Vec<_>>()
        } else if let Some(fill) = model.zone_fills.get(&id.id).filter(|fill| {
            fill.state == eda_engine::substrate::ZoneFillState::Filled
                && fill.model_revision == model.model_revision
                && model
                    .objects
                    .get(&id.id)
                    .is_some_and(|source| fill.source_zone_revision == source.object_revision)
        }) {
            fill.islands.iter().collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        result.zone_regions.insert(
            zone.object_id.clone(),
            polygons
                .into_iter()
                .map(|p| {
                    p.vertices
                        .iter()
                        .map(|p| crate::PointNm { x: p.x, y: p.y })
                        .collect()
                })
                .collect(),
        );
    }
    let needed: BTreeSet<_> = result
        .members
        .iter()
        .filter_map(|id| model.objects.get(&id.id).map(|o| o.source_shard_id))
        .collect();
    let mut shards = BTreeMap::new();
    for shard in &model.source_shards {
        if needed.contains(&shard.shard_id)
            && let Ok(value) =
                model.materialized_source_shard_value_by_relative_path(&shard.relative_path)
        {
            shards.insert(shard.shard_id, value);
        }
    }
    let mut index = BTreeMap::new();
    for (shard, value) in &shards {
        index_sources(*shard, value, &mut index);
    }
    for id in &result.members {
        if let Some(object) = model.objects.get(&id.id)
            && let Some(value) = index.get(&(object.source_shard_id, id.id))
        {
            if value.get("locked").and_then(Value::as_bool) == Some(true) {
                result.locked.insert(id.clone());
            }
            result
                .values
                .insert(id.clone(), properties::values(id.class, value));
        }
    }
    let entry_sources: BTreeSet<_> = projection
        .electrical
        .as_ref()
        .into_iter()
        .flat_map(|e| e.owned.iter())
        .filter(|e| e.class == "bus_entries" && e.instance_path.is_empty())
        .map(|e| e.source_id.to_string())
        .collect();
    let mut rendered = BTreeSet::new();
    let mut represented = BTreeSet::new();
    for (scene, own, related) in std::iter::once((
        board,
        &mut result.board_keys,
        &mut result.related_board_keys,
    ))
    .chain(schematic.map(|scene| {
        (
            scene,
            &mut result.schematic_keys,
            &mut result.related_schematic_keys,
        )
    })) {
        for graphic in &scene.board_graphics {
            if entry_sources.contains(&graphic.source_object_uuid) {
                own.insert(graphic.object_id.clone());
            }
        }
        let bindings = crate::selection_scene::SceneSelectionBindings::capture(scene, authority);
        for (key, binding) in bindings.iter() {
            for id in [&binding.projected, &binding.acquired] {
                represented.insert(id.clone());
                if binding.visible(&visible) {
                    rendered.insert(id.clone());
                }
                if result.members.contains(id) {
                    own.insert(key.to_owned());
                }
                if result.related.contains(id) {
                    related.insert(key.to_owned());
                }
            }
        }
    }
    result.hidden = result
        .members
        .intersection(&represented)
        .filter(|id| !rendered.contains(*id))
        .cloned()
        .collect();
    result.unprojected = result.members.difference(&represented).cloned().collect();
    result.inspection = inspection::compile(&result);
    projection_inspection::append(
        &mut result,
        authority,
        projection.electrical.as_ref(),
        projection.board_run.as_ref(),
        projection.bus_run.as_ref(),
        board,
        schematic,
    );
    net_coverage::append(&mut result, model, authority);
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
