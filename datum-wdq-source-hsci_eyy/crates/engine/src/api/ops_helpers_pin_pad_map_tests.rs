use super::*;
use std::collections::{HashMap, HashSet};

use crate::api::ops_helpers::{component_pads, replace_component_pads_for_assign_part};
use crate::board::{Board, Net, PadShape, PlacedPackage, PlacedPad, Stackup};
use crate::ir::geometry::{Point, Polygon};
use crate::pool::{
    Entity, Footprint, Gate, LibraryPinElectricalType, Lifecycle, Package, Pad, Part, Pin,
    PinPadMap, Unit,
};

const COMPONENT_ID: uuid::Uuid = uuid::Uuid::from_u128(0x100);
const OLD_PACKAGE_ID: uuid::Uuid = uuid::Uuid::from_u128(0x200);
const NEW_PACKAGE_ID: uuid::Uuid = uuid::Uuid::from_u128(0x201);
const CURRENT_PART_ID: uuid::Uuid = uuid::Uuid::from_u128(0x300);
const TARGET_PART_ID: uuid::Uuid = uuid::Uuid::from_u128(0x301);
const ENTITY_ID: uuid::Uuid = uuid::Uuid::from_u128(0x400);
const UNIT_ID: uuid::Uuid = uuid::Uuid::from_u128(0x500);
const GATE_ID: uuid::Uuid = uuid::Uuid::from_u128(0x600);
const GATE_B_ID: uuid::Uuid = uuid::Uuid::from_u128(0x601);
const PIN_A_ID: uuid::Uuid = uuid::Uuid::from_u128(0x700);
const PIN_B_ID: uuid::Uuid = uuid::Uuid::from_u128(0x701);
const OLD_PAD_A_ID: uuid::Uuid = uuid::Uuid::from_u128(0x800);
const OLD_PAD_B_ID: uuid::Uuid = uuid::Uuid::from_u128(0x801);
const NEW_PAD_X_ID: uuid::Uuid = uuid::Uuid::from_u128(0x900);
const NEW_PAD_Y_ID: uuid::Uuid = uuid::Uuid::from_u128(0x901);
const CURRENT_MAP_ID: uuid::Uuid = uuid::Uuid::from_u128(0xa00);
const TARGET_MAP_ID: uuid::Uuid = uuid::Uuid::from_u128(0xa01);
const SIG_NET_ID: uuid::Uuid = uuid::Uuid::from_u128(0xb00);
const OLD_FOOTPRINT_ID: uuid::Uuid = uuid::Uuid::from_u128(0xc00);
const TARGET_FOOTPRINT_ID: uuid::Uuid = uuid::Uuid::from_u128(0xc01);
const OLD_FOOTPRINT_PAD_A_ID: uuid::Uuid = uuid::Uuid::from_u128(0xd00);
const TARGET_FOOTPRINT_PAD_X_ID: uuid::Uuid = uuid::Uuid::from_u128(0xd01);
const TARGET_FOOTPRINT_PAD_Y_ID: uuid::Uuid = uuid::Uuid::from_u128(0xd02);

#[test]
fn replace_component_pads_prefers_default_pin_pad_map_over_legacy_pad_map() {
    let (mut pool, mut board, previous, next) = replacement_fixture();
    pool.parts
        .get_mut(&CURRENT_PART_ID)
        .expect("current part should exist")
        .pad_map = HashMap::from([(
        OLD_PAD_A_ID,
        PadMapEntry {
            gate: GATE_ID,
            pin: PIN_B_ID,
        },
    )]);
    pool.parts
        .get_mut(&TARGET_PART_ID)
        .expect("target part should exist")
        .pad_map = HashMap::from([(
        NEW_PAD_Y_ID,
        PadMapEntry {
            gate: GATE_ID,
            pin: PIN_B_ID,
        },
    )]);

    let target_part = pool.parts.get(&TARGET_PART_ID).unwrap();
    let target_package = pool.packages.get(&NEW_PACKAGE_ID).unwrap();
    replace_component_pads_for_assign_part(
        &mut board,
        &previous,
        &next,
        target_part,
        target_package,
        &pool,
    )
    .expect("pad replacement should succeed");

    let remapped = component_pads(&board, COMPONENT_ID);
    let new_y = remapped
        .iter()
        .find(|pad| pad.name == "NEW_Y")
        .expect("NEW_Y should exist");
    let new_x = remapped
        .iter()
        .find(|pad| pad.name == "NEW_X")
        .expect("NEW_X should exist");
    assert_eq!(new_y.net, Some(SIG_NET_ID));
    assert_eq!(new_x.net, None);
}

