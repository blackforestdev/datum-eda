//! Source-owned projection of frozen local geometry. Display keys are not evidence.
use super::*;
use crate::board::{PadLayerConnection, PlacedPackage, PlacedPad, Stackup, StackupLayerType};
use crate::substrate::{Operation, SourceShardKind};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Default)]
pub struct ComponentPadProjection {
    pub pads: BTreeMap<Uuid, PlacedPad>,
    pub statuses: BTreeMap<Uuid, ComponentPadMaterializationStatus>,
    pub unavailable: BTreeMap<Uuid, String>,
    pub unadopted_packages: BTreeSet<Uuid>,
}
fn recorded(
    model: &DesignModel,
    package: Uuid,
    pad: &ComponentPadSource,
    pending: &[Operation],
) -> bool {
    model
        .journal
        .iter()
        .flat_map(|t| t.operations.iter())
        .chain(pending)
        .any(|op| {
            let value = match op {
                Operation::CreateBoardPackage {
                    package_id,
                    materialized,
                    ..
                }
                | Operation::SetBoardPackagePackage {
                    package_id,
                    materialized,
                    ..
                } if *package_id == package => materialized,
                _ => return false,
            };
            value
                .get("component_pads")
                .and_then(Value::as_array)
                .is_some_and(|pads| {
                    pads.iter().any(|v| {
                        serde_json::from_value::<ComponentPadSource>(v.clone())
                            .is_ok_and(|p| p == *pad)
                    })
                })
        })
}
/// Recorded source geometry remains inspectable after Set Part. Electrical
/// assignment is supplied only by existing explicit PM053 correspondence.
pub(crate) fn projection_with_operations(
    model: &DesignModel,
    board: &Value,
    pending: &[Operation],
) -> Result<ComponentPadProjection, EngineError> {
    let board_id: Uuid = serde_json::from_value(board["uuid"].clone())?;
    let mut result = ComponentPadProjection::default();
    let Some(placements) = board.get("component_pads").and_then(Value::as_object) else {
        return Ok(result);
    };
    if placements
        .values()
        .filter_map(Value::as_array)
        .flatten()
        .all(|p| p.get("source_binding").is_none_or(Value::is_null))
    {
        result.unadopted_packages.extend(
            placements
                .iter()
                .filter(|(_, v)| v.as_array().is_some_and(|a| !a.is_empty()))
                .filter_map(|(k, _)| Uuid::parse_str(k).ok()),
        );
        return Ok(result);
    }
    let stackup: Stackup = serde_json::from_value(board["stackup"].clone())?;
    for (key, values) in placements {
        let package_id =
            Uuid::parse_str(key).map_err(|_| invalid("invalid placement source key"))?;
        let package: PlacedPackage = serde_json::from_value(board["packages"][key].clone())?;
        if package.uuid != package_id {
            return Err(invalid("placement key/source UUID mismatch"));
        }
        let pads: Vec<ComponentPadSource> = serde_json::from_value(values.clone())?;
        for pad in pads {
            let Some(b) = &pad.source_binding else {
                result.unadopted_packages.insert(package_id);
                continue;
            };
            let projected = (|| {
                pad.validate_frozen()?;
                if b.project_id != model.project.project_id
                    || b.board_id != board_id
                    || b.package_id != package_id
                    || !recorded(model, package_id, &pad, pending)
                {
                    return Err(invalid("missing authoritative journaled materialization"));
                }
                if model.objects.get(&pad.uuid).is_none_or(|o| {
                    o.domain != "board"
                        || model
                            .source_shards
                            .iter()
                            .find(|s| s.shard_id == o.source_shard_id)
                            .is_none_or(|s| s.kind != SourceShardKind::BoardRoot)
                }) {
                    return Err(invalid("missing placed Pad source registration"));
                }
                let status = pad.materialization_status(model, package.part, package.package);
                result.statuses.insert(pad.uuid, status);
                project_pad(&pad, &package, &stackup)
            })();
            match projected {
                Ok(mut physical) => {
                    if result.statuses[&pad.uuid] == ComponentPadMaterializationStatus::Current {
                        physical.net = authoritative_net(model, &physical, b);
                    }
                    if result.pads.insert(pad.uuid, physical).is_some() {
                        return Err(invalid("duplicate per-placement Pad ID"));
                    }
                }
                Err(error) => {
                    result.unavailable.insert(pad.uuid, error.to_string());
                }
            }
        }
    }
    Ok(result)
}
fn project_pad(
    pad: &ComponentPadSource,
    package: &PlacedPackage,
    stackup: &Stackup,
) -> Result<PlacedPad, EngineError> {
    let b = pad.source_binding.as_ref().unwrap();
    let g = &b.frozen_geometry;
    let copper: Vec<_> = stackup
        .layers
        .iter()
        .filter(|l| l.layer_type == StackupLayerType::Copper)
        .map(|l| l.id)
        .collect();
    let first = *copper
        .first()
        .ok_or_else(|| invalid("missing copper stackup"))?;
    let last = *copper.last().unwrap();
    if package.layer != first && package.layer != last {
        return Err(invalid("placement side is not a declared copper face"));
    }
    let back = package.layer != first;
    let x = if back {
        g.position
            .x
            .checked_neg()
            .ok_or_else(|| invalid("placement reflection range"))?
    } else {
        g.position.x
    };
    let y = g.position.y;
    let (x, y) = match package.rotation.rem_euclid(360) {
        0 => (x, y),
        90 => (
            y.checked_neg()
                .ok_or_else(|| invalid("placement rotation range"))?,
            x,
        ),
        180 => (
            x.checked_neg()
                .ok_or_else(|| invalid("placement rotation range"))?,
            y.checked_neg()
                .ok_or_else(|| invalid("placement rotation range"))?,
        ),
        270 => (
            y,
            x.checked_neg()
                .ok_or_else(|| invalid("placement rotation range"))?,
        ),
        _ => {
            return Err(invalid(
                "exact local-point transform for this rotation is unavailable; no rounded authority",
            ));
        }
    };
    let position = Point::new(
        package
            .position
            .x
            .checked_add(x)
            .ok_or_else(|| invalid("placement position range"))?,
        package
            .position
            .y
            .checked_add(y)
            .ok_or_else(|| invalid("placement position range"))?,
    );
    let side_layer = |layer| {
        if back {
            if layer == first {
                last
            } else if layer == last {
                first
            } else {
                layer
            }
        } else {
            layer
        }
    };
    let layer = side_layer(g.layer);
    let (copper_layers, layer_connection) = match g.layer_span {
        PadstackLayerSpan::PadLayer => (vec![layer], PadLayerConnection::Separate),
        PadstackLayerSpan::Through => {
            let c = if g.plated == Some(true) {
                PadLayerConnection::PlatedThrough
            } else if g.plated == Some(false) {
                PadLayerConnection::Separate
            } else {
                PadLayerConnection::Unknown
            };
            (copper.clone(), c)
        }
        PadstackLayerSpan::Blind {
            start_layer,
            end_layer,
        } => {
            let start = side_layer(start_layer);
            let end = side_layer(end_layer);
            let a = copper
                .iter()
                .position(|l| *l == start)
                .ok_or_else(|| invalid("missing pinned span endpoint"))?;
            let z = copper
                .iter()
                .position(|l| *l == end)
                .ok_or_else(|| invalid("missing pinned span endpoint"))?;
            let c = if g.plated == Some(true) {
                PadLayerConnection::PlatedSpan {
                    start_layer: start,
                    end_layer: end,
                }
            } else if g.plated == Some(false) {
                PadLayerConnection::Separate
            } else {
                PadLayerConnection::Unknown
            };
            (copper[a.min(z)..=a.max(z)].to_vec(), c)
        }
    };
    let physical = PlacedPad {
        uuid: pad.uuid,
        package: package.uuid,
        name: pad.name.clone(),
        net: None,
        position,
        layer,
        copper_layers,
        layer_connection,
        shape: pad.shape.unwrap(),
        diameter: pad.diameter_nm,
        width: pad.width_nm,
        height: pad.height_nm,
        drill: pad.drill_nm.unwrap_or(0),
        rotation: package.rotation,
        roundrect_rratio_ppm: 250000,
        mask_layers: vec![],
        paste_layers: vec![],
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
    };
    crate::board::occupied_copper::validate_pad(&physical)
        .map_err(|e| invalid(format!("unavailable frozen Pad geometry: {e:?}")))?;
    Ok(physical)
}
fn authoritative_net(
    model: &DesignModel,
    pad: &PlacedPad,
    b: &ComponentPadSourceBinding,
) -> Option<Uuid> {
    let ci = b.component_instance.as_ref()?;
    let map = b.pin_pad_map.as_ref()?;
    if model
        .component_instances
        .get(&ci.object_id)
        .is_none_or(|c| {
            c.object_revision != ci.object_revision
                || c.part_ref != Some(b.part.object_id)
                || !c.placed_package_refs.contains(&pad.package)
        })
        || check_pin(model, map).is_err()
    {
        return None;
    }
    let mut nets = BTreeSet::new();
    for relation in model.electrical_identities.values() {
        if let crate::substrate::ElectricalIdentity::NetRelationship {
            board_net: Some(net),
            intent: crate::substrate::NetRelationshipIntent::Implemented,
            evidence,
            ..
        } = &relation.identity
            && evidence.iter().any(|e| {
                e.board_pad.object_id == pad.uuid
                    && e.component_instance == *ci
                    && e.pin_pad_map.object_id == map.object_id
                    && e.pin_pad_map.object_revision == map.object_revision
                    && e.library_pad.as_ref().is_some_and(|p| {
                        p.object_id == b.library_pad.object_id
                            && p.object_revision == b.library_pad.object_revision
                    })
            })
        {
            nets.insert(net.object_id);
        }
    }
    if nets.len() == 1 {
        nets.pop_first()
    } else {
        None
    }
}

