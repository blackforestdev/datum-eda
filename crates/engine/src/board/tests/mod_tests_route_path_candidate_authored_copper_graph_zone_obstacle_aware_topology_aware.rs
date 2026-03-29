use std::collections::HashMap;

use uuid::Uuid;

use crate::board::*;
use crate::ir::geometry::{Point, Polygon};

#[test]
fn topology_aware_authored_copper_graph_prefers_fewer_topology_transitions_for_equal_step_count() {
    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let from_pad_uuid = Uuid::new_v4();
    let to_pad_uuid = Uuid::new_v4();
    let top_track_uuid = Uuid::from_u128(1);
    let anchor_via_uuid = Uuid::from_u128(2);
    let lower_track_first_uuid = Uuid::from_u128(3);
    let lower_track_second_uuid = Uuid::from_u128(4);
    let alternate_via_uuid = Uuid::from_u128(5);
    let alternate_lower_track_uuid = Uuid::from_u128(6);

    let board = Board {
        uuid: Uuid::new_v4(),
        name: "topology-aware".into(),
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
                    name: "Inner".into(),
                    layer_type: StackupLayerType::Copper,
                    thickness_nm: 35_000,
                },
            ],
        },
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(5_000_000, 0),
            Point::new(5_000_000, 5_000_000),
            Point::new(0, 5_000_000),
        ]),
        packages: HashMap::new(),
        pads: HashMap::from([
            (
                from_pad_uuid,
                PlacedPad {
                    uuid: from_pad_uuid,
                    package: Uuid::new_v4(),
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(500_000, 500_000),
                    layer: 1,
                    shape: PadShape::Circle,
                    diameter: 300_000,
                    width: 0,
                    height: 0,
                },
            ),
            (
                to_pad_uuid,
                PlacedPad {
                    uuid: to_pad_uuid,
                    package: Uuid::new_v4(),
                    name: "2".into(),
                    net: Some(net_uuid),
                    position: Point::new(3_500_000, 500_000),
                    layer: 2,
                    shape: PadShape::Circle,
                    diameter: 300_000,
                    width: 0,
                    height: 0,
                },
            ),
        ]),
        tracks: HashMap::from([
            (
                top_track_uuid,
                Track {
                    uuid: top_track_uuid,
                    net: net_uuid,
                    from: Point::new(500_000, 500_000),
                    to: Point::new(1_500_000, 500_000),
                    width: 120_000,
                    layer: 1,
                },
            ),
            (
                lower_track_first_uuid,
                Track {
                    uuid: lower_track_first_uuid,
                    net: net_uuid,
                    from: Point::new(500_000, 500_000),
                    to: Point::new(2_000_000, 500_000),
                    width: 120_000,
                    layer: 2,
                },
            ),
            (
                lower_track_second_uuid,
                Track {
                    uuid: lower_track_second_uuid,
                    net: net_uuid,
                    from: Point::new(2_000_000, 500_000),
                    to: Point::new(3_500_000, 500_000),
                    width: 120_000,
                    layer: 2,
                },
            ),
            (
                alternate_lower_track_uuid,
                Track {
                    uuid: alternate_lower_track_uuid,
                    net: net_uuid,
                    from: Point::new(1_500_000, 500_000),
                    to: Point::new(3_500_000, 500_000),
                    width: 120_000,
                    layer: 2,
                },
            ),
        ]),
        vias: HashMap::from([
            (
                anchor_via_uuid,
                Via {
                    uuid: anchor_via_uuid,
                    net: net_uuid,
                    position: Point::new(500_000, 500_000),
                    from_layer: 1,
                    to_layer: 2,
                    diameter: 300_000,
                    drill: 150_000,
                },
            ),
            (
                alternate_via_uuid,
                Via {
                    uuid: alternate_via_uuid,
                    net: net_uuid,
                    position: Point::new(1_500_000, 500_000),
                    from_layer: 1,
                    to_layer: 2,
                    diameter: 300_000,
                    drill: 150_000,
                },
            ),
        ]),
        zones: HashMap::new(),
        nets: HashMap::from([(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: class_uuid,
            },
        )]),
        net_classes: HashMap::from([(
            class_uuid,
            NetClass {
                uuid: class_uuid,
                name: "Default".into(),
                clearance: 100_000,
                track_width: 120_000,
                via_drill: 150_000,
                via_diameter: 300_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            },
        )]),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let report = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("topology-aware query should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.path_step_count, 3);
    assert_eq!(report.summary.topology_transition_count, 1);
    assert_eq!(report.summary.path_via_step_count, 1);
    let path = report.path.expect("path should exist");
    assert_eq!(path.steps[0].object_uuid, anchor_via_uuid);
    assert_eq!(path.steps[1].object_uuid, lower_track_first_uuid);
    assert_eq!(path.steps[2].object_uuid, lower_track_second_uuid);
}

#[test]
fn topology_aware_authored_copper_graph_reports_no_path_when_all_paths_are_blocked() {
    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let from_pad_uuid = Uuid::new_v4();
    let to_pad_uuid = Uuid::new_v4();

    let track_uuid = Uuid::new_v4();
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "topology-aware-blocked".into(),
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
            Point::new(4_000_000, 0),
            Point::new(4_000_000, 4_000_000),
            Point::new(0, 4_000_000),
        ]),
        packages: HashMap::new(),
        pads: HashMap::from([
            (
                from_pad_uuid,
                PlacedPad {
                    uuid: from_pad_uuid,
                    package: Uuid::new_v4(),
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(500_000, 500_000),
                    layer: 1,
                    shape: PadShape::Circle,
                    diameter: 300_000,
                    width: 0,
                    height: 0,
                },
            ),
            (
                to_pad_uuid,
                PlacedPad {
                    uuid: to_pad_uuid,
                    package: Uuid::new_v4(),
                    name: "2".into(),
                    net: Some(net_uuid),
                    position: Point::new(3_500_000, 500_000),
                    layer: 1,
                    shape: PadShape::Circle,
                    diameter: 300_000,
                    width: 0,
                    height: 0,
                },
            ),
        ]),
        tracks: HashMap::from([(
            track_uuid,
            Track {
                uuid: track_uuid,
                net: net_uuid,
                from: Point::new(500_000, 500_000),
                to: Point::new(3_500_000, 500_000),
                width: 120_000,
                layer: 1,
            },
        )]),
        vias: HashMap::new(),
        zones: HashMap::new(),
        nets: HashMap::from([(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: class_uuid,
            },
        )]),
        net_classes: HashMap::from([(
            class_uuid,
            NetClass {
                uuid: class_uuid,
                name: "Default".into(),
                clearance: 100_000,
                track_width: 120_000,
                via_drill: 150_000,
                via_diameter: 300_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            },
        )]),
        rules: Vec::new(),
        keepouts: vec![Keepout {
            uuid: Uuid::new_v4(),
            polygon: Polygon::new(vec![
                Point::new(1_500_000, 400_000),
                Point::new(2_500_000, 400_000),
                Point::new(2_500_000, 600_000),
                Point::new(1_500_000, 600_000),
            ]),
            layers: vec![1],
            kind: "route".into(),
        }],
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let report = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("topology-aware query should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.blocked_track_count, 1);
    assert!(report.path.is_none());
}
