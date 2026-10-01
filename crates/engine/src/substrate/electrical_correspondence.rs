//! Exact explicit placed terminal → pinned PinPadMap cell correspondence.
use super::electrical_basis::ElectricalBasis;
use super::{DesignModel, EngineError, NetCorrespondence, NetCorrespondenceStatus};
use uuid::Uuid;

pub(super) fn certify(
    model: &DesignModel,
    source: &ElectricalBasis,
    evidence: &NetCorrespondence,
    board_net: Option<Uuid>,
) -> Result<NetCorrespondenceStatus, EngineError> {
    let Some(library_pad) = &evidence.library_pad else {
        return Ok(NetCorrespondenceStatus::Unverified);
    };
    let current = |reference: &super::RevisionedRef| {
        model
            .objects
            .get(&reference.object_id)
            .is_some_and(|o| o.object_revision == reference.object_revision)
    };
    if !current(&evidence.component_instance)
        || !current(&evidence.pin_pad_map)
        || !current(&evidence.board_pad)
    {
        return Ok(NetCorrespondenceStatus::Stale);
    }
    if !current(library_pad) {
        return Ok(NetCorrespondenceStatus::Stale);
    }
    if evidence.schematic_terminal.class != "pins" {
        return Ok(NetCorrespondenceStatus::Mismatch);
    }
    let Some(ci) = model
        .component_instances
        .get(&evidence.component_instance.object_id)
    else {
        return Ok(NetCorrespondenceStatus::Unverified);
    };
    let map: crate::pool::PinPadMap = serde_json::from_value(
        source
            .object(model, evidence.pin_pad_map.object_id, "pin_pad_maps")
            .ok_or_else(|| invalid("missing pinned map source"))?
            .clone(),
    )?;
    let pad: crate::board::PlacedPad = serde_json::from_value(
        source
            .object(model, evidence.board_pad.object_id, "pads")
            .ok_or_else(|| invalid("missing pad source"))?
            .clone(),
    )?;
    if !ci.placed_package_refs.contains(&pad.package)
        || ci.part_ref != Some(map.part)
        || pad.net != board_net
    {
        return Ok(NetCorrespondenceStatus::Mismatch);
    }
    let part: crate::pool::Part = serde_json::from_value(
        source
            .object(model, map.part, "parts")
            .ok_or_else(|| invalid("missing Part source"))?
            .clone(),
    )?;
    let placed: crate::board::PlacedPackage = serde_json::from_value(
        source
            .object(model, pad.package, "packages")
            .ok_or_else(|| invalid("missing placed package source"))?
            .clone(),
    )?;
    if placed.part != map.part || placed.package != part.package {
        return Ok(NetCorrespondenceStatus::Mismatch);
    }
    let library_owner = map.footprint.unwrap_or(part.package);
    let Some(owner) = model.objects.get(&library_owner) else {
        return Ok(NetCorrespondenceStatus::Unverified);
    };
    if model
        .objects
        .get(&library_pad.object_id)
        .is_none_or(|o| o.domain != "pool" || o.source_shard_id != owner.source_shard_id)
    {
        return Ok(NetCorrespondenceStatus::Mismatch);
    }
    let entity: crate::pool::Entity = serde_json::from_value(
        source
            .object(model, part.entity, "entities")
            .ok_or_else(|| invalid("missing Entity source"))?
            .clone(),
    )?;
    let Some(cell) = map.mappings.get(&library_pad.object_id) else {
        return Ok(NetCorrespondenceStatus::Mismatch);
    };
    let Some(gate) = entity.gates.get(&cell.gate) else {
        return Ok(NetCorrespondenceStatus::Mismatch);
    };
    let unit: crate::pool::Unit = serde_json::from_value(
        source
            .object(model, gate.unit, "units")
            .ok_or_else(|| invalid("missing Unit source"))?
            .clone(),
    )?;
    if !unit.pins.contains_key(&cell.pin) {
        return Ok(NetCorrespondenceStatus::Mismatch);
    }
    // The relationship explicitly names the library pad. It does not recover it
    // from the placed pad's display name or from a renderer-generated UUID.
    if source
        .object(model, library_pad.object_id, "pads")
        .is_none()
    {
        return Ok(NetCorrespondenceStatus::Mismatch);
    }
    let mut matches = 0;
    for symbol_id in &ci.placed_symbol_refs {
        let Some(value) = source.object(model, *symbol_id, "symbols") else {
            continue;
        };
        let symbol: crate::schematic::PlacedSymbol = serde_json::from_value(value.clone())?;
        for pin in &symbol.pins {
            if pin.uuid != evidence.schematic_terminal.source_id {
                continue;
            }
            if let Some(reference) = &evidence.library_pin {
                if !current(reference) {
                    return Ok(NetCorrespondenceStatus::Stale);
                }
                if pin.library_pin.is_some_and(|id| id != reference.object_id) {
                    return Ok(NetCorrespondenceStatus::Mismatch);
                }
            }
            let Some(library_pin) = evidence
                .library_pin
                .as_ref()
                .map(|r| r.object_id)
                .or(pin.library_pin)
            else {
                return Ok(NetCorrespondenceStatus::Unverified);
            };
            if symbol.gate != Some(cell.gate)
                || symbol.entity != Some(part.entity)
                || library_pin != cell.pin
            {
                return Ok(NetCorrespondenceStatus::Mismatch);
            }
            matches += 1;
        }
    }
    Ok(if matches == 1 {
        NetCorrespondenceStatus::Complete
    } else {
        NetCorrespondenceStatus::Mismatch
    })
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("invalid correspondence basis: {reason}"))
}
