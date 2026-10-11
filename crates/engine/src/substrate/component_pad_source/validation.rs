//! Journal admission for whole materialization transitions and canonical preimages.
use super::*;
use crate::api::native_write::board_components::pad_materialization::ComponentPadTransition;
use crate::substrate::{Operation, OperationBatch, SourceShardKind, TransactionKind};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
pub(crate) fn validate_native(board: &Value) -> Result<(), EngineError> {
    let mut ids = BTreeSet::new();
    if let Some(map) = board.get("component_pads").and_then(Value::as_object) {
        for (key, values) in map {
            let Some(array) = values.as_array() else {
                continue;
            };
            for value in array
                .iter()
                .filter(|v| v.get("source_binding").is_some_and(|v| !v.is_null()))
            {
                if board["schema_version"].as_u64() != Some(4) {
                    return Err(invalid("placed-Pad bindings require explicit profile 4"));
                }
                let pad: ComponentPadSource = serde_json::from_value(value.clone())?;
                pad.validate_frozen()?;
                let binding = pad.source_binding.as_ref().unwrap();
                if let Some(snapshot) = &binding.process_snapshot {
                    super::process::validate_dependency_layers(board, snapshot, &pad)?;
                }
                if binding.package_id.to_string() != *key
                    || board["uuid"] != serde_json::json!(binding.board_id)
                    || !ids.insert(pad.uuid)
                    || serde_json::to_value(&pad)? != *value
                {
                    return Err(invalid(
                        "noncanonical, duplicate or foreign frozen Pad source",
                    ));
                }
            }
        }
    }
    for id in ids {
        for name in [
            "packages",
            "pads",
            "tracks",
            "vias",
            "zones",
            "graphics",
            "nets",
            "net_classes",
        ] {
            if board
                .get(name)
                .and_then(|v| v.get(id.to_string()))
                .is_some()
            {
                return Err(invalid("placed Pad source namespace collision"));
            }
        }
    }
    Ok(())
}
fn has_bindings(value: &Value) -> bool {
    if value.get("pad_source_transition").is_some() {
        return true;
    }
    value
        .get("component_pads")
        .and_then(Value::as_array)
        .is_some_and(|p| {
            p.iter()
                .any(|p| p.get("source_binding").is_some_and(|v| !v.is_null()))
        })
}
pub(crate) fn touches(operation: &Operation) -> bool {
    match operation {
        Operation::CreateBoardPackage { materialized, .. }
        | Operation::DeleteBoardPackage { materialized, .. } => has_bindings(materialized),
        Operation::SetBoardPackagePackage {
            materialized,
            previous_materialized,
            ..
        } => has_bindings(materialized) || has_bindings(previous_materialized),
        _ => false,
    }
}
pub(crate) fn touches_batch(
    model: &DesignModel,
    batch: &OperationBatch,
) -> Result<bool, EngineError> {
    if batch.operations.iter().any(touches) {
        return Ok(true);
    }
    let ids: BTreeSet<_> = batch
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::SetBoardPackagePosition { package_id, .. }
            | Operation::SetBoardPackageRotation { package_id, .. }
            | Operation::SetBoardPackageLocked { package_id, .. }
            | Operation::SetBoardPackagePart { package_id, .. }
            | Operation::SetBoardPackagePackage { package_id, .. }
            | Operation::SetBoardPackageValue { package_id, .. }
            | Operation::SetBoardPackageReference { package_id, .. }
            | Operation::SetComponentSide { package_id, .. }
            | Operation::DeleteBoardPackage { package_id, .. } => Some(*package_id),
            _ => None,
        })
        .collect();
    if ids.is_empty()
        || !model
            .source_shards
            .iter()
            .any(|s| s.kind == SourceShardKind::BoardRoot && s.schema_version == Some(4))
    {
        return Ok(false);
    }
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    Ok(ids.iter().any(|id| {
        board["component_pads"][id.to_string()]
            .as_array()
            .is_some_and(|pads| {
                pads.iter()
                    .any(|p| p.get("source_binding").is_some_and(|v| !v.is_null()))
            })
    }))
}
pub(crate) fn validate_batch(
    model: &DesignModel,
    root: &Path,
    batch: &OperationBatch,
    kind: TransactionKind,
) -> Result<(), EngineError> {
    if !touches_batch(model, batch)? {
        return Ok(());
    }
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::BoardRoot)
        .ok_or_else(|| invalid("missing BoardRoot"))?;
    let bytes = std::fs::read(root.join(&shard.relative_path))?;
    let board: Value = serde_json::from_slice(&bytes)?;
    if board["schema_version"].as_u64() != Some(4)
        || crate::substrate::sha256_hex(&bytes) != shard.content_hash
        || bytes
            != format!(
                "{}\n",
                crate::ir::serialization::to_json_deterministic(&board)?
            )
            .as_bytes()
    {
        return Err(invalid("current canonical profile-4 preimage required"));
    }
    let mut current = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    for operation in &batch.operations {
        match operation {
            Operation::CreateBoardPackage {
                package_id,
                package,
                materialized,
            } => {
                if kind == TransactionKind::Normal
                    && model.journal.iter().any(|t| {
                        t.diff.created.contains(package_id) || t.diff.deleted.contains(package_id)
                    })
                {
                    return Err(invalid(
                        "deleted/re-placed Package cannot recycle historical identity",
                    ));
                }
                let mut prospective = current.clone();
                prospective["packages"][package_id.to_string()] = package.clone();
                validate_materialization(
                    model,
                    &prospective,
                    *package_id,
                    None,
                    materialized,
                    kind,
                )?;
            }
            Operation::SetBoardPackagePackage {
                package_id,
                package_ref_id,
                previous_materialized,
                materialized,
            } => {
                let mut prospective = current.clone();
                prospective["packages"][package_id.to_string()]["package"] =
                    serde_json::json!(package_ref_id);
                validate_materialization(
                    model,
                    &prospective,
                    *package_id,
                    Some(previous_materialized),
                    materialized,
                    kind,
                )?;
            }
            Operation::DeleteBoardPackage {
                package_id,
                package,
                materialized,
            } => {
                validate_preimage(&current, *package_id, materialized)?;
                if current["packages"][package_id.to_string()] != *package {
                    return Err(invalid("stale deletion placement preimage"));
                }
            }
            _ => {}
        }
        crate::substrate::board_journal_ops::apply_board_operation(&mut current, operation)?;
        validate_native(&current)?;
    }
    Ok(())
}
fn require_taxon(
    model: &DesignModel,
    pin: &PinnedPadLibrarySource,
    taxon: crate::substrate::SourceShardTaxon,
) -> Result<(), EngineError> {
    if pin.owner_id != pin.object_id
        || model
            .source_shards
            .iter()
            .find(|s| s.shard_id == pin.source_shard_id)
            .is_none_or(|s| s.taxon != Some(taxon))
    {
        return Err(invalid("incorrect authoritative source family"));
    }
    Ok(())
}
fn validate_preimage(board: &Value, package_id: Uuid, previous: &Value) -> Result<(), EngineError> {
    let mut actual = serde_json::Map::new();
    for (name, map) in board
        .as_object()
        .ok_or_else(|| invalid("invalid BoardRoot"))?
    {
        if name.starts_with("component_")
            && let Some(value) = map.get(package_id.to_string())
        {
            actual.insert(name.clone(), value.clone());
        }
    }
    let mut supplied = previous
        .as_object()
        .cloned()
        .ok_or_else(|| invalid("invalid materialization preimage"))?;
    supplied.remove("pad_source_transition");
    if supplied != actual {
        return Err(invalid(
            "stale or incomplete whole materialization preimage",
        ));
    }
    Ok(())
}
fn validate_materialization(
    model: &DesignModel,
    board: &Value,
    package_id: Uuid,
    previous: Option<&Value>,
    next: &Value,
    kind: TransactionKind,
) -> Result<(), EngineError> {
    if let Some(previous) = previous {
        validate_preimage(board, package_id, previous)?;
    }
    let known = [
        "component_silkscreen",
        "component_silkscreen_texts",
        "component_silkscreen_arcs",
        "component_silkscreen_circles",
        "component_silkscreen_polygons",
        "component_silkscreen_polylines",
        "component_mechanical_lines",
        "component_mechanical_texts",
        "component_mechanical_polygons",
        "component_mechanical_polylines",
        "component_mechanical_circles",
        "component_mechanical_arcs",
        "component_pads",
        "component_models_3d",
        "pad_source_transition",
    ];
    if next
        .as_object()
        .is_none_or(|m| m.keys().any(|key| !known.contains(&key.as_str())))
    {
        return Err(invalid("unsupported materialization payload field"));
    }
    let old: Vec<ComponentPadSource> = previous
        .and_then(|v| v.get("component_pads"))
        .map_or_else(|| Ok(vec![]), |v| serde_json::from_value(v.clone()))?;
    let Some(array) = next.get("component_pads").and_then(Value::as_array) else {
        if kind == TransactionKind::Normal
            && (old.iter().any(|p| p.source_binding.is_some())
                || next.get("pad_source_transition").is_some())
        {
            return Err(invalid(
                "bound materialization requires an explicit component_pads array and complete transition accounting",
            ));
        }
        return Ok(());
    };
    let pads: Vec<ComponentPadSource> = serde_json::from_value(Value::Array(array.clone()))?;
    if pads.iter().all(|p| p.source_binding.is_none())
        && old.iter().all(|p| p.source_binding.is_none())
        && next.get("pad_source_transition").is_none()
    {
        return Ok(());
    }
    if kind == TransactionKind::Normal && pads.iter().any(|p| p.source_binding.is_none()) {
        return Err(invalid(
            "bound materialization cannot downgrade to legacy references",
        ));
    }
    if kind != TransactionKind::Normal && pads.iter().all(|p| p.source_binding.is_none()) {
        return Ok(());
    }
    let old_ids: BTreeSet<_> = old
        .iter()
        .filter(|p| p.source_binding.is_some())
        .map(|p| p.uuid)
        .collect();
    for pad in &pads {
        pad.validate_frozen()?;
        let b = pad.source_binding.as_ref().unwrap();
        if b.project_id != model.project.project_id
            || b.board_id != serde_json::from_value::<Uuid>(board["uuid"].clone())?
            || b.package_id != package_id
            || (kind == TransactionKind::Normal
                && board["packages"][package_id.to_string()]["part"]
                    != serde_json::json!(b.part.object_id))
            || board["packages"][package_id.to_string()]["package"]
                != serde_json::json!(b.package.object_id)
        {
            return Err(invalid("foreign source binding in materialization"));
        }
        if kind == TransactionKind::Normal {
            let placement: crate::board::PlacedPackage =
                serde_json::from_value(board["packages"][package_id.to_string()].clone())?;
            let expected_process = super::process::capture(model, board, pad, &placement)?;
            if b.process_snapshot != expected_process {
                return Err(invalid(
                    "process snapshot differs from current pinned authoritative inputs",
                ));
            }
            for p in [
                &b.part,
                &b.package,
                &b.land_pattern,
                &b.library_pad,
                &b.padstack,
            ] {
                check_pin(model, p)?;
            }
            let library = source_value(model, &b.land_pattern)?;
            let lib_pad: crate::pool::Pad = serde_json::from_value(
                library["pads"][b.library_pad.object_id.to_string()].clone(),
            )?;
            let padstack: crate::pool::Padstack =
                serde_json::from_value(source_value(model, &b.padstack)?)?;
            let frozen = &b.frozen_geometry;
            if lib_pad.uuid != b.library_pad.object_id
                || lib_pad.name != frozen.name
                || lib_pad.position != frozen.position
                || lib_pad.layer != frozen.layer
                || lib_pad.padstack != b.padstack.object_id
                || padstack.aperture.as_ref() != Some(&frozen.aperture)
                || padstack.drill_nm != frozen.drill_nm
                || padstack.layer_span != frozen.layer_span
                || padstack.plated != frozen.plated
            {
                return Err(invalid(
                    "frozen geometry differs from pinned authoritative library source",
                ));
            }
            if let Some(map) = &b.pin_pad_map {
                check_pin(model, map)?;
            }
            if let Some(reference) = &b.component_instance {
                let ci = model
                    .component_instances
                    .get(&reference.object_id)
                    .filter(|c| {
                        c.object_revision == reference.object_revision
                            && c.part_ref == Some(b.part.object_id)
                            && c.placed_package_refs.contains(&package_id)
                    })
                    .ok_or_else(|| invalid("stale ComponentInstance binding"))?;
                if let Some(map) = &b.pin_pad_map
                    && !ci.library_bindings.values().any(|binding| {
                        binding.binding_role == crate::substrate::LibraryBindingRole::PinPadMap
                            && binding.target_object_id == map.object_id
                            && binding.pinned_object_revision == map.object_revision
                    })
                {
                    return Err(invalid("PinPadMap is not pinned by the ComponentInstance"));
                }
            }
            if !old_ids.contains(&pad.uuid)
                && (model.objects.contains_key(&pad.uuid)
                    || model.journal.iter().any(|t| {
                        t.diff.created.contains(&pad.uuid) || t.diff.deleted.contains(&pad.uuid)
                    }))
            {
                return Err(invalid(
                    "new allocation reuses an existing or retired ObjectId",
                ));
            }
        }
    }
    if kind != TransactionKind::Normal {
        return Ok(());
    }
    let transition: ComponentPadTransition = serde_json::from_value(
        next.get("pad_source_transition")
            .cloned()
            .ok_or_else(|| invalid("missing whole-batch allocation/correspondence proof"))?,
    )?;
    if transition.project_id != model.project.project_id
        || transition.package_id != package_id
        || serde_json::json!(transition.board_id) != board["uuid"]
    {
        return Err(invalid("foreign transition proof"));
    }
    for pin in [
        &transition.target_part,
        &transition.target_package,
        &transition.target_land_pattern,
    ] {
        check_pin(model, pin)?;
    }
    if board["packages"][package_id.to_string()]["part"]
        != serde_json::json!(transition.target_part.object_id)
        || board["packages"][package_id.to_string()]["package"]
            != serde_json::json!(transition.target_package.object_id)
    {
        return Err(invalid("target producer differs from placement source"));
    }
    use crate::substrate::SourceShardTaxon as Taxon;
    require_taxon(model, &transition.target_part, Taxon::PoolPart)?;
    require_taxon(model, &transition.target_package, Taxon::PoolPackage)?;
    require_taxon(
        model,
        &transition.target_land_pattern,
        if transition.target_land_pattern.object_id == transition.target_package.object_id {
            Taxon::PoolPackage
        } else {
            Taxon::PoolFootprint
        },
    )?;
    for pad in &pads {
        let b = pad.source_binding.as_ref().unwrap();
        require_taxon(model, &b.padstack, Taxon::PoolPadstack)?;
        if let Some(map) = &b.pin_pad_map {
            require_taxon(model, map, Taxon::PoolPinPadMap)?;
            let value = source_value(model, map)?;
            let footprint = if transition.target_land_pattern.object_id
                == transition.target_package.object_id
            {
                None
            } else {
                Some(transition.target_land_pattern.object_id)
            };
            if value["part"] != serde_json::json!(transition.target_part.object_id)
                || value["footprint"] != serde_json::json!(footprint)
            {
                return Err(invalid("foreign PinPadMap producer"));
            }
        }
        if b.materialized_against != model.model_revision {
            return Err(invalid(
                "materialization was prepared against another source basis",
            ));
        }
    }
    let owner = source_value(model, &transition.target_land_pattern)?;
    if transition.target_land_pattern.object_id != transition.target_package.object_id
        && owner["package"] != serde_json::json!(transition.target_package.object_id)
    {
        return Err(invalid("target land pattern belongs to another package"));
    }
    let expected: BTreeSet<Uuid> = owner["pads"]
        .as_object()
        .map(|m| {
            m.keys()
                .map(|k| Uuid::parse_str(k).map_err(|_| invalid("invalid library Pad source key")))
                .collect()
        })
        .transpose()?
        .unwrap_or_default();
    let actual: BTreeSet<_> = pads
        .iter()
        .map(|p| p.source_binding.as_ref().unwrap().library_pad.object_id)
        .collect();
    if expected != actual
        || actual.len() != pads.len()
        || pads.iter().any(|p| {
            let b = p.source_binding.as_ref().unwrap();
            b.part != transition.target_part
                || b.package != transition.target_package
                || b.land_pattern != transition.target_land_pattern
        })
    {
        return Err(invalid(
            "materialization omits or mixes authoritative producer sources",
        ));
    }
    let new: BTreeMap<_, _> = pads.iter().map(|p| (p.uuid, p)).collect();
    let old: BTreeMap<_, _> = old.iter().map(|p| (p.uuid, p)).collect();
    if new.len() != pads.len() {
        return Err(invalid("duplicate placed Pad target"));
    }
    for (id, registration) in &transition.library_source_registrations {
        let pin = pads
            .iter()
            .filter_map(|p| p.source_binding.as_ref())
            .map(|b| &b.library_pad)
            .find(|p| p.object_id == *id)
            .ok_or_else(|| invalid("unrelated library registration recovery"))?;
        if registration.object_id != *id
            || registration.domain != "pool"
            || registration.source_shard_id != pin.source_shard_id
            || registration.object_revision != pin.object_revision
            || registration.kind != id.to_string()
        {
            return Err(invalid(
                "library source registration differs from its pinned source",
            ));
        }
    }
    let mut preserved = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for pair in &transition.correspondences {
        let old = old
            .get(&pair.placed_pad_id)
            .ok_or_else(|| invalid("foreign old correspondence"))?;
        let next = new
            .get(&pair.placed_pad_id)
            .ok_or_else(|| invalid("missing preserved target"))?;
        let prior = old
            .source_binding
            .as_ref()
            .ok_or_else(|| invalid("legacy library reference is not an authored Pad ID"))?;
        let target = &next.source_binding.as_ref().unwrap().library_pad;
        if *target != pair.target_library_pad
            || !preserved.insert(pair.placed_pad_id)
            || !targets.insert(target.object_id)
        {
            return Err(invalid("invalid one-to-one correspondence"));
        }
        continuity(model, &prior.library_pad, target)?;
    }
    let retired: BTreeSet<_> = old_ids.difference(&preserved).copied().collect();
    let allocations: BTreeMap<_, _> = new
        .iter()
        .filter(|(id, _)| !preserved.contains(id))
        .map(|(id, p)| {
            (
                p.source_binding.as_ref().unwrap().library_pad.object_id,
                *id,
            )
        })
        .collect();
    let legacy: BTreeSet<_> = old
        .values()
        .filter(|p| p.source_binding.is_none())
        .map(|p| p.uuid)
        .collect();
    if retired != transition.retirements
        || allocations != transition.allocations
        || legacy != transition.legacy_references
        || targets
            .intersection(&allocations.keys().copied().collect())
            .next()
            .is_some()
        || allocations.len() + preserved.len() != pads.len()
    {
        return Err(invalid(
            "whole old/new materialization is not accounted for exactly once",
        ));
    }
    Ok(())
}
