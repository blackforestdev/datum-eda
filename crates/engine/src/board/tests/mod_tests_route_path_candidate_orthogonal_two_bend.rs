use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

pub(super) fn orthogonal_two_bend_board() -> (Board, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9200);
    let other_net_uuid = Uuid::from_u128(0x9201);
    let class_uuid = Uuid::from_u128(0x9202);
    let pkg_a = Uuid::from_u128(0x9203);
    let pkg_b = Uuid::from_u128(0x9204);
    let anchor_a_uuid = Uuid::from_u128(0x9205);
    let anchor_b_uuid = Uuid::from_u128(0x9206);
    let left_block_uuid = Uuid::from_u128(0x9207);
    let right_block_uuid = Uuid::from_u128(0x9208);

    (
        Board {
            uuid: Uuid::new_v4(),
            name: "orthogonal-two-bend".into(),
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
                    left_block_uuid,
                    Track {
                        uuid: left_block_uuid,
                        net: other_net_uuid,
                        from: Point::new(100_000, 300_000),
                        to: Point::new(100_000, 700_000),
                        width: 150_000,
                        layer: 1,
                    },
                ),
                (
                    right_block_uuid,
                    Track {
                        uuid: right_block_uuid,
                        net: other_net_uuid,
                        from: Point::new(900_000, 300_000),
                        to: Point::new(900_000, 700_000),
                        width: 150_000,
                        layer: 1,
                    },
                ),
            ]),
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
    )
}

#[test]
fn route_path_candidate_orthogonal_two_bend_finds_first_unblocked_detour_path() {
    let (board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_two_bend_board();

    let report = board
        .route_path_candidate_orthogonal_two_bend(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal two-bend should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.available_path_count, 1);
    assert_eq!(
        report.path.as_ref().map(|path| path.points.clone()),
        Some(vec![
            Point::new(100_000, 100_000),
            Point::new(0, 100_000),
            Point::new(0, 900_000),
            Point::new(900_000, 900_000),
        ])
    );
}

#[test]
fn route_path_candidate_orthogonal_two_bend_prefers_horizontal_detour_with_smallest_coordinate_when_clear()
{
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_two_bend_board();
    board.tracks.clear();

    let report = board
        .route_path_candidate_orthogonal_two_bend(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal two-bend should succeed");

    assert_eq!(
        report.path.as_ref().map(|path| path.detour_coordinate),
        Some(0)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.points.clone()),
        Some(vec![
            Point::new(100_000, 100_000),
            Point::new(100_000, 0),
            Point::new(900_000, 0),
            Point::new(900_000, 900_000),
        ])
    );
}

#[test]
fn route_path_candidate_orthogonal_two_bend_reports_no_path_when_all_candidates_are_blocked() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_two_bend_board();
    board.tracks.insert(
        Uuid::from_u128(0x9209),
        Track {
            uuid: Uuid::from_u128(0x9209),
            net: Uuid::from_u128(0x9201),
            from: Point::new(300_000, 900_000),
            to: Point::new(700_000, 900_000),
            width: 150_000,
            layer: 1,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_two_bend(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal two-bend should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert!(report.summary.candidate_path_count > 0);
    assert_eq!(report.summary.available_path_count, 0);
    assert!(report.path.is_none());
}
