//! Projection of producing finding evidence, never authored acquisition authority.
use super::{CheckFindingSummary, CheckRunReviewState};
use crate::{BoardReviewSceneV1, PointNm};
use std::collections::BTreeMap;

/// One source projection index per pane. Fingerprints remain the sole selected
/// diagnostic identities. No scene display key becomes an authored identity.
pub struct MarkerProjectionIndex {
    current: bool,
    schematic: bool,
    fingerprints: BTreeMap<String, usize>,
    targets: BTreeMap<uuid::Uuid, Result<PointNm, &'static str>>,
}
impl MarkerProjectionIndex {
    pub fn capture(
        scene: &BoardReviewSceneV1,
        checks: &CheckRunReviewState,
        surface: crate::PaneContent,
        visible: impl Fn(&str) -> bool,
    ) -> Self {
        let current = checks.project_id.as_deref() == Some(scene.project_uuid.as_str())
            && checks.model_revision.as_deref() == Some(scene.source_revision.as_str())
            && matches!(
                surface,
                crate::PaneContent::Board | crate::PaneContent::Schematic
            );
        let mut fingerprints = BTreeMap::new();
        for finding in &checks.findings {
            *fingerprints.entry(finding.fingerprint.clone()).or_insert(0) += 1;
        }
        let mut targets = BTreeMap::new();
        let mut add = |source: &str, point: Option<PointNm>, layers: &[&str]| {
            if let Ok(uuid) = source.parse::<uuid::Uuid>() {
                let point = point
                    .filter(|_| layers.iter().any(|layer| visible(layer)))
                    .ok_or("Target is hidden or has no current navigation anchor");
                targets
                    .entry(uuid)
                    .and_modify(|value| *value = Err("Target has ambiguous scene projections"))
                    .or_insert(point);
            }
        };
        for v in &scene.components {
            add(
                &v.source_object_uuid,
                Some(v.position),
                &[&v.placement_layer],
            );
        }
        for v in &scene.pads {
            add(
                &v.source_object_uuid,
                v.center.integer_point(),
                &v.copper_layer_ids
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            );
        }
        for v in &scene.tracks {
            add(
                &v.source_object_uuid,
                v.path.first().copied(),
                &[&v.layer_id],
            );
        }
        for v in &scene.vias {
            add(
                &v.source_object_uuid,
                Some(v.position),
                &[&v.start_layer_id, &v.end_layer_id],
            );
        }
        for v in &scene.zones {
            add(
                &v.source_object_uuid,
                v.polygon.first().copied(),
                &[&v.layer_id],
            );
        }
        for v in &scene.board_graphics {
            add(
                &v.source_object_uuid,
                v.path.first().copied(),
                &[&v.layer_id],
            );
        }
        for v in &scene.board_texts {
            add(
                v.source_object_uuid.as_deref().unwrap_or(&v.text_uuid),
                Some(v.position),
                &[&v.layer_id],
            );
        }

        Self {
            current,
            schematic: matches!(surface, crate::PaneContent::Schematic),
            fingerprints,
            targets,
        }
    }
    /// Exact producing location first, otherwise a unique root-scene navigation
    /// anchor. Non-root frames are unavailable until projected as occurrences.
    pub fn anchor(&self, finding: &CheckFindingSummary) -> Result<PointNm, &'static str> {
        if !self.current {
            return Err("Finding and scene project/revision differ");
        }
        if finding.fingerprint.is_empty() || self.fingerprints.get(&finding.fingerprint) != Some(&1)
        {
            return Err("Finding fingerprint is unavailable or ambiguous");
        }
        if !root_frame(&finding.primary_target) || !root_frame(&finding.payload) {
            return Err("Finding occurrence frame is not projected by this root scene");
        }
        if (self.schematic && finding.domain != "erc")
            || (!self.schematic && finding.domain != "drc")
        {
            return Err("Finding domain has no marker on this surface");
        }
        if !self.schematic
            && let Some(location) = finding.payload.get("location").filter(|l| !l.is_null())
        {
            return Ok(PointNm {
                x: location
                    .get("x_nm")
                    .and_then(serde_json::Value::as_i64)
                    .ok_or("Producing location is not an exact nm point")?,
                y: location
                    .get("y_nm")
                    .and_then(serde_json::Value::as_i64)
                    .ok_or("Producing location is not an exact nm point")?,
            });
        }
        let uuid = finding
            .primary_target
            .get("id")
            .and_then(serde_json::Value::as_str)
            .and_then(|s| s.parse().ok())
            .ok_or("Finding has no source UUID navigation target")?;
        self.targets
            .get(&uuid)
            .copied()
            .unwrap_or(Err("Finding target is not projected on this surface"))
    }
}
fn root_frame(value: &serde_json::Value) -> bool {
    for key in ["instance_path", "occurrence_path"] {
        if let Some(path) = value.get(key)
            && !path.as_array().is_some_and(Vec::is_empty)
        {
            return false;
        }
    }
    value.get("occurrence").is_none_or(root_frame)
}
