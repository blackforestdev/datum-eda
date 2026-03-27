use std::collections::{BTreeMap, BTreeSet};

use crate::board::Board;
use crate::error::EngineError;
use crate::pool::Pool;
use crate::rules::ast::RuleType;

type PadPositions = Vec<(uuid::Uuid, crate::ir::geometry::Point)>;

pub(super) fn apply_package_transform(
    board: &mut Board,
    before: &crate::board::PlacedPackage,
    after: &crate::board::PlacedPackage,
) -> Result<(PadPositions, PadPositions), EngineError> {
    let package = board
        .packages
        .get_mut(&before.uuid)
        .ok_or(EngineError::NotFound {
            object_type: "component",
            uuid: before.uuid,
        })?;
    *package = after.clone();

    let mut before_pads = Vec::new();
    let mut after_pads = Vec::new();
    for pad in board
        .pads
        .values_mut()
        .filter(|pad| pad.package == before.uuid)
    {
        before_pads.push((pad.uuid, pad.position));
        let local =
            inverse_transform_board_local_point(before.position, before.rotation, pad.position);
        pad.position = transform_board_local_point(after.position, after.rotation, local);
        after_pads.push((pad.uuid, pad.position));
    }
    before_pads.sort_by_key(|(uuid, _)| *uuid);
    after_pads.sort_by_key(|(uuid, _)| *uuid);
    Ok((before_pads, after_pads))
}

pub(super) fn restore_package_transform(
    board: &mut Board,
    package_uuid: uuid::Uuid,
    package_state: crate::board::PlacedPackage,
    pad_positions: &[(uuid::Uuid, crate::ir::geometry::Point)],
) -> Result<(), EngineError> {
    let package = board
        .packages
        .get_mut(&package_uuid)
        .ok_or(EngineError::NotFound {
            object_type: "component",
            uuid: package_uuid,
        })?;
    *package = package_state;
    for (pad_uuid, position) in pad_positions {
        let pad = board.pads.get_mut(pad_uuid).ok_or(EngineError::NotFound {
            object_type: "pad",
            uuid: *pad_uuid,
        })?;
        pad.position = *position;
    }
    Ok(())
}

pub(super) fn component_pads(
    board: &Board,
    component_uuid: uuid::Uuid,
) -> Vec<crate::board::PlacedPad> {
    let mut pads: Vec<_> = board
        .pads
        .values()
        .filter(|pad| pad.package == component_uuid)
        .cloned()
        .collect();
    pads.sort_by_key(|pad| pad.uuid);
    pads
}

pub(super) fn restore_component_pads(
    board: &mut Board,
    component_uuid: uuid::Uuid,
    pads: &[crate::board::PlacedPad],
) {
    let stale_pad_uuids: Vec<_> = board
        .pads
        .values()
        .filter(|pad| pad.package == component_uuid)
        .map(|pad| pad.uuid)
        .collect();
    for pad_uuid in stale_pad_uuids {
        board.pads.remove(&pad_uuid);
    }
    for pad in pads {
        board.pads.insert(pad.uuid, pad.clone());
    }
}

pub(super) fn replace_component_pads_from_pool_package(
    board: &mut Board,
    component: &crate::board::PlacedPackage,
    package: &crate::pool::Package,
) -> Result<(), EngineError> {
    let net_by_name: BTreeMap<String, Option<uuid::Uuid>> = component_pads(board, component.uuid)
        .into_iter()
        .map(|pad| (pad.name, pad.net))
        .collect();
    let mut regenerated = Vec::new();
    for package_pad in package.pads.values() {
        regenerated.push(crate::board::PlacedPad {
            uuid: deterministic_component_pad_uuid(component.uuid, &package_pad.name),
            package: component.uuid,
            name: package_pad.name.clone(),
            net: net_by_name.get(&package_pad.name).copied().flatten(),
            position: transform_board_local_point(
                component.position,
                component.rotation,
                package_pad.position,
            ),
            layer: package_pad.layer,
            diameter: 0,
        });
    }
    regenerated.sort_by_key(|pad| pad.uuid);
    restore_component_pads(board, component.uuid, &regenerated);
    Ok(())
}