/// Read-only, source-qualified projection. Electrical assignment is qualified
/// by the same PM053 basis used by complete Run/Net queries.
pub fn component_pad_projection(
    model: &DesignModel,
    board: &Value,
) -> Result<ComponentPadProjection, EngineError> {
    let mut projection = projection_with_operations(model, board, &[])?;
    if projection.pads.values().any(|p| p.net.is_some()) {
        let basis = crate::substrate::electrical_basis::ElectricalBasis::new(model, &[])?;
        for pad in projection.pads.values_mut() {
            pad.net = basis
                .object(model, pad.uuid, "pads")
                .and_then(|v| v.get("net"))
                .and_then(|n| serde_json::from_value(n.clone()).ok());
        }
    }
    Ok(projection)
}
pub(crate) fn project_board_value(
    model: &DesignModel,
    board: &Value,
    pending: &[Operation],
) -> Result<Value, EngineError> {
    let projection = projection_with_operations(model, board, pending)?;
    let mut value = board.clone();
    if projection.pads.is_empty() && projection.unavailable.is_empty() {
        return Ok(value);
    }
    if value.get("pads").is_none() {
        value["pads"] = serde_json::json!({});
    }
    let map = value
        .get_mut("pads")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| invalid("missing canonical Pad source map"))?;
    for (id, pad) in projection.pads {
        if map
            .insert(id.to_string(), serde_json::to_value(pad)?)
            .is_some()
        {
            return Err(invalid(
                "placed Pad projection collides with another authored source",
            ));
        }
    }
    // A failed transform remains explicit. Source identity can be inspected;
    // dependent physical membership must not advertise a partial result.
    value["component_pad_unavailable"] = serde_json::to_value(projection.unavailable)?;
    Ok(value)
}
