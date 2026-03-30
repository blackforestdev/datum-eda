use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

pub(super) fn orthogonal_graph_via_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9400);
    let other_net_uuid = Uuid::from_u128(0x9401);
    let class_uuid = Uuid::from_u128(0x9402);
    let top_pkg_uuid = Uuid::from_u128(0x9403);
    let bottom_pkg_uuid = Uuid::from_u128(0x9404);
    let anchor_top_uuid = Uuid::from_u128(0x9405);
    let anchor_bottom_uuid = Uuid::from_u128(0x9406);
    let via_uuid = Uuid::from_u128(0x9407);
    let top_wall_a_uuid = Uuid::from_u128(0x9408);
    let top_wall_b_uuid = Uuid::from_u128(0x9409);
    let bottom_wall_a_uuid = Uuid::from_u128(0x940a);
    let bottom_wall_b_uuid = Uuid::from_u128(0x940b);
    let guide_via_top_uuid = Uuid::from_u128(0x940c);
    let guide_via_bottom_uuid = Uuid::from_u128(0x940d);

    (
        Board {
            uuid: Uuid::new_v4(),
            name: "orthogonal-graph-via".into(),
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
                    anchor_top_uuid,
                    PlacedPad {
                        uuid: anchor_top_uuid,
                        package: top_pkg_uuid,
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
                    anchor_bottom_uuid,
                    PlacedPad {
                        uuid: anchor_bottom_uuid,
                        package: bottom_pkg_uuid,
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(100_000, 900_000),
                        layer: 3,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::from([
                (
                    top_wall_a_uuid,
                    Track {
                        uuid: top_wall_a_uuid,
                        net: other_net_uuid,
                        from: Point::new(0, 300_000),
                        to: Point::new(800_000, 300_000),
                        width: 150_000,
                        layer: 1,
                    },
                ),
                (
                    top_wall_b_uuid,
                    Track {
                        uuid: top_wall_b_uuid,
                        net: other_net_uuid,
                        from: Point::new(200_000, 500_000),
                        to: Point::new(800_000, 500_000),
                        width: 150_000,
                        layer: 1,
                    },
                ),
                (
                    bottom_wall_a_uuid,
                    Track {
                        uuid: bottom_wall_a_uuid,
                        net: other_net_uuid,
                        from: Point::new(0, 500_000),
                        to: Point::new(800_000, 500_000),
                        width: 150_000,
                        layer: 3,
                    },
                ),
                (
                    bottom_wall_b_uuid,
                    Track {
                        uuid: bottom_wall_b_uuid,
                        net: other_net_uuid,
                        from: Point::new(200_000, 700_000),
                        to: Point::new(1_000_000, 700_000),
                        width: 150_000,
                        layer: 3,
                    },
                ),
            ]),
            vias: HashMap::from([
                (
                    via_uuid,
                    Via {
                        uuid: via_uuid,
                        net: net_uuid,
                        position: Point::new(900_000, 500_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 1,
                        to_layer: 3,
                    },
                ),
                (
                    guide_via_top_uuid,
                    Via {
                        uuid: guide_via_top_uuid,
                        net: net_uuid,
                        position: Point::new(950_000, 400_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 1,
                        to_layer: 2,
                    },
                ),
                (
                    guide_via_bottom_uuid,
                    Via {
                        uuid: guide_via_bottom_uuid,
                        net: net_uuid,
                        position: Point::new(50_000, 600_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 2,
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
        anchor_top_uuid,
        anchor_bottom_uuid,
        via_uuid,
    )
}

#[test]
fn route_path_candidate_orthogonal_graph_via_reports_deterministic_path_using_authored_via() {
    let (board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_uuid) =
        orthogonal_graph_via_board();

    let report = board
        .route_path_candidate_orthogonal_graph_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("orthogonal graph via should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.candidate_via_count, 3);
    assert_eq!(report.summary.matching_via_count, 1);
    assert_eq!(report.summary.available_via_count, 1);
    assert_eq!(
        report.path.as_ref().map(|path| path.via_uuid),
        Some(via_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.segments.len()),
        Some(2)
    );
    assert_eq!(
        report
            .path
            .as_ref()
            .map(|path| path.segments[0].points.clone()),
        Some(vec![
            Point::new(100_000, 100_000),
            Point::new(900_000, 100_000),
            Point::new(900_000, 500_000),
        ])
    );
    assert_eq!(
        report
            .path
            .as_ref()
            .map(|path| path.segments[1].points.clone()),
        Some(vec![
            Point::new(900_000, 500_000),
            Point::new(900_000, 600_000),
            Point::new(100_000, 600_000),
            Point::new(100_000, 900_000),
        ])
    );
}

#[test]
fn route_path_candidate_orthogonal_graph_via_reports_no_path_when_matching_via_sides_are_cut() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, _) =
        orthogonal_graph_via_board();
    board.tracks.insert(
        Uuid::from_u128(0x940e),
        Track {
            uuid: Uuid::from_u128(0x940e),
            net: Uuid::from_u128(0x9401),
            from: Point::new(0, 600_000),
            to: Point::new(1_000_000, 600_000),
            width: 150_000,
            layer: 3,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_graph_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("orthogonal graph via should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.matching_via_count, 1);
    assert_eq!(report.summary.available_via_count, 0);
    assert!(report.path.is_none());
}

#[test]
fn route_path_candidate_orthogonal_graph_via_reports_no_path_when_no_matching_authored_via_exists()
{
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_uuid) =
        orthogonal_graph_via_board();
    board.vias.remove(&via_uuid);

    let report = board
        .route_path_candidate_orthogonal_graph_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("orthogonal graph via should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.matching_via_count, 0);
    assert_eq!(report.summary.available_via_count, 0);
    assert!(report.path.is_none());
}
