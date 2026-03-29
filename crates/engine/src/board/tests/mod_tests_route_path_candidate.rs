use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x10);
    let other_net_uuid = Uuid::from_u128(0x11);
    let class_uuid = Uuid::from_u128(0x12);
    let pkg_a = Uuid::from_u128(0x20);
    let pkg_b = Uuid::from_u128(0x21);
    let pkg_c = Uuid::from_u128(0x22);
    let anchor_a_uuid = Uuid::from_u128(0x30);
    let anchor_b_uuid = Uuid::from_u128(0x31);
    let anchor_c_uuid = Uuid::from_u128(0x32);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate".into(),
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
                    StackupLayer {
                        id: 3,
                        name: "Bottom".into(),
                        layer_type: StackupLayerType::Copper,
                        thickness_nm: 35_000,
                    },
                ],
            },
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(1_000_000, 0),
                Point::new(1_000_000, 1_000_000),
                Point::new(0, 1_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    anchor_a_uuid,
                    PlacedPad {
                        uuid: anchor_a_uuid,
                        package: pkg_a,
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(100_000, 100_000),
                        layer: 1,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
                (
                    anchor_b_uuid,
                    PlacedPad {
                        uuid: anchor_b_uuid,
                        package: pkg_b,
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
                    anchor_c_uuid,
                    PlacedPad {
                        uuid: anchor_c_uuid,
                        package: pkg_c,
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 900_000),
                        layer: 1,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::new(),
            vias: HashMap::new(),
            zones: HashMap::new(),
            nets: HashMap::from([
                (
                    net_uuid,
                    Net {
                        uuid: net_uuid,
                        name: "SIG".into(),
                        class: class_uuid,
                    },
                ),
                (
                    other_net_uuid,
                    Net {
                        uuid: other_net_uuid,
                        name: "OTHER".into(),
                        class: class_uuid,
                    },
                ),
            ]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 150_000,
                    track_width: 200_000,
                    via_drill: 300_000,
                    via_diameter: 600_000,
                    diffpair_width: 0,
                    diffpair_gap: 0,
                },
            )]),
            rules: Vec::new(),
            keepouts: Vec::new(),
            dimensions: Vec::new(),
            texts: Vec::new(),
        },
        net_uuid,
        other_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        anchor_c_uuid,
        class_uuid,
    )
}

#[test]
fn route_path_candidate_uses_explicit_first_unblocked_matching_span_rule() {
    let (board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, _, _) = demo_board();

    let report = board
        .route_path_candidate(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        report.selection_rule,
        "select the first unblocked matching corridor span in corridor report order (sorted by candidate copper layer order, then pair index)"
    );
    assert_eq!(report.summary.matching_span_count, 2);
    assert_eq!(report.summary.available_span_count, 2);
    assert_eq!(report.path.as_ref().map(|path| path.layer), Some(1));
    assert_eq!(
        report.path.as_ref().map(|path| path.points.clone()),
        Some(vec![
            Point::new(100_000, 100_000),
            Point::new(500_000, 500_000)
        ])
    );
}

#[test]
fn route_path_candidate_uses_selected_corridor_span_geometry_in_requested_anchor_order() {
    let (board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, _, _) = demo_board();

    let report = board
        .route_path_candidate(net_uuid, anchor_b_uuid, anchor_a_uuid)
        .expect("path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.path.as_ref().map(|path| path.layer), Some(1));
    assert_eq!(
        report.path.as_ref().map(|path| path.points.clone()),
        Some(vec![
            Point::new(500_000, 500_000),
            Point::new(100_000, 100_000)
        ])
    );
}

#[test]
fn route_path_candidate_reports_no_path_when_all_matching_spans_are_blocked() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, _, _) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::new_v4(),
        polygon: Polygon::new(vec![
            Point::new(450_000, 450_000),
            Point::new(550_000, 450_000),
            Point::new(550_000, 550_000),
            Point::new(450_000, 550_000),
        ]),
        layers: vec![1, 3],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.matching_span_count, 2);
    assert_eq!(report.summary.blocked_span_count, 2);
    assert!(report.path.is_none());
}

#[test]
fn route_path_candidate_selects_next_matching_span_when_earlier_corridor_span_is_blocked() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, _, _) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::new_v4(),
        polygon: Polygon::new(vec![
            Point::new(260_000, 200_000),
            Point::new(340_000, 200_000),
            Point::new(340_000, 420_000),
            Point::new(260_000, 420_000),
        ]),
        layers: vec![1],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.matching_span_count, 2);
    assert_eq!(report.summary.blocked_span_count, 1);
    assert_eq!(report.summary.available_span_count, 1);
    assert_eq!(report.path.as_ref().map(|path| path.layer), Some(3));
    assert_eq!(
        report.path.as_ref().map(|path| path.points.clone()),
        Some(vec![
            Point::new(100_000, 100_000),
            Point::new(500_000, 500_000)
        ])
    );
}

