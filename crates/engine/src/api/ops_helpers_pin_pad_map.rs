use std::collections::{BTreeMap, BTreeSet};

use crate::pool::{PadMapEntry, Pool};

pub(super) fn part_pin_signature(
    part: &crate::pool::Part,
    pool: &Pool,
) -> Option<BTreeSet<String>> {
    let entity = pool.entities.get(&part.entity)?;
    let mut pins = BTreeSet::new();
    for entry in resolved_part_pad_map(part, pool).values() {
        let gate = entity.gates.get(&entry.gate)?;
        let unit = pool.units.get(&gate.unit)?;
        let pin = unit.pins.get(&entry.pin)?;
        pins.insert(pin.name.clone());
    }
    Some(pins)
}

pub(super) fn resolved_part_pad_map(
    part: &crate::pool::Part,
    pool: &Pool,
) -> BTreeMap<uuid::Uuid, PadMapEntry> {
    if let Some(map_id) = part.default_pin_pad_map
        && let Some(pin_pad_map) = pool.pin_pad_maps.get(&map_id)
        && pin_pad_map.part == part.uuid
    {
        let resolved = resolve_first_class_pin_pad_map(part, pin_pad_map, pool);
        if !resolved.is_empty() {
            return resolved;
        }
    }

    part.pad_map
        .iter()
        .map(|(pad_uuid, entry)| (*pad_uuid, entry.clone()))
        .collect()
}

fn resolve_first_class_pin_pad_map(
    part: &crate::pool::Part,
    pin_pad_map: &crate::pool::PinPadMap,
    pool: &Pool,
) -> BTreeMap<uuid::Uuid, PadMapEntry> {
    let Some(entity) = pool.entities.get(&part.entity) else {
        return BTreeMap::new();
    };
    let Some(package) = pool.packages.get(&part.package) else {
        return BTreeMap::new();
    };
    let footprint = pin_pad_map
        .footprint
        .and_then(|footprint_id| pool.footprints.get(&footprint_id))
        .filter(|footprint| footprint.package == part.package);

    let package_pad_by_name: BTreeMap<&str, uuid::Uuid> = package
        .pads
        .values()
        .map(|pad| (pad.name.as_str(), pad.uuid))
        .collect();

    let mut resolved = BTreeMap::new();
    for (mapped_pad_uuid, entry) in &pin_pad_map.mappings {
        let Some(gate) = entity.gates.get(&entry.gate) else {
            continue;
        };
        let Some(unit) = pool.units.get(&gate.unit) else {
            continue;
        };
        if !unit.pins.contains_key(&entry.pin) {
            continue;
        }
        let Some(package_pad_uuid) =
            resolve_mapped_package_pad(*mapped_pad_uuid, package, footprint, &package_pad_by_name)
        else {
            continue;
        };
        resolved.insert(
            package_pad_uuid,
            PadMapEntry {
                gate: entry.gate,
                pin: entry.pin,
            },
        );
    }
    resolved
}

fn resolve_mapped_package_pad(
    mapped_pad_uuid: uuid::Uuid,
    package: &crate::pool::Package,
    footprint: Option<&crate::pool::Footprint>,
    package_pad_by_name: &BTreeMap<&str, uuid::Uuid>,
) -> Option<uuid::Uuid> {
    if package.pads.contains_key(&mapped_pad_uuid) {
        return Some(mapped_pad_uuid);
    }
    let footprint_pad = footprint?.pads.get(&mapped_pad_uuid)?;
    package_pad_by_name
        .get(footprint_pad.name.as_str())
        .copied()
}

#[cfg(test)]
#[path = "ops_helpers_pin_pad_map_tests.rs"]
mod tests;