pub(super) fn replace_component_pads_for_assign_part(
    board: &mut Board,
    previous_component: &crate::board::PlacedPackage,
    next_component: &crate::board::PlacedPackage,
    target_part: &crate::pool::Part,
    target_package: &crate::pool::Package,
    pool: &Pool,
) -> Result<(), EngineError> {
    let net_by_name: BTreeMap<String, Option<uuid::Uuid>> =
        component_pads(board, previous_component.uuid)
            .into_iter()
            .map(|pad| (pad.name, pad.net))
            .collect();
    let mut net_by_pin = BTreeMap::new();

    if previous_component.part != uuid::Uuid::nil()
        && let Some(current_part) = pool.parts.get(&previous_component.part)
        && current_part.package == previous_component.package
    {
        let current_package = pool.packages.get(&previous_component.package).ok_or(
            EngineError::DanglingReference {
                source_type: "component",
                source_uuid: previous_component.uuid,
                target_type: "package",
                target_uuid: previous_component.package,
            },
        )?;
        let current_pad_name_by_uuid: BTreeMap<uuid::Uuid, &str> = current_package
            .pads
            .values()
            .map(|pad| (pad.uuid, pad.name.as_str()))
            .collect();

        for (pad_uuid, entry) in &current_part.pad_map {
            if let Some(pad_name) = current_pad_name_by_uuid.get(pad_uuid)
                && let Some(net) = net_by_name.get(*pad_name).copied().flatten()
            {
                net_by_pin.insert(entry.pin, net);
            }
        }
    }

    let mut regenerated = Vec::new();
    for package_pad in target_package.pads.values() {
        let net = target_part
            .pad_map
            .get(&package_pad.uuid)
            .and_then(|entry| net_by_pin.get(&entry.pin).copied())
            .or_else(|| net_by_name.get(&package_pad.name).copied().flatten());
        regenerated.push(crate::board::PlacedPad {
            uuid: deterministic_component_pad_uuid(next_component.uuid, &package_pad.name),
            package: next_component.uuid,
            name: package_pad.name.clone(),
            net,
            position: transform_board_local_point(
                next_component.position,
                next_component.rotation,
                package_pad.position,
            ),
            layer: package_pad.layer,
            diameter: 0,
        });
    }
    regenerated.sort_by_key(|pad| pad.uuid);
    restore_component_pads(board, next_component.uuid, &regenerated);
    Ok(())
}

pub(super) fn resolve_compatible_part_for_package_change(
    current_part_uuid: uuid::Uuid,
    target_package_uuid: uuid::Uuid,
    pool: &Pool,
) -> Option<uuid::Uuid> {
    let current_part = pool.parts.get(&current_part_uuid)?;
    let current_signature = part_pin_signature(current_part, pool)?;
    let mut candidates = pool
        .parts
        .values()
        .filter(|part| {
            part.package == target_package_uuid
                && part_pin_signature(part, pool).as_ref() == Some(&current_signature)
        })
        .map(|part| part.uuid);
    let first = candidates.next()?;
    if candidates.next().is_some() {
        None
    } else {
        Some(first)
    }
}

pub(super) fn part_pin_signature(
    part: &crate::pool::Part,
    pool: &Pool,
) -> Option<BTreeSet<String>> {
    let entity = pool.entities.get(&part.entity)?;
    let mut pins = BTreeSet::new();
    for entry in part.pad_map.values() {
        let gate = entity.gates.get(&entry.gate)?;
        let unit = pool.units.get(&gate.unit)?;
        let pin = unit.pins.get(&entry.pin)?;
        pins.insert(pin.name.clone());
    }
    Some(pins)
}

pub(super) fn deterministic_component_pad_uuid(
    component_uuid: uuid::Uuid,
    pad_name: &str,
) -> uuid::Uuid {
    crate::ir::ids::import_uuid(
        &crate::ir::ids::namespace_kicad(),
        &format!("board/pad/{component_uuid}/{pad_name}"),
    )
}

pub(super) fn transform_board_local_point(
    origin: crate::ir::geometry::Point,
    rotation_deg: i32,
    local: crate::ir::geometry::Point,
) -> crate::ir::geometry::Point {
    let rotated = match rotation_deg.rem_euclid(360) {
        90 => crate::ir::geometry::Point::new(-local.y, local.x),
        180 => crate::ir::geometry::Point::new(-local.x, -local.y),
        270 => crate::ir::geometry::Point::new(local.y, -local.x),
        _ => local,
    };
    crate::ir::geometry::Point::new(origin.x + rotated.x, origin.y + rotated.y)
}

pub(super) fn inverse_transform_board_local_point(
    origin: crate::ir::geometry::Point,
    rotation_deg: i32,
    absolute: crate::ir::geometry::Point,
) -> crate::ir::geometry::Point {
    let translated = crate::ir::geometry::Point::new(absolute.x - origin.x, absolute.y - origin.y);
    match rotation_deg.rem_euclid(360) {
        90 => crate::ir::geometry::Point::new(translated.y, -translated.x),
        180 => crate::ir::geometry::Point::new(-translated.x, -translated.y),
        270 => crate::ir::geometry::Point::new(-translated.y, translated.x),
        _ => translated,
    }
}

pub(super) fn default_rule_name(rule_type: &RuleType) -> String {
    match rule_type {
        RuleType::ClearanceCopper => "clearance_copper".to_string(),
        RuleType::TrackWidth => "track_width".to_string(),
        RuleType::ViaHole => "via_hole".to_string(),
        RuleType::ViaAnnularRing => "via_annular_ring".to_string(),
        RuleType::HoleSize => "hole_size".to_string(),
        RuleType::SilkClearance => "silk_clearance".to_string(),
        RuleType::Connectivity => "connectivity".to_string(),
    }
}
