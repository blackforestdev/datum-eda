//! Compose PM057 adoption with the existing board/fill import and map owner.
use super::super::board_annotations::graphics::{
    build_create_board_graphic, build_set_board_graphic,
};
use super::*;
use crate::board::graphics::BoardGraphic;
use serde_json::Value;

/// Complete one atomic import after the separately explicit profile adoption.
/// Reimport cannot overwrite a native edit; source-authored refresh is proven
/// against its recorded map/geometry preimage, never object revision heuristics.
pub fn build_kicad_graphic_import(
    model: &DesignModel,
    provenance: WriteProvenance,
    base: ImportWrite,
    board_id: ObjectId,
    graphics: &[BoardGraphic],
) -> Result<ImportWrite, EngineError> {
    let affected = !graphics.is_empty()
        || base.prepared.as_ref().is_some_and(|p| {
            p.batch.operations.iter().any(|op| {
                if let Operation::CreateImportMapShard { shard, .. } = op {
                    serde_json::from_value::<ImportMapShard>(shard.clone())
                        .ok()
                        .is_some_and(|s| {
                            s.entries
                                .iter()
                                .any(|e| e.import_key.starts_with("kicad:board-graphic:"))
                        })
                } else {
                    false
                }
            })
        });
    if !affected {
        return Ok(base);
    }
    let root = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let mut map_operations = Vec::new();
    let mut operations = Vec::new();
    if let Some(prepared) = base.prepared {
        for operation in prepared.batch.operations {
            match &operation {
                Operation::GuardObjectRevision { .. } => {}
                Operation::CreateImportMapShard { relative_path, .. } => {
                    // The existing Create/Delete pair records an actual preimage
                    // when replacing a sidecar. Its inverse restores that map,
                    // rather than deleting previously established provenance.
                    if model.source_shards.iter().any(|s| {
                        s.kind == SourceShardKind::ImportMap && &s.relative_path == relative_path
                    }) {
                        let old = model
                            .materialized_source_shard_value_by_relative_path(relative_path)?;
                        let shard = model
                            .source_shards
                            .iter()
                            .find(|s| &s.relative_path == relative_path)
                            .expect("checked import-map source");
                        let raw = std::fs::read(&shard.path)?;
                        let canonical = format!(
                            "{}\n",
                            crate::ir::serialization::to_json_deterministic(&old)?
                        );
                        if raw != canonical.as_bytes() {
                            return Err(EngineError::Validation("graphic adoption requires a canonical ImportMap preimage; unsupported noncanonical replacement refused".into()));
                        }
                        map_operations.push(Operation::DeleteImportMapShard {
                            relative_path: relative_path.clone(),
                            shard: old,
                        });
                    }
                    map_operations.push(operation);
                }
                _ => operations.push(operation),
            }
        }
    }
    let mut created = base.created_object_count;
    let mut present = root.get("graphics").is_some();
    let mut ordered: Vec<_> = graphics.iter().collect();
    ordered.sort_by_key(|g| g.uuid);
    for graphic in ordered {
        let key = graphic.import_key.as_deref().ok_or_else(|| {
            EngineError::Validation(
                "graphic adoption requires recorded ImportMap provenance".into(),
            )
        })?;
        let payload = serde_json::to_value(graphic)?;
        if let Some(previous) = root
            .get("graphics")
            .and_then(|g| g.get(graphic.uuid.to_string()))
        {
            if previous == &payload {
                continue;
            }
            if previous.get("import_key").and_then(Value::as_str) != Some(key)
                || !recorded_import_preimage(model, key, graphic.uuid, previous)
            {
                return Err(EngineError::Validation(format!(
                    "graphic {} conflicts with native edits or lacks a recorded imported preimage; explicit source reconciliation required",
                    graphic.uuid
                )));
            }
            operations.extend(unguarded_operations(build_set_board_graphic(
                model,
                provenance.clone(),
                board_id,
                graphic,
            )?));
        } else {
            if model.objects.contains_key(&graphic.uuid) {
                return Err(EngineError::Validation(
                    "graphic import identity collides with another native source".into(),
                ));
            }
            let prepared =
                build_create_board_graphic(model, provenance.clone(), board_id, graphic)?;
            for mut operation in unguarded_operations(prepared) {
                if let Operation::CreateBoardGraphic {
                    map_was_present, ..
                } = &mut operation
                {
                    *map_was_present = present;
                }
                operations.push(operation);
            }
            present = true;
            created += 1;
        }
    }
    map_operations.extend(operations);
    let mut result = finish_import_write(model, provenance, map_operations, created)?;
    if let Some(prepared) = &mut result.prepared
        && prepared
            .batch
            .operations
            .iter()
            .any(|op| matches!(op, Operation::SetZoneFill { .. }))
    {
        // Reuse PM056's final-source basis after adding nonconductive objects;
        // never attach the imported fill to the shorter base batch's revision.
        let basis = crate::substrate::project_board_basis(model, &prepared.batch)?;
        for operation in &mut prepared.batch.operations {
            if let Operation::SetZoneFill {
                zone_id, zone_fill, ..
            } = operation
            {
                zone_fill["model_revision"] = serde_json::to_value(&basis.model_revision)?;
                zone_fill["source_zone_revision"] = serde_json::to_value(
                    basis
                        .objects
                        .get(zone_id)
                        .ok_or_else(|| {
                            EngineError::Validation("imported fill has no source Zone".into())
                        })?
                        .object_revision,
                )?;
            }
        }
    }
    Ok(result)
}
fn recorded_import_preimage(
    model: &DesignModel,
    key: &str,
    id: ObjectId,
    previous: &Value,
) -> bool {
    for transaction in model.journal.iter().rev() {
        let bound = transaction.operations.iter().any(|op| {
            if let Operation::CreateImportMapShard { shard, .. } = op {
                serde_json::from_value::<ImportMapShard>(shard.clone())
                    .ok()
                    .is_some_and(|map| {
                        map.entries.iter().any(|e| {
                            e.import_key == key && e.object_id == id && e.source_tool == "kicad"
                        })
                    })
            } else {
                false
            }
        });
        if !bound {
            continue;
        }
        if let Some(payload) = transaction.operations.iter().rev().find_map(|op| match op {
            Operation::CreateBoardGraphic {
                graphic_id,
                graphic,
                ..
            }
            | Operation::SetBoardGraphic {
                graphic_id,
                graphic,
                ..
            } if *graphic_id == id => Some(graphic),
            _ => None,
        }) {
            return payload == previous;
        }
    }
    false
}

#[cfg(test)]
#[path = "imported_graphics/tests.rs"]
mod tests;
