use crate::pool::Pool;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LandPatternPad {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub position: crate::ir::geometry::Point,
    pub layer: crate::ir::geometry::LayerId,
}

pub(super) fn land_pattern_pads_for_part(
    part: &crate::pool::Part,
    package: &crate::pool::Package,
    pool: &Pool,
) -> Vec<LandPatternPad> {
    if let Some(footprint_id) = part.default_footprint
        && let Some(pads) = footprint_pads_for_package(pool, footprint_id, package.uuid)
    {
        return pads;
    }
    if let Some(map_id) = part.default_pin_pad_map
        && let Some(map) = pool.pin_pad_maps.get(&map_id)
        && map.part == part.uuid
        && let Some(footprint_id) = map.footprint
        && let Some(pads) = footprint_pads_for_package(pool, footprint_id, package.uuid)
    {
        return pads;
    }
    land_pattern_pads_for_package(package, pool)
}

pub(super) fn land_pattern_pads_for_package(
    package: &crate::pool::Package,
    pool: &Pool,
) -> Vec<LandPatternPad> {
    if let Some(pads) = unique_footprint_pads_for_package(pool, package.uuid) {
        return pads;
    }
    package_pads(package)
}

fn unique_footprint_pads_for_package(
    pool: &Pool,
    package_uuid: uuid::Uuid,
) -> Option<Vec<LandPatternPad>> {
    let mut matches = pool
        .footprints
        .values()
        .filter(|footprint| footprint.package == package_uuid && !footprint.pads.is_empty());
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(footprint_pads(first))
}

fn footprint_pads_for_package(
    pool: &Pool,
    footprint_uuid: uuid::Uuid,
    package_uuid: uuid::Uuid,
) -> Option<Vec<LandPatternPad>> {
    let footprint = pool.footprints.get(&footprint_uuid)?;
    if footprint.package != package_uuid || footprint.pads.is_empty() {
        return None;
    }
    Some(footprint_pads(footprint))
}

fn footprint_pads(footprint: &crate::pool::Footprint) -> Vec<LandPatternPad> {
    let mut pads: Vec<_> = footprint
        .pads
        .values()
        .map(|pad| LandPatternPad {
            uuid: pad.uuid,
            name: pad.name.clone(),
            position: pad.position,
            layer: pad.layer,
        })
        .collect();
    pads.sort_by_key(|pad| pad.uuid);
    pads
}

fn package_pads(package: &crate::pool::Package) -> Vec<LandPatternPad> {
    let mut pads: Vec<_> = package
        .pads
        .values()
        .map(|pad| LandPatternPad {
            uuid: pad.uuid,
            name: pad.name.clone(),
            position: pad.position,
            layer: pad.layer,
        })
        .collect();
    pads.sort_by_key(|pad| pad.uuid);
    pads
}

#[cfg(test)]
#[path = "ops_helpers_landpattern_tests.rs"]
mod tests;
