use std::collections::HashMap;

use uuid::Uuid;

use crate::board::*;
use crate::ir::geometry::Point;

#[test]
fn board_round_trip() {
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "demo".into(),
        stackup: Stackup {
            layers: vec![StackupLayer {
                id: 1,
                name: "Top".into(),
                layer_type: StackupLayerType::Copper,
                thickness_nm: 35_000,
            }],
        },
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
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
    };

    let json = serde_json::to_string(&board).unwrap();
    let restored: Board = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, board);
}

#[test]
fn board_summary_counts_core_objects() {
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "demo".into(),
        stackup: Stackup {
            layers: vec![
                StackupLayer {
                    id: 1,
                    name: "Top".into(),
                    layer_type: StackupLayerType::Copper,
                    thickness_nm: 35_000,
                },
                StackupLayer {
                    id: 2,
                    name: "Core".into(),
                    layer_type: StackupLayerType::Dielectric,
                    thickness_nm: 1_000_000,
                },
            ],
        },
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
        packages: HashMap::from([(
            Uuid::new_v4(),
            PlacedPackage {
                uuid: Uuid::new_v4(),
                part: Uuid::new_v4(),
                package: Uuid::nil(),
                reference: "R1".into(),
                value: "10k".into(),
                position: Point::new(0, 0),
                rotation: 0,
                layer: 1,
                locked: false,
            },
        )]),
        pads: HashMap::new(),
        tracks: HashMap::new(),
        vias: HashMap::new(),
        zones: HashMap::new(),
        nets: HashMap::from([(
            Uuid::new_v4(),
            Net {
                uuid: Uuid::new_v4(),
                name: "VCC".into(),
                class: Uuid::new_v4(),
            },
        )]),
        net_classes: HashMap::new(),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let summary = board.summary();
    assert_eq!(summary.name, "demo");
    assert_eq!(summary.layer_count, 2);
    assert_eq!(summary.component_count, 1);
    assert_eq!(summary.net_count, 1);
}

#[test]
fn component_query_is_sorted_by_reference() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "demo".into(),
        stackup: Stackup { layers: Vec::new() },
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
        packages: HashMap::from([
            (
                a,
                PlacedPackage {
                    uuid: a,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "R10".into(),
                    value: "10k".into(),
                    position: Point::new(0, 0),
                    rotation: 0,
                    layer: 0,
                    locked: false,
                },
            ),
            (
                b,
                PlacedPackage {
                    uuid: b,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "R1".into(),
                    value: "1k".into(),
                    position: Point::new(1, 1),
                    rotation: 90,
                    layer: 31,
                    locked: true,
                },
            ),
        ]),
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
    };

    let components = board.components();
    assert_eq!(components.len(), 2);
    assert_eq!(components[0].reference, "R1");
    assert_eq!(components[1].reference, "R10");
}

#[test]
fn board_net_info_counts_tracks_and_vias() {
    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let pkg_a = Uuid::new_v4();
    let pkg_b = Uuid::new_v4();
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "demo".into(),
        stackup: Stackup {
            layers: vec![StackupLayer {
                id: 31,
                name: "B.Cu".into(),
                layer_type: StackupLayerType::Copper,
                thickness_nm: 35_000,
            }],
        },
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
        packages: HashMap::from([
            (
                pkg_a,
                PlacedPackage {
                    uuid: pkg_a,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "R1".into(),
                    value: "10k".into(),
                    position: Point::new(0, 0),
                    rotation: 0,
                    layer: 31,
                    locked: false,
                },
            ),
            (
                pkg_b,
                PlacedPackage {
                    uuid: pkg_b,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "R2".into(),
                    value: "10k".into(),
                    position: Point::new(5_000_000, 0),
                    rotation: 0,
                    layer: 31,
                    locked: false,
                },
            ),
        ]),
        pads: HashMap::from([
            (
                Uuid::new_v4(),
                PlacedPad {
                    uuid: Uuid::new_v4(),
                    package: pkg_a,
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(0, 0),
                    layer: 31,
                },
            ),
            (
                Uuid::new_v4(),
                PlacedPad {
                    uuid: Uuid::new_v4(),
                    package: pkg_b,
                    name: "2".into(),
                    net: Some(net_uuid),
                    position: Point::new(5_000_000, 0),
                    layer: 31,
                },
            ),
        ]),
        tracks: HashMap::from([(
            Uuid::new_v4(),
            Track {
                uuid: Uuid::new_v4(),
                net: net_uuid,
                from: Point::new(0, 0),
                to: Point::new(3_000_000, 4_000_000),
                width: 250_000,
                layer: 31,
            },
        )]),
        vias: HashMap::from([(
            Uuid::new_v4(),
            Via {
                uuid: Uuid::new_v4(),
                net: net_uuid,
                position: Point::new(3_000_000, 4_000_000),
                drill: 400_000,
                diameter: 800_000,
                from_layer: 0,
                to_layer: 31,
            },
        )]),
        zones: HashMap::new(),
        nets: HashMap::from([(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "GND".into(),
                class: class_uuid,
            },
        )]),
        net_classes: HashMap::from([(
            class_uuid,
            NetClass {
                uuid: class_uuid,
                name: "Power".into(),
                clearance: 0,
                track_width: 0,
                via_drill: 0,
                via_diameter: 0,
                diffpair_width: 0,
                diffpair_gap: 0,
            },
        )]),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let infos = board.net_info();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].name, "GND");
    assert_eq!(infos[0].class, "Power");
    assert_eq!(infos[0].tracks, 1);
    assert_eq!(infos[0].vias, 1);
    assert_eq!(infos[0].zones, 0);
    assert_eq!(infos[0].pins.len(), 2);
    assert_eq!(infos[0].routed_length_nm, 5_000_000);
    assert_eq!(infos[0].routed_pct, 1.0);
}
