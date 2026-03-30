use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

pub(super) fn orthogonal_graph_board() -> (Board, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9300);
    let other_net_uuid = Uuid::from_u128(0x9301);
    let class_uuid = Uuid::from_u128(0x9302);
    let pkg_a = Uuid::from_u128(0x9303);
    let pkg_b = Uuid::from_u128(0x9304);
    let anchor_a_uuid = Uuid::from_u128(0x9305);
    let anchor_b_uuid = Uuid::from_u128(0x9306);
    let wall_a_uuid = Uuid::from_u128(0x9307);
    let wall_b_uuid = Uuid::from_u128(0x9308);
    let wall_c_uuid = Uuid::from_u128(0x9309);
    let guide_via_a_uuid = Uuid::from_u128(0x930a);
    let guide_via_b_uuid = Uuid::from_u128(0x930b);
    let guide_via_c_uuid = Uuid::from_u128(0x930d);

    (
        Board {
            uuid: Uuid::new_v4(),
            name: "orthogonal-graph".into(),
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
                        position: Point::new(900_000, 900_000),
                        layer: 1,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::from([
                (
                    wall_a_uuid,
                    Track {
                        uuid: wall_a_uuid,
                        net: other_net_uuid,
                        from: Point::new(0, 300_000),
                        to: Point::new(900_000, 300_000),
                        width: 150_000,
                        layer: 1,
                    },
                ),
                (
                    wall_b_uuid,
                    Track {
                        uuid: wall_b_uuid,
                        net: other_net_uuid,
                        from: Point::new(200_000, 500_000),
                        to: Point::new(1_000_000, 500_000),
                        width: 150_000,
                        layer: 1,
                    },
                ),
                (
                    wall_c_uuid,
                    Track {
                        uuid: wall_c_uuid,
                        net: other_net_uuid,
                        from: Point::new(0, 700_000),
                        to: Point::new(800_000, 700_000),
                        width: 150_000,
                        layer: 1,
                    },
                ),
            ]),
            vias: HashMap::from([
                (
                    guide_via_a_uuid,
                    Via {
                        uuid: guide_via_a_uuid,
                        net: net_uuid,
                        position: Point::new(100_000, 400_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 1,
                        to_layer: 3,
                    },
                ),
                (
                    guide_via_b_uuid,
                    Via {
                        uuid: guide_via_b_uuid,
                        net: net_uuid,
                        position: Point::new(100_000, 600_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 1,
                        to_layer: 3,
                    },
                ),
                (
                    guide_via_c_uuid,
                    Via {
                        uuid: guide_via_c_uuid,
                        net: net_uuid,
                        position: Point::new(950_000, 400_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 1,
                        to_layer: 3,
                    },
                ),
            ]),
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
    )
}

#[test]
fn route_path_candidate_orthogonal_graph_finds_multi_segment_same_layer_path() {
    let (board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_graph_board();

    let report = board
        .route_path_candidate_orthogonal_graph(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal graph should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert!(report.summary.graph_node_count > 0);
    assert!(report.summary.graph_edge_count > 0);
    assert_eq!(
        report.path.as_ref().map(|path| path.points.clone()),
        Some(vec![
            Point::new(100_000, 100_000),
            Point::new(950_000, 100_000),
            Point::new(950_000, 400_000),
            Point::new(0, 400_000),
            Point::new(0, 600_000),
            Point::new(900_000, 600_000),
            Point::new(900_000, 900_000),
        ])
    );
}

#[test]
fn route_path_candidate_orthogonal_graph_prefers_fewer_segments_when_direct_span_is_clear() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_graph_board();
    board.tracks.clear();

    let report = board
        .route_path_candidate_orthogonal_graph(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal graph should succeed");

    assert_eq!(
        report.path.as_ref().map(|path| path.points.len()),
        Some(3)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.points.clone()),
        Some(vec![
            Point::new(100_000, 100_000),
            Point::new(900_000, 100_000),
            Point::new(900_000, 900_000),
        ])
    );
}

#[test]
fn route_path_candidate_orthogonal_graph_reports_no_path_when_graph_is_cut() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_graph_board();
    board.tracks.insert(
        Uuid::from_u128(0x930c),
        Track {
            uuid: Uuid::from_u128(0x930c),
            net: Uuid::from_u128(0x9301),
            from: Point::new(0, 600_000),
            to: Point::new(1_000_000, 600_000),
            width: 150_000,
            layer: 1,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_graph(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal graph should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert!(report.summary.graph_node_count > 0);
    assert!(report.summary.blocked_edge_count > 0);
    assert!(report.path.is_none());
}
