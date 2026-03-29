use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x910);
    let other_net_uuid = Uuid::from_u128(0x911);
    let class_uuid = Uuid::from_u128(0x912);
    let anchor_top_uuid = Uuid::from_u128(0x913);
    let anchor_bottom_uuid = Uuid::from_u128(0x914);
    let via_uuid = Uuid::from_u128(0x915);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate-via".into(),
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
                        package: Uuid::from_u128(0x920),
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
                        package: Uuid::from_u128(0x921),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 900_000),
                        layer: 3,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::new(),
            vias: HashMap::from([(
                via_uuid,
                Via {
                    uuid: via_uuid,
                    net: net_uuid,
                    position: Point::new(500_000, 500_000),
                    drill: 300_000,
                    diameter: 600_000,
                    from_layer: 1,
                    to_layer: 3,
                },
            )]),
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
fn route_path_candidate_via_reports_deterministic_path_using_authored_via() {
    let (board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_uuid) = demo_board();

    let report = board
        .route_path_candidate_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("via path candidate should succeed");

    assert_eq!(report.status, RoutePathCandidateStatus::DeterministicPathFound);
    assert_eq!(report.summary.candidate_via_count, 1);
    assert_eq!(report.summary.matching_via_count, 1);
    assert_eq!(report.summary.available_via_count, 1);
    assert_eq!(report.path.as_ref().map(|path| path.via_uuid), Some(via_uuid));
    assert_eq!(report.path.as_ref().map(|path| path.segments.len()), Some(2));
    assert_eq!(report.path.as_ref().map(|path| path.segments[0].layer), Some(1));
    assert_eq!(report.path.as_ref().map(|path| path.segments[1].layer), Some(3));
}

#[test]
fn route_path_candidate_via_preserves_segment_orientation_for_reversed_anchor_order() {
    let (board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_uuid) = demo_board();

    let report = board
        .route_path_candidate_via(net_uuid, anchor_bottom_uuid, anchor_top_uuid)
        .expect("via path candidate should succeed");

    assert_eq!(report.status, RoutePathCandidateStatus::DeterministicPathFound);
    assert_eq!(report.path.as_ref().map(|path| path.via_uuid), Some(via_uuid));
    assert_eq!(report.path.as_ref().map(|path| path.segments[0].layer), Some(3));
    assert_eq!(report.path.as_ref().map(|path| path.segments[1].layer), Some(1));
    assert_eq!(
        report
            .path
            .as_ref()
            .map(|path| path.segments[0].points.clone()),
        Some(vec![Point::new(900_000, 900_000), Point::new(500_000, 500_000)])
    );
    assert_eq!(
        report
            .path
            .as_ref()
            .map(|path| path.segments[1].points.clone()),
        Some(vec![Point::new(500_000, 500_000), Point::new(100_000, 100_000)])
    );
}

#[test]
fn route_path_candidate_via_selects_next_matching_via_when_earlier_via_is_blocked() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, first_via_uuid) =
        demo_board();
    let second_via_uuid = Uuid::from_u128(0x916);
    board.vias.insert(
        second_via_uuid,
        Via {
            uuid: second_via_uuid,
            net: net_uuid,
            position: Point::new(700_000, 300_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 3,
        },
    );
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0x917),
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
        .route_path_candidate_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("via path candidate should succeed");

    assert_eq!(report.status, RoutePathCandidateStatus::DeterministicPathFound);
    assert_eq!(report.summary.candidate_via_count, 2);
    assert_eq!(report.summary.matching_via_count, 2);
    assert_eq!(report.summary.blocked_via_count, 1);
    assert_eq!(report.summary.available_via_count, 1);
    assert_eq!(report.path.as_ref().map(|path| path.via_uuid), Some(second_via_uuid));
    assert_ne!(report.path.as_ref().map(|path| path.via_uuid), Some(first_via_uuid));
}

#[test]
fn route_path_candidate_via_reports_no_path_when_matching_via_segments_are_blocked() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, _) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0x918),
        polygon: Polygon::new(vec![
            Point::new(300_000, 300_000),
            Point::new(700_000, 300_000),
            Point::new(700_000, 700_000),
            Point::new(300_000, 700_000),
        ]),
        layers: vec![1, 3],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("via path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.matching_via_count, 1);
    assert_eq!(report.summary.blocked_via_count, 1);
    assert_eq!(report.summary.available_via_count, 0);
    assert!(report.path.is_none());
}

#[test]
fn route_path_candidate_via_reports_no_path_when_no_authored_via_matches_anchor_layers() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_uuid) = demo_board();
    board.vias.insert(
        via_uuid,
        Via {
            uuid: via_uuid,
            net: net_uuid,
            position: Point::new(500_000, 500_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 1,
        },
    );

    let report = board
        .route_path_candidate_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("via path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.candidate_via_count, 1);
    assert_eq!(report.summary.matching_via_count, 0);
    assert_eq!(report.summary.blocked_via_count, 0);
    assert!(report.path.is_none());
}
