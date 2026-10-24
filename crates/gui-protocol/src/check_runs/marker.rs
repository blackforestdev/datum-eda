//! Projection of producing finding evidence, never authored acquisition authority.
use super::{CheckFindingSummary, CheckRunReviewState};
use crate::schematic_documents::{SchematicDocumentKey, qualify_source_occurrence};
use crate::{BoardReviewSceneV1, PointNm};
use std::collections::BTreeMap;
use uuid::Uuid;

/// One source projection index per pane. Fingerprints remain the sole selected
/// diagnostic identities. No scene display key becomes an authored identity.
pub struct MarkerProjectionIndex {
    current: bool,
    schematic: bool,
    fingerprints: BTreeMap<String, usize>,
    targets: BTreeMap<uuid::Uuid, Result<PointNm, &'static str>>,
    document: Option<SchematicDocumentKey>,
    occurrences: Option<BTreeMap<Uuid, Vec<SchematicDocumentKey>>>,
}
impl MarkerProjectionIndex {
    pub fn capture(
        scene: &BoardReviewSceneV1,
        checks: &CheckRunReviewState,
        surface: crate::PaneContent,
        visible: impl Fn(&str) -> bool,
    ) -> Self {
        Self::capture_qualified(scene, checks, surface, visible, None)
    }
    pub fn capture_qualified(
        scene: &BoardReviewSceneV1,
        checks: &CheckRunReviewState,
        surface: crate::PaneContent,
        visible: impl Fn(&str) -> bool,
        documents: Option<&crate::schematic_documents::SchematicDocuments>,
    ) -> Self {
        let documents = documents.filter(|documents| documents.project.is_some());
        let current = checks.project_id.as_deref() == Some(scene.project_uuid.as_str())
            && (documents.is_none() || checks.persisted)
            && checks.model_revision.as_deref() == Some(scene.source_revision.as_str())
            && matches!(
                surface,
                crate::PaneContent::Board | crate::PaneContent::Schematic
            )
            && (surface != crate::PaneContent::Schematic
                || match (scene.schematic_document.as_ref(), documents) {
                    (Some(key), Some(documents)) => {
                        documents.project == Some(key.project)
                            && key.project.to_string() == scene.project_uuid
                            && documents
                                .revision
                                .as_ref()
                                .is_some_and(|revision| revision.0 == scene.source_revision)
                            && documents.row(key).is_some()
                    }
                    (None, None) => true, // Existing unqualified legacy root projection.
                    _ => false,
                });
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
        if surface == crate::PaneContent::Schematic
            && let Some(sheet) = scene
                .schematic_document
                .as_ref()
                .and_then(|key| documents?.source_sheet(key))
        {
            // Pin lines, terminal circles and labels share one canonical Pin.
            // Their render multiplicity must not manufacture source ambiguity.
            let mut pins = BTreeMap::new();
            for pin in sheet.symbols.values().flat_map(|symbol| &symbol.pins) {
                pins.entry(pin.uuid)
                    .and_modify(|point| *point = None)
                    .or_insert(Some(PointNm {
                        x: pin.position.x,
                        y: pin.position.y,
                    }));
            }
            for (id, point) in pins {
                let source = id.to_string();
                let other_source = sheet.uuid == id
                    || sheet.symbols.contains_key(&id)
                    || sheet.wires.contains_key(&id)
                    || sheet.junctions.contains_key(&id)
                    || sheet.labels.contains_key(&id)
                    || sheet.buses.contains_key(&id)
                    || sheet.bus_entries.contains_key(&id)
                    || sheet.ports.contains_key(&id)
                    || sheet.noconnects.contains_key(&id)
                    || sheet.texts.contains_key(&id)
                    || sheet.drawings.contains_key(&id);
                let foreign_primitive = scene.board_graphics.iter().any(|graphic| {
                    graphic.source_object_uuid == source && graphic.object_kind != "schematic_pin"
                });
                let anchor = if point.is_none() || other_source || foreign_primitive {
                    Err("Target has ambiguous source identities")
                } else if scene.board_graphics.iter().any(|graphic| {
                    graphic.source_object_uuid == source
                        && graphic.object_kind == "schematic_pin"
                        && visible(&graphic.layer_id)
                }) {
                    Ok(point.unwrap())
                } else {
                    Err("Target is hidden or has no current navigation anchor")
                };
                targets.insert(id, anchor);
            }
        }

        Self {
            current,
            schematic: matches!(surface, crate::PaneContent::Schematic),
            fingerprints,
            targets,
            document: scene.schematic_document.clone(),
            occurrences: documents.map(|documents| {
                checks
                    .findings
                    .iter()
                    .filter_map(|finding| {
                        let id = finding
                            .primary_target
                            .get("id")?
                            .as_str()?
                            .parse::<Uuid>()
                            .ok()?;
                        Some((id, documents.source_occurrences(id).cloned().collect()))
                    })
                    .collect()
            }),
        }
    }
    /// Exact producing frame and source identity, independent of display keys.
    pub fn anchor(&self, finding: &CheckFindingSummary) -> Result<PointNm, &'static str> {
        if !self.current {
            return Err("Finding and scene project/revision differ");
        }
        if finding.fingerprint.is_empty() || self.fingerprints.get(&finding.fingerprint) != Some(&1)
        {
            return Err("Finding fingerprint is unavailable or ambiguous");
        }
        if (self.schematic && finding.domain != "erc")
            || (!self.schematic && finding.domain != "drc")
        {
            return Err("Finding domain has no marker on this surface");
        }
        let primary = occurrence_path(&finding.primary_target)?;
        let payload = occurrence_path(&finding.payload)?;
        if primary
            .as_ref()
            .zip(payload.as_ref())
            .is_some_and(|(a, b)| a != b)
        {
            return Err("Finding primary and producing occurrence frames disagree");
        }
        let declared = primary.or(payload);
        if self.schematic {
            if finding
                .primary_target
                .get("kind")
                .and_then(serde_json::Value::as_str)
                != Some("object_uuid")
            {
                return Err("Finding has no typed source object navigation target");
            }
            let uuid = finding
                .primary_target
                .get("id")
                .and_then(serde_json::Value::as_str)
                .and_then(|id| id.parse::<Uuid>().ok())
                .ok_or("Finding has no source UUID navigation target")?;
            if let Some(occurrences) = &self.occurrences {
                let occurrence = qualify_source_occurrence(
                    occurrences.get(&uuid).into_iter().flatten(),
                    declared.as_deref(),
                )?;
                if self.document.as_ref() != Some(occurrence) {
                    return Err("Finding belongs to another source document occurrence");
                }
            } else if declared.is_some_and(|frame| !frame.is_empty()) || self.document.is_some() {
                return Err("Finding occurrence authority is unavailable");
            }
        } else if declared.is_some_and(|frame| !frame.is_empty()) {
            return Err("Finding occurrence frame is not a Board root frame");
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
pub(crate) fn occurrence_path(
    value: &serde_json::Value,
) -> Result<Option<Vec<Uuid>>, &'static str> {
    let mut result = None;
    for key in ["instance_path", "occurrence_path"] {
        if let Some(value) = value.get(key) {
            let path = value
                .as_array()
                .ok_or("Finding occurrence frame is malformed")?
                .iter()
                .map(|v| {
                    v.as_str()
                        .and_then(|s| s.parse().ok())
                        .ok_or("Finding occurrence UUID is malformed")
                })
                .collect::<Result<Vec<_>, _>>()?;
            if result.as_ref().is_some_and(|prior| prior != &path) {
                return Err("Finding occurrence frames disagree");
            }
            result = Some(path);
        }
    }
    if let Some(nested) = value.get("occurrence") {
        if let Some(path) = occurrence_path(nested)? {
            if result.as_ref().is_some_and(|prior| prior != &path) {
                return Err("Finding occurrence frames disagree");
            }
            result = Some(path);
        }
    }
    Ok(result)
}

#[cfg(test)]
#[path = "marker/tests.rs"]
mod tests;