#[test]
fn part_pin_signature_prefers_default_pin_pad_map_over_legacy_pad_map() {
    let (mut pool, _, _, _) = replacement_fixture();
    pool.parts
        .get_mut(&TARGET_PART_ID)
        .expect("target part should exist")
        .pad_map = HashMap::from([(
        NEW_PAD_Y_ID,
        PadMapEntry {
            gate: GATE_ID,
            pin: PIN_B_ID,
        },
    )]);

    let target_part = pool.parts.get(&TARGET_PART_ID).unwrap();
    let signature = part_pin_signature(target_part, &pool).expect("signature should resolve");

    assert_eq!(signature, BTreeSet::from(["A".to_string()]));
}

#[test]
fn resolved_part_pad_map_falls_back_to_legacy_when_default_map_is_missing() {
    let (mut pool, mut board, previous, next) = replacement_fixture();
    pool.parts
        .get_mut(&CURRENT_PART_ID)
        .expect("current part should exist")
        .default_pin_pad_map = Some(uuid::Uuid::from_u128(0xdead));
    pool.parts
        .get_mut(&TARGET_PART_ID)
        .expect("target part should exist")
        .default_pin_pad_map = Some(uuid::Uuid::from_u128(0xbeef));
    pool.parts
        .get_mut(&TARGET_PART_ID)
        .expect("target part should exist")
        .pad_map = HashMap::from([(
        NEW_PAD_X_ID,
        PadMapEntry {
            gate: GATE_ID,
            pin: PIN_A_ID,
        },
    )]);

    let target_part = pool.parts.get(&TARGET_PART_ID).unwrap();
    let target_package = pool.packages.get(&NEW_PACKAGE_ID).unwrap();
    replace_component_pads_for_assign_part(
        &mut board,
        &previous,
        &next,
        target_part,
        target_package,
        &pool,
    )
    .expect("pad replacement should succeed");

    let remapped = component_pads(&board, COMPONENT_ID);
    assert_eq!(
        remapped
            .iter()
            .find(|pad| pad.name == "NEW_X")
            .expect("NEW_X should exist")
            .net,
        Some(SIG_NET_ID)
    );
}

#[test]
fn replace_component_pads_for_assign_part_prefers_default_footprint_pads() {
    let (mut pool, mut board, previous, next) = replacement_fixture();
    pool.footprints.insert(
        OLD_FOOTPRINT_ID,
        footprint(
            OLD_FOOTPRINT_ID,
            OLD_PACKAGE_ID,
            OLD_FOOTPRINT_PAD_A_ID,
            "OLD_A",
            uuid::Uuid::from_u128(0xd03),
            "OLD_B",
        ),
    );
    pool.footprints.insert(
        TARGET_FOOTPRINT_ID,
        footprint(
            TARGET_FOOTPRINT_ID,
            NEW_PACKAGE_ID,
            TARGET_FOOTPRINT_PAD_X_ID,
            "NEW_X",
            TARGET_FOOTPRINT_PAD_Y_ID,
            "NEW_Y",
        ),
    );
    pool.parts
        .get_mut(&CURRENT_PART_ID)
        .expect("current part should exist")
        .default_footprint = Some(OLD_FOOTPRINT_ID);
    pool.parts
        .get_mut(&TARGET_PART_ID)
        .expect("target part should exist")
        .default_footprint = Some(TARGET_FOOTPRINT_ID);

    let target_part = pool.parts.get(&TARGET_PART_ID).unwrap();
    let target_package = pool.packages.get(&NEW_PACKAGE_ID).unwrap();
    replace_component_pads_for_assign_part(
        &mut board,
        &previous,
        &next,
        target_part,
        target_package,
        &pool,
    )
    .expect("pad replacement should succeed");

    let remapped = component_pads(&board, COMPONENT_ID);
    let new_y = remapped
        .iter()
        .find(|pad| pad.name == "NEW_Y")
        .expect("NEW_Y should exist");
    let new_x = remapped
        .iter()
        .find(|pad| pad.name == "NEW_X")
        .expect("NEW_X should exist");
    assert_eq!(new_y.net, Some(SIG_NET_ID));
    assert_eq!(new_y.position, Point::new(20, 20));
    assert_eq!(new_x.net, None);
    assert_eq!(new_x.position, Point::new(10, 10));
}

