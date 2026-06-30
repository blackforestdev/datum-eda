use super::*;
use std::collections::{HashMap, HashSet};

use crate::ir::geometry::{Point, Polygon};
use crate::pool::{Footprint, Lifecycle, Package, Pad, Part, PinPadMap, Pool};

const PACKAGE_ID: uuid::Uuid = uuid::Uuid::from_u128(0x10);
const OTHER_PACKAGE_ID: uuid::Uuid = uuid::Uuid::from_u128(0x11);
const PART_ID: uuid::Uuid = uuid::Uuid::from_u128(0x20);
const FOOTPRINT_ID: uuid::Uuid = uuid::Uuid::from_u128(0x30);
const OTHER_FOOTPRINT_ID: uuid::Uuid = uuid::Uuid::from_u128(0x31);
const PIN_PAD_MAP_ID: uuid::Uuid = uuid::Uuid::from_u128(0x40);
const PACKAGE_PAD_ID: uuid::Uuid = uuid::Uuid::from_u128(0x50);
const FOOTPRINT_PAD_ID: uuid::Uuid = uuid::Uuid::from_u128(0x60);

#[test]
fn land_pattern_pads_for_part_prefers_default_footprint_over_package_pads() {
    let mut pool = base_pool();
    pool.footprints.insert(
        FOOTPRINT_ID,
        footprint(FOOTPRINT_ID, PACKAGE_ID, "FP1", 11, 12),
    );
    let mut part = part();
    part.default_footprint = Some(FOOTPRINT_ID);

    let pads = land_pattern_pads_for_part(&part, pool.packages.get(&PACKAGE_ID).unwrap(), &pool);

    assert_eq!(pads.len(), 1);
    assert_eq!(pads[0].uuid, FOOTPRINT_PAD_ID);
    assert_eq!(pads[0].name, "FP1");
    assert_eq!(pads[0].position, Point::new(11, 12));
}

#[test]
fn land_pattern_pads_for_part_uses_pin_pad_map_footprint_when_part_default_absent() {
    let mut pool = base_pool();
    pool.footprints.insert(
        FOOTPRINT_ID,
        footprint(FOOTPRINT_ID, PACKAGE_ID, "MAP1", 21, 22),
    );
    pool.pin_pad_maps.insert(
        PIN_PAD_MAP_ID,
        PinPadMap {
            uuid: PIN_PAD_MAP_ID,
            part: PART_ID,
            footprint: Some(FOOTPRINT_ID),
            mappings: HashMap::new(),
            tags: HashSet::new(),
        },
    );
    let mut part = part();
    part.default_pin_pad_map = Some(PIN_PAD_MAP_ID);

    let pads = land_pattern_pads_for_part(&part, pool.packages.get(&PACKAGE_ID).unwrap(), &pool);

    assert_eq!(pads.len(), 1);
    assert_eq!(pads[0].uuid, FOOTPRINT_PAD_ID);
    assert_eq!(pads[0].name, "MAP1");
    assert_eq!(pads[0].position, Point::new(21, 22));
}

#[test]
fn land_pattern_pads_for_part_rejects_wrong_package_default_footprint_and_falls_back() {
    let mut pool = base_pool();
    pool.footprints.insert(
        OTHER_FOOTPRINT_ID,
        footprint(OTHER_FOOTPRINT_ID, OTHER_PACKAGE_ID, "WRONG", 31, 32),
    );
    let mut part = part();
    part.default_footprint = Some(OTHER_FOOTPRINT_ID);

    let pads = land_pattern_pads_for_part(&part, pool.packages.get(&PACKAGE_ID).unwrap(), &pool);

    assert_eq!(pads.len(), 1);
    assert_eq!(pads[0].uuid, PACKAGE_PAD_ID);
    assert_eq!(pads[0].name, "PKG1");
    assert_eq!(pads[0].position, Point::new(1, 2));
}

#[test]
fn land_pattern_pads_for_package_uses_unique_matching_footprint_then_fallback_on_ambiguity() {
    let mut pool = base_pool();
    pool.footprints.insert(
        FOOTPRINT_ID,
        footprint(FOOTPRINT_ID, PACKAGE_ID, "FP1", 41, 42),
    );

    let pads = land_pattern_pads_for_package(pool.packages.get(&PACKAGE_ID).unwrap(), &pool);
    assert_eq!(pads[0].name, "FP1");

    pool.footprints.insert(
        OTHER_FOOTPRINT_ID,
        footprint(OTHER_FOOTPRINT_ID, PACKAGE_ID, "FP2", 51, 52),
    );
    let fallback = land_pattern_pads_for_package(pool.packages.get(&PACKAGE_ID).unwrap(), &pool);
    assert_eq!(fallback[0].uuid, PACKAGE_PAD_ID);
    assert_eq!(fallback[0].name, "PKG1");
}

fn base_pool() -> Pool {
    let mut pool = Pool::default();
    pool.packages
        .insert(PACKAGE_ID, package(PACKAGE_ID, "PKG", "PKG1", 1, 2));
    pool.packages.insert(
        OTHER_PACKAGE_ID,
        package(OTHER_PACKAGE_ID, "OTHER", "OTHER1", 3, 4),
    );
    pool
}

fn package(uuid: uuid::Uuid, name: &str, pad_name: &str, x: i64, y: i64) -> Package {
    Package {
        uuid,
        name: name.to_string(),
        package_family: None,
        package_code: None,
        mounting_type: None,
        body_dimensions: None,
        terminals: HashMap::new(),
        pads: HashMap::from([(
            PACKAGE_PAD_ID,
            Pad {
                uuid: PACKAGE_PAD_ID,
                name: pad_name.to_string(),
                position: Point::new(x, y),
                padstack: uuid::Uuid::nil(),
                layer: 1,
            },
        )]),
        courtyard: Polygon::new(Vec::new()),
        silkscreen: Vec::new(),
        models_3d: Vec::new(),
        body_height_nm: None,
        body_height_mounted_nm: None,
        tags: HashSet::new(),
    }
}

fn footprint(uuid: uuid::Uuid, package: uuid::Uuid, pad_name: &str, x: i64, y: i64) -> Footprint {
    Footprint {
        uuid,
        name: "FP".to_string(),
        package,
        pads: HashMap::from([(
            FOOTPRINT_PAD_ID,
            Pad {
                uuid: FOOTPRINT_PAD_ID,
                name: pad_name.to_string(),
                position: Point::new(x, y),
                padstack: uuid::Uuid::nil(),
                layer: 1,
            },
        )]),
        courtyard: Polygon::new(Vec::new()),
        silkscreen: Vec::new(),
        fab: Vec::new(),
        assembly: Vec::new(),
        mechanical: Vec::new(),
        models_3d: Vec::new(),
        standards_basis: None,
        process_aperture_policy: None,
        tags: HashSet::new(),
    }
}

fn part() -> Part {
    Part {
        uuid: PART_ID,
        entity: uuid::Uuid::nil(),
        package: PACKAGE_ID,
        default_footprint: None,
        default_pin_pad_map: None,
        pad_map: HashMap::new(),
        mpn: String::new(),
        manufacturer: String::new(),
        manufacturer_jep106: None,
        value: String::new(),
        description: String::new(),
        datasheet: String::new(),
        parametric: HashMap::new(),
        orderable_mpns: Vec::new(),
        packaging_options: Vec::new(),
        tags: HashSet::new(),
        lifecycle: Lifecycle::Active,
        base: None,
        behavioural_models: Vec::new(),
        thermal: None,
        supply_chain_offers: None,
        last_supply_chain_check: None,
    }
}
