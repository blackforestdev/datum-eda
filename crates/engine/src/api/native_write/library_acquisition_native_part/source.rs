//! Native source traversal preserves raw fields and refuses unsupported linked resources.
use super::*;
use crate::{
    api::native_write::schematic_library::part_placement::CompletePartSelection,
    pool::{Entity, Footprint, Package, Part},
    substrate::SourceShardKind,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

fn validate_owned_rows(raw: &Value, kind: &str) -> Result<(), EngineError> {
    let fields: &[(&str, bool)] = match kind {
        "units" => &[("pins", true)],
        "entities" => &[("gates", true)],
        "footprints" => &[("pads", true)],
        "packages" => &[("pads", true), ("terminals", true)],
        "pin_pad_maps" => &[("mappings", false)],
        _ => &[],
    };
    for (field, embedded) in fields {
        let Some(rows) = raw.get(*field) else {
            continue;
        };
        let rows = rows.as_object().ok_or_else(|| {
            invalid(format!(
                "Native {kind}.{field} must be an identity-keyed object"
            ))
        })?;
        let mut seen = std::collections::BTreeSet::new();
        for (key, row) in rows {
            let id: Uuid = key.parse().map_err(|_| {
                invalid(format!("Native {kind}.{field} has an invalid identity key"))
            })?;
            if !seen.insert(id)
                || key != &id.to_string()
                || (*embedded
                    && row
                        .get("uuid")
                        .and_then(Value::as_str)
                        .and_then(|s| s.parse::<Uuid>().ok())
                        != Some(id))
            {
                return Err(invalid(format!(
                    "Native {kind}.{field} has an ambiguous or mismatched owned identity"
                )));
            }
        }
    }
    for field in ["process_basis"] {
        if raw.get(field).is_some_and(|v| {
            !v.is_null()
                && v.as_array().is_none_or(|a| !a.is_empty())
                && v.as_object().is_none_or(|o| !o.is_empty())
        }) {
            return Err(invalid(format!(
                "Native {kind}.{field} linked resources are unsupported by acquisition"
            )));
        }
    }
    Ok(())
}

/// Materialize each native shard once per snapshot, not once per dependency/Part.
pub(super) struct Index<'a> {
    model: &'a DesignModel,
    rows: BTreeMap<Uuid, Vec<(&'a crate::substrate::SourceShardRef, Value)>>,
}
impl<'a> Index<'a> {
    pub(super) fn new(model: &'a DesignModel) -> Self {
        let mut rows: BTreeMap<Uuid, Vec<_>> = BTreeMap::new();
        for shard in model
            .source_shards
            .iter()
            .filter(|s| s.kind == SourceShardKind::Pool)
        {
            let Ok(raw) =
                model.materialized_source_shard_value_by_relative_path(&shard.relative_path)
            else {
                continue;
            };
            if let Some(id) = raw
                .get("uuid")
                .and_then(Value::as_str)
                .and_then(|v| v.parse::<Uuid>().ok())
            {
                rows.entry(id).or_default().push((shard, raw));
            }
        }
        Self { model, rows }
    }
    pub(super) fn has_identity(&self, id: Uuid) -> bool {
        self.rows.contains_key(&id)
    }
    pub(super) fn node(&self, id: Uuid, kind: &str) -> Result<NativePartNode, EngineError> {
        let record = self
            .model
            .objects
            .get(&id)
            .filter(|o| o.domain == "pool" && o.kind == kind)
            .ok_or_else(|| invalid(format!("Native {kind} {id} is unavailable or ambiguous")))?;
        let matches = self.rows.get(&id).map(Vec::as_slice).unwrap_or_default();
        let [(shard, raw)] = matches else {
            return Err(invalid(format!(
                "Native {kind} {id} does not have one unambiguous source"
            )));
        };
        if record.source_shard_id != shard.shard_id {
            return Err(invalid("Native object source ownership disagrees"));
        }
        validate_owned_rows(raw, kind)?;
        Ok(NativePartNode {
            id,
            kind: kind.into(),
            object: raw.clone(),
            revision: record.object_revision.0,
            source_hash: format!("sha256:{:x}", Sha256::digest(serde_json::to_vec(raw)?)),
            source_path: shard.relative_path.clone(),
        })
    }
}
#[cfg(test)]
pub(super) fn node(
    model: &DesignModel,
    id: Uuid,
    kind: &str,
) -> Result<NativePartNode, EngineError> {
    Index::new(model).node(id, kind)
}
pub(super) fn capture(
    path: &Path,
    model: &DesignModel,
    index: &Index<'_>,
    id: Uuid,
) -> Result<NativePartPreview, EngineError> {
    let part_node = index.node(id, "parts")?;
    let part: Part = serde_json::from_value(part_node.object.clone())?;
    if part.base.is_some()
        || !part.packaging_options.is_empty()
        || !part.behavioural_models.is_empty()
        || !part.pad_map.is_empty()
    {
        return Err(invalid(
            "Native Part base, alternate packaging, legacy inline map or linked models are not yet supported by acquisition",
        ));
    }
    let entity_node = index.node(part.entity, "entities")?;
    let entity: Entity = serde_json::from_value(entity_node.object.clone())?;
    let Some(first) = entity.gates.values().min_by_key(|g| g.uuid) else {
        return Err(invalid("Native Part has no declared gates"));
    };
    let complete = CompletePartSelection::capture(model, id, first.uuid, first.symbol)?;
    let package_node = index.node(part.package, "packages")?;
    let package: Package = serde_json::from_value(package_node.object.clone())?;
    let footprint_node = index.node(complete.footprint, "footprints")?;
    let footprint: Footprint = serde_json::from_value(footprint_node.object.clone())?;
    if !package.models_3d.is_empty()
        || !footprint.models_3d.is_empty()
        || footprint.process_aperture_policy.is_some()
        || footprint_node
            .object
            .get("process_basis")
            .is_some_and(|v| !v.is_null())
    {
        return Err(invalid(
            "Native linked models or footprint process resources are unsupported; acquisition cannot leave external dependencies",
        ));
    }
    let mut nodes = BTreeMap::from([
        (id, part_node),
        (part.entity, entity_node),
        (part.package, package_node),
        (complete.footprint, footprint_node),
    ]);
    let map = index.node(complete.pin_pad_map, "pin_pad_maps")?;
    nodes.insert(map.id, map);
    for gate in entity.gates.values() {
        CompletePartSelection::capture(model, id, gate.uuid, gate.symbol)?;
        for (id, kind) in [(gate.unit, "units"), (gate.symbol, "symbols")] {
            let value = index.node(id, kind)?;
            nodes.insert(id, value);
        }
        let unit: crate::pool::Unit = serde_json::from_value(nodes[&gate.unit].object.clone())?;
        let symbol: crate::pool::Symbol =
            serde_json::from_value(nodes[&gate.symbol].object.clone())?;
        if symbol
            .pin_anchors
            .iter()
            .any(|a| !unit.pins.contains_key(&a.pin))
        {
            return Err(invalid(
                "Native Symbol anchor references a pin outside its Unit",
            ));
        }
    }
    for pad in footprint.pads.values().chain(package.pads.values()) {
        let value = index.node(pad.padstack, "padstacks")?;
        let _: crate::pool::Padstack = serde_json::from_value(value.object.clone())?;
        nodes.insert(value.id, value);
    }
    let mut graph = crate::pool::LibraryGraph::default();
    for value in nodes.values() {
        graph.insert_pool_object(
            &value.kind,
            value.id,
            &value.source_path,
            value.object.clone(),
        );
    }
    if let Some(error) = graph
        .validation_report()
        .diagnostics
        .into_iter()
        .find(|d| d.severity == "error")
    {
        return Err(invalid(format!(
            "Native Part dependency validation failed: {}",
            error.message
        )));
    }
    let mut digest = Sha256::new();
    digest.update(model.project.project_id.as_bytes());
    digest.update(id.as_bytes());
    for n in nodes.values() {
        digest.update(n.id.as_bytes());
        digest.update(n.kind.as_bytes());
        digest.update([0]);
        digest.update(n.revision.to_le_bytes());
        digest.update(serde_json::to_vec(&n.object)?);
        digest.update([0]);
    }
    Ok(NativePartPreview {
        source_path: path.to_path_buf(),
        source_project: model.project.project_id,
        part: id,
        name: format!("{} {} · {}", part.manufacturer, part.mpn, part.value),
        digest: format!("sha256:{:x}", digest.finalize()),
        nodes: nodes.into_values().collect(),
    })
}
