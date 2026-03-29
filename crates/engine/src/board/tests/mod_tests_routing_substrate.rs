use std::collections::HashMap;

use super::super::{
    Board, Keepout, Net, NetClass, RoutingComponentPad, RoutingPadSource, Stackup, StackupLayer,
    StackupLayerType, Track, Via, Zone,
};
use crate::board::{PlacedPackage, PlacedPad, RuleSet};
use crate::ir::geometry::{Point, Polygon};
use uuid::Uuid;

#[test]
fn routing_substrate_reports_deterministic_sorted_persisted_facts() {
    let net_class_uuid = Uuid::new_v4();
    let net_uuid_b = Uuid::new_v4();
    let net_uuid_a = Uuid::new_v4();
    let component_uuid_b = Uuid::new_v4();
    let component_uuid_a = Uuid::new_v4();
    let board_pad_uuid = Uuid::new_v4();
    let component_pad_uuid_b = Uuid::new_v4();
    let component_pad_uuid_a = Uuid::new_v4();
    let track_uuid_b = Uuid::new_v4();
    let track_uuid_a = Uuid::new_v4();
    let via_uuid = Uuid::new_v4();
    let zone_uuid = Uuid::new_v4();
    let keepout_uuid_b = Uuid::new_v4();
    let keepout_uuid_a = Uuid::new_v4();

    let board = Board {
        uuid: Uuid::new_v4(),
        name: "Routing Substrate".to_string(),
        stackup: Stackup {
            layers: vec![
                StackupLayer {
                    id: 41,
                    name: "Mechanical 41".to_string(),
                    layer_type: StackupLayerType::Mechanical,
                    thickness_nm: 0,
                },
                StackupLayer {
                    id: 1,
                    name: "Top Copper".to_string(),
                    layer_type: StackupLayerType::Copper,
                    thickness_nm: 35_000,
                },
            ],
        },
        outline: Polygon {
            vertices: vec![
                Point { x: 0, y: 0 },
                Point { x: 1_000_000, y: 0 },
                Point {
                    x: 1_000_000,
                    y: 500_000,
                },
            ],
            closed: true,
        },
        packages: HashMap::from([
            (
                component_uuid_b,
                PlacedPackage {
                    uuid: component_uuid_b,
                    part: Uuid::new_v4(),
                    package: Uuid::new_v4(),
                    reference: "U2".to_string(),
                    value: "B".to_string(),
                    position: Point { x: 0, y: 0 },
                    rotation: 0,
                    layer: 1,
                    locked: false,
                },
            ),
            (
                component_uuid_a,
                PlacedPackage {
                    uuid: component_uuid_a,
                    part: Uuid::new_v4(),
                    package: Uuid::new_v4(),
                    reference: "U1".to_string(),
                    value: "A".to_string(),
                    position: Point { x: 0, y: 0 },
                    rotation: 0,
                    layer: 1,
                    locked: false,
                },
            ),
        ]),
        pads: HashMap::from([(
            board_pad_uuid,
            PlacedPad {
                uuid: board_pad_uuid,
                package: component_uuid_b,
                name: "TP1".to_string(),
                net: Some(net_uuid_b),
                position: Point { x: 20, y: 20 },
                layer: 1,
                shape: super::super::PadShape::Circle,
                diameter: 250_000,
                width: 0,
                height: 0,
            },
        )]),
        tracks: HashMap::from([
            (
                track_uuid_b,
                Track {
                    uuid: track_uuid_b,
                    net: net_uuid_b,
                    from: Point { x: 0, y: 0 },
                    to: Point { x: 10, y: 0 },
                    width: 120_000,
                    layer: 1,
                },
            ),
            (
                track_uuid_a,
                Track {
                    uuid: track_uuid_a,
                    net: net_uuid_a,
                    from: Point { x: 0, y: 10 },
                    to: Point { x: 10, y: 10 },
                    width: 120_000,
                    layer: 1,
                },
            ),
        ]),
        vias: HashMap::from([(
            via_uuid,
            Via {
                uuid: via_uuid,
                net: net_uuid_a,
                position: Point { x: 50, y: 50 },
                drill: 300_000,
                diameter: 600_000,
                from_layer: 1,
                to_layer: 2,
            },
        )]),
        zones: HashMap::from([(
            zone_uuid,
            Zone {
                uuid: zone_uuid,
                net: net_uuid_b,
                polygon: Polygon {
                    vertices: vec![
                        Point { x: 0, y: 0 },
                        Point { x: 100, y: 0 },
                        Point { x: 100, y: 100 },
                    ],
                    closed: true,
                },
                layer: 1,
                priority: 1,
                thermal_relief: true,
                thermal_gap: 150_000,
                thermal_spoke_width: 120_000,
            },
        )]),
        nets: HashMap::from([
            (
                net_uuid_b,
                Net {
                    uuid: net_uuid_b,
                    name: "SIG_B".to_string(),
                    class: net_class_uuid,
                },
            ),
            (
                net_uuid_a,
                Net {
                    uuid: net_uuid_a,
                    name: "SIG_A".to_string(),
                    class: net_class_uuid,
                },
            ),
        ]),
        net_classes: HashMap::from([(
            net_class_uuid,
            NetClass {
                uuid: net_class_uuid,
                name: "Default".to_string(),
                clearance: 150_000,
                track_width: 125_000,
                via_drill: 300_000,
                via_diameter: 600_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            },
        )]),
        rules: RuleSet::new(),
        keepouts: vec![
            Keepout {
                uuid: keepout_uuid_b,
                polygon: Polygon {
                    vertices: vec![
                        Point { x: 10, y: 10 },
                        Point { x: 20, y: 10 },
                        Point { x: 20, y: 20 },
                    ],
                    closed: true,
                },
                layers: vec![1],
                kind: "component".to_string(),
            },
            Keepout {
                uuid: keepout_uuid_a,
                polygon: Polygon {
                    vertices: vec![
                        Point { x: 30, y: 30 },
                        Point { x: 40, y: 30 },
                        Point { x: 40, y: 40 },
                    ],
                    closed: true,
                },
                layers: vec![1],
                kind: "board_edge".to_string(),
            },
        ],
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let substrate = board.routing_substrate(&[
        RoutingComponentPad {
            component_uuid: component_uuid_b,
            uuid: component_pad_uuid_b,
            name: "2".to_string(),
            position: Point { x: 200, y: 200 },
            padstack_uuid: Uuid::new_v4(),
            layer: 1,
            drill_nm: Some(350_000),
            shape: Some(super::super::PadShape::Circle),
            diameter_nm: 700_000,
            width_nm: 0,
            height_nm: 0,
        },
        RoutingComponentPad {
            component_uuid: component_uuid_a,
            uuid: component_pad_uuid_a,
            name: "1".to_string(),
            position: Point { x: 100, y: 100 },
            padstack_uuid: Uuid::new_v4(),
            layer: 1,
            drill_nm: None,
            shape: None,
            diameter_nm: 0,
            width_nm: 0,
            height_nm: 0,
        },
    ]);

    assert_eq!(substrate.contract, "m5_routing_substrate_v1");
    assert!(substrate.persisted_native_board_state_only);
    assert_eq!(substrate.summary.layer_count, 2);
    assert_eq!(substrate.summary.copper_layer_count, 1);
    assert_eq!(substrate.summary.keepout_count, 2);
    assert_eq!(substrate.summary.board_pad_count, 1);
    assert_eq!(substrate.summary.component_pad_count, 2);
    assert_eq!(substrate.summary.track_count, 2);
    assert_eq!(substrate.summary.via_count, 1);
    assert_eq!(substrate.summary.zone_count, 1);
    assert_eq!(substrate.summary.net_count, 2);
    assert_eq!(substrate.summary.net_class_count, 1);

    assert_eq!(substrate.layers[0].id, 1);
    assert_eq!(substrate.layers[1].id, 41);
    assert_eq!(substrate.copper_layer_ids, vec![1]);
    assert_eq!(substrate.keepouts[0].kind, "board_edge");
    assert_eq!(substrate.keepouts[1].kind, "component");

    assert_eq!(substrate.pads.len(), 3);
    assert!(matches!(
        substrate.pads[0].source,
        RoutingPadSource::BoardPad
    ));
    assert!(matches!(
        substrate.pads[1].source,
        RoutingPadSource::ComponentPad
    ));
    let component_pad_owners = substrate.pads[1..]
        .iter()
        .map(|pad| pad.owner_uuid)
        .collect::<Vec<_>>();
    assert!(component_pad_owners.contains(&component_uuid_a));
    assert!(component_pad_owners.contains(&component_uuid_b));
    assert!(
        substrate.pads[1..]
            .iter()
            .any(|pad| pad.drill_nm == Some(350_000))
    );

    let track_ids = substrate
        .tracks
        .iter()
        .map(|track| track.uuid)
        .collect::<Vec<_>>();
    assert!(track_ids.contains(&track_uuid_a));
    assert!(track_ids.contains(&track_uuid_b));
    assert_eq!(substrate.nets[0].name, "SIG_A");
    assert_eq!(substrate.nets[1].name, "SIG_B");
    assert_eq!(substrate.net_classes[0].name, "Default");
}