#[test]
fn route_path_candidate_stress_tests_explicit_tie_break_across_multiple_matching_layers() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, _, _) = demo_board();
    board.stackup.layers = vec![
        StackupLayer {
            id: 1,
            name: "Top".into(),
            layer_type: StackupLayerType::Copper,
            thickness_nm: 35_000,
        },
        StackupLayer {
            id: 2,
            name: "Core A".into(),
            layer_type: StackupLayerType::Dielectric,
            thickness_nm: 1_000_000,
        },
        StackupLayer {
            id: 3,
            name: "Inner 1".into(),
            layer_type: StackupLayerType::Copper,
            thickness_nm: 35_000,
        },
        StackupLayer {
            id: 4,
            name: "Core B".into(),
            layer_type: StackupLayerType::Dielectric,
            thickness_nm: 1_000_000,
        },
        StackupLayer {
            id: 5,
            name: "Inner 2".into(),
            layer_type: StackupLayerType::Copper,
            thickness_nm: 35_000,
        },
        StackupLayer {
            id: 6,
            name: "Core C".into(),
            layer_type: StackupLayerType::Dielectric,
            thickness_nm: 1_000_000,
        },
        StackupLayer {
            id: 7,
            name: "Bottom".into(),
            layer_type: StackupLayerType::Copper,
            thickness_nm: 35_000,
        },
    ];
    for layer in [1, 3] {
        board.keepouts.push(Keepout {
            uuid: Uuid::new_v4(),
            polygon: Polygon::new(vec![
                Point::new(260_000, 200_000),
                Point::new(340_000, 200_000),
                Point::new(340_000, 420_000),
                Point::new(260_000, 420_000),
            ]),
            layers: vec![layer],
            kind: "route".into(),
        });
    }

    let report = board
        .route_path_candidate(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.matching_span_count, 4);
    assert_eq!(report.summary.blocked_span_count, 2);
    assert_eq!(report.summary.available_span_count, 2);
    assert_eq!(report.path.as_ref().map(|path| path.layer), Some(5));
    assert_eq!(
        report.path.as_ref().map(|path| path.points.clone()),
        Some(vec![
            Point::new(100_000, 100_000),
            Point::new(500_000, 500_000)
        ])
    );
}

#[test]
fn route_path_candidate_reports_no_path_for_non_consecutive_pair_under_corridor_model() {
    let (board, net_uuid, _, anchor_a_uuid, _, anchor_c_uuid, _) = demo_board();

    let report = board
        .route_path_candidate(net_uuid, anchor_a_uuid, anchor_c_uuid)
        .expect("path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.matching_span_count, 0);
    assert!(report.path.is_none());
}

#[test]
fn route_path_candidate_honors_segment_polygon_truth_boundary_for_blocked_pair() {
    let (mut board, net_uuid, other_net_uuid, anchor_a_uuid, anchor_b_uuid, _, _) = demo_board();
    let top_zone_uuid = Uuid::new_v4();
    let bottom_zone_uuid = Uuid::new_v4();
    board.zones.insert(
        top_zone_uuid,
        Zone {
            uuid: top_zone_uuid,
            net: other_net_uuid,
            polygon: Polygon::new(vec![
                Point::new(260_000, 200_000),
                Point::new(340_000, 200_000),
                Point::new(340_000, 420_000),
                Point::new(260_000, 420_000),
            ]),
            layer: 1,
            priority: 1,
            thermal_relief: true,
            thermal_gap: 150_000,
            thermal_spoke_width: 120_000,
        },
    );
    board.zones.insert(
        bottom_zone_uuid,
        Zone {
            uuid: bottom_zone_uuid,
            net: other_net_uuid,
            polygon: Polygon::new(vec![
                Point::new(260_000, 200_000),
                Point::new(340_000, 200_000),
                Point::new(340_000, 420_000),
                Point::new(260_000, 420_000),
            ]),
            layer: 3,
            priority: 1,
            thermal_relief: true,
            thermal_gap: 150_000,
            thermal_spoke_width: 120_000,
        },
    );

    let report = board
        .route_path_candidate(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.blocked_span_count, 2);
    assert!(report.path.is_none());
}