#[test]
fn first_class_pin_pad_map_preserves_repeated_unit_pin_across_multiple_gates() {
    let (mut pool, _, _, _) = replacement_fixture();
    pool.entities.insert(
        ENTITY_ID,
        Entity {
            uuid: ENTITY_ID,
            name: "Dual Gate".to_string(),
            prefix: "U".to_string(),
            manufacturer: String::new(),
            gates: HashMap::from([
                (
                    GATE_ID,
                    Gate {
                        uuid: GATE_ID,
                        name: "A".to_string(),
                        unit: UNIT_ID,
                        symbol: uuid::Uuid::nil(),
                    },
                ),
                (
                    GATE_B_ID,
                    Gate {
                        uuid: GATE_B_ID,
                        name: "B".to_string(),
                        unit: UNIT_ID,
                        symbol: uuid::Uuid::nil(),
                    },
                ),
            ]),
            tags: HashSet::new(),
        },
    );
    pool.pin_pad_maps.insert(
        TARGET_MAP_ID,
        PinPadMap {
            uuid: TARGET_MAP_ID,
            part: TARGET_PART_ID,
            footprint: None,
            mappings: HashMap::from([
                (
                    NEW_PAD_X_ID,
                    PadMapEntry {
                        gate: GATE_ID,
                        pin: PIN_A_ID,
                    },
                ),
                (
                    NEW_PAD_Y_ID,
                    PadMapEntry {
                        gate: GATE_B_ID,
                        pin: PIN_A_ID,
                    },
                ),
            ]),
            tags: HashSet::new(),
        },
    );

    let target_part = pool.parts.get(&TARGET_PART_ID).unwrap();
    let resolved = resolved_part_pad_map(target_part, &pool);

    assert_eq!(resolved.get(&NEW_PAD_X_ID).unwrap().gate, GATE_ID);
    assert_eq!(resolved.get(&NEW_PAD_X_ID).unwrap().pin, PIN_A_ID);
    assert_eq!(resolved.get(&NEW_PAD_Y_ID).unwrap().gate, GATE_B_ID);
    assert_eq!(resolved.get(&NEW_PAD_Y_ID).unwrap().pin, PIN_A_ID);
}

fn replacement_fixture() -> (Pool, Board, PlacedPackage, PlacedPackage) {
    let mut pool = Pool::default();
    pool.units.insert(UNIT_ID, unit());
    pool.entities.insert(ENTITY_ID, entity());
    pool.packages.insert(
        OLD_PACKAGE_ID,
        package(
            OLD_PACKAGE_ID,
            "OLD",
            OLD_PAD_A_ID,
            "OLD_A",
            OLD_PAD_B_ID,
            "OLD_B",
        ),
    );
    pool.packages.insert(
        NEW_PACKAGE_ID,
        package(
            NEW_PACKAGE_ID,
            "NEW",
            NEW_PAD_X_ID,
            "NEW_X",
            NEW_PAD_Y_ID,
            "NEW_Y",
        ),
    );
    pool.pin_pad_maps.insert(
        CURRENT_MAP_ID,
        pin_pad_map(
            CURRENT_MAP_ID,
            CURRENT_PART_ID,
            GATE_ID,
            PIN_A_ID,
            OLD_PAD_A_ID,
        ),
    );
    pool.pin_pad_maps.insert(
        TARGET_MAP_ID,
        pin_pad_map(
            TARGET_MAP_ID,
            TARGET_PART_ID,
            GATE_ID,
            PIN_A_ID,
            NEW_PAD_Y_ID,
        ),
    );
    pool.parts.insert(
        CURRENT_PART_ID,
        part(
            CURRENT_PART_ID,
            OLD_PACKAGE_ID,
            CURRENT_MAP_ID,
            OLD_PAD_A_ID,
        ),
    );
    pool.parts.insert(
        TARGET_PART_ID,
        part(TARGET_PART_ID, NEW_PACKAGE_ID, TARGET_MAP_ID, NEW_PAD_X_ID),
    );

    let previous = placed_component(CURRENT_PART_ID, OLD_PACKAGE_ID, "OLD");
    let next = placed_component(TARGET_PART_ID, NEW_PACKAGE_ID, "NEW");
    let mut board = empty_board();
    board.packages.insert(COMPONENT_ID, previous.clone());
    board.pads.insert(
        OLD_PAD_A_ID,
        placed_pad(OLD_PAD_A_ID, "OLD_A", Some(SIG_NET_ID)),
    );
    board
        .pads
        .insert(OLD_PAD_B_ID, placed_pad(OLD_PAD_B_ID, "OLD_B", None));
    board
        .nets
        .insert(SIG_NET_ID, Net::new(SIG_NET_ID, "SIG", uuid::Uuid::nil()));

    (pool, board, previous, next)
}

fn unit() -> Unit {
    Unit {
        uuid: UNIT_ID,
        name: "U".to_string(),
        manufacturer: String::new(),
        pins: HashMap::from([
            (PIN_A_ID, pin(PIN_A_ID, "A")),
            (PIN_B_ID, pin(PIN_B_ID, "B")),
        ]),
        tags: HashSet::new(),
    }
}

fn pin(uuid: uuid::Uuid, name: &str) -> Pin {
    Pin {
        uuid,
        name: name.to_string(),
        direction: LibraryPinElectricalType::Passive,
        swap_group: 0,
        alternates: Vec::new(),
    }
}

fn entity() -> Entity {
    Entity {
        uuid: ENTITY_ID,
        name: "E".to_string(),
        prefix: "U".to_string(),
        manufacturer: String::new(),
        gates: HashMap::from([(
            GATE_ID,
            Gate {
                uuid: GATE_ID,
                name: "A".to_string(),
                unit: UNIT_ID,
                symbol: uuid::Uuid::nil(),
            },
        )]),
        tags: HashSet::new(),
    }
}

fn package(
    uuid: uuid::Uuid,
    name: &str,
    first_pad_uuid: uuid::Uuid,
    first_pad_name: &str,
    second_pad_uuid: uuid::Uuid,
    second_pad_name: &str,
) -> Package {
    Package {
        uuid,
        name: name.to_string(),
        package_family: None,
        package_code: None,
        mounting_type: None,
        body_dimensions: None,
        terminals: HashMap::new(),
        pads: HashMap::from([
            (first_pad_uuid, package_pad(first_pad_uuid, first_pad_name)),
            (
                second_pad_uuid,
                package_pad(second_pad_uuid, second_pad_name),
            ),
        ]),
        courtyard: Polygon::new(Vec::new()),
        silkscreen: Vec::new(),
        models_3d: Vec::new(),
        body_height_nm: None,
        body_height_mounted_nm: None,
        tags: HashSet::new(),
    }
}

fn package_pad(uuid: uuid::Uuid, name: &str) -> Pad {
    Pad {
        uuid,
        name: name.to_string(),
        position: Point::new(1, 1),
        padstack: uuid::Uuid::nil(),
        layer: 1,
    }
}

fn footprint(
    uuid: uuid::Uuid,
    package: uuid::Uuid,
    first_pad_uuid: uuid::Uuid,
    first_pad_name: &str,
    second_pad_uuid: uuid::Uuid,
    second_pad_name: &str,
) -> Footprint {
    Footprint {
        uuid,
        name: "FP".to_string(),
        package,
        pads: HashMap::from([
            (
                first_pad_uuid,
                Pad {
                    uuid: first_pad_uuid,
                    name: first_pad_name.to_string(),
                    position: Point::new(10, 10),
                    padstack: uuid::Uuid::nil(),
                    layer: 1,
                },
            ),
            (
                second_pad_uuid,
                Pad {
                    uuid: second_pad_uuid,
                    name: second_pad_name.to_string(),
                    position: Point::new(20, 20),
                    padstack: uuid::Uuid::nil(),
                    layer: 1,
                },
            ),
        ]),
        courtyard: Polygon::new(Vec::new()),
        silkscreen: Vec::new(),
        fab: Vec::new(),
        assembly: Vec::new(),
        mechanical: Vec::new(),
        models_3d: Vec::new(),
        standards_basis: None,
        ipc_basis: None,
        process_aperture_policy: None,
        tags: HashSet::new(),
    }
}

fn pin_pad_map(
    uuid: uuid::Uuid,
    part: uuid::Uuid,
    gate_uuid: uuid::Uuid,
    pin_uuid: uuid::Uuid,
    pad_uuid: uuid::Uuid,
) -> PinPadMap {
    PinPadMap {
        uuid,
        part,
        footprint: None,
        mappings: HashMap::from([(
            pad_uuid,
            PadMapEntry {
                gate: gate_uuid,
                pin: pin_uuid,
            },
        )]),
        tags: HashSet::new(),
    }
}

fn part(
    uuid: uuid::Uuid,
    package: uuid::Uuid,
    default_pin_pad_map: uuid::Uuid,
    legacy_pad_uuid: uuid::Uuid,
) -> Part {
    Part {
        uuid,
        entity: ENTITY_ID,
        package,
        default_footprint: None,
        default_pin_pad_map: Some(default_pin_pad_map),
        pad_map: HashMap::from([(
            legacy_pad_uuid,
            PadMapEntry {
                gate: GATE_ID,
                pin: PIN_A_ID,
            },
        )]),
        mpn: String::new(),
        manufacturer: String::new(),
        manufacturer_jep106: None,
        value: "P".to_string(),
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

fn placed_component(part: uuid::Uuid, package: uuid::Uuid, value: &str) -> PlacedPackage {
    PlacedPackage {
        uuid: COMPONENT_ID,
        part,
        package,
        reference: "U1".to_string(),
        value: value.to_string(),
        position: Point::zero(),
        rotation: 0,
        layer: 1,
        locked: false,
    }
}

fn placed_pad(uuid: uuid::Uuid, name: &str, net: Option<uuid::Uuid>) -> PlacedPad {
    PlacedPad {
        uuid,
        package: COMPONENT_ID,
        name: name.to_string(),
        net,
        position: Point::zero(),
        layer: 1,
        copper_layers: Vec::new(),
        shape: PadShape::Circle,
        diameter: 0,
        width: 0,
        height: 0,
        drill: 0,
        rotation: 0,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        roundrect_rratio_ppm: 250_000,
    }
}

fn empty_board() -> Board {
    Board {
        uuid: uuid::Uuid::new_v4(),
        name: "B".to_string(),
        stackup: Stackup { layers: Vec::new() },
        pad_expansion_setup: Default::default(),
        outline: Polygon::new(Vec::new()),
        packages: HashMap::new(),
        pads: HashMap::new(),
        tracks: HashMap::new(),
        vias: HashMap::new(),
        zones: HashMap::new(),
        nets: HashMap::new(),
        net_classes: HashMap::new(),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    }
}
