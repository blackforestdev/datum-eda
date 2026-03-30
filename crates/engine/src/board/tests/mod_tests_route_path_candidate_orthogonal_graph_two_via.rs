use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

pub(super) fn orthogonal_graph_two_via_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9500);
    let other_net_uuid = Uuid::from_u128(0x9501);
    let class_uuid = Uuid::from_u128(0x9502);
    let anchor_top_uuid = Uuid::from_u128(0x9503);
    let anchor_bottom_uuid = Uuid::from_u128(0x9504);
    let via_a_uuid = Uuid::from_u128(0x9505);
    let via_b_uuid = Uuid::from_u128(0x9506);
    let guide_via_top_uuid = Uuid::from_u128(0x9507);
    let guide_via_mid_uuid = Uuid::from_u128(0x9508);
    let guide_via_bottom_uuid = Uuid::from_u128(0x9509);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "orthogonal-graph-two-via".into(),
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
                        name: "Core A".into(),
                        layer_type: StackupLayerType::Dielectric,
                        thickness_nm: 1_000_000,
                    },
                    StackupLayer {
                        id: 3,
                        name: "Inner".into(),
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
                        package: Uuid::from_u128(0x9520),
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
                        package: Uuid::from_u128(0x9521),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(100_000, 900_000),
                        layer: 5,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::from([
                (
                    Uuid::from_u128(0x9510),
                    Track {
                        uuid: Uuid::from_u128(0x9510),
                        net: other_net_uuid,
                        from: Point::new(0, 300_000),
                        to: Point::new(800_000, 300_000),
                        width: 150_000,
                        layer: 1,
                    },
                ),
                (
                    Uuid::from_u128(0x9511),
                    Track {
                        uuid: Uuid::from_u128(0x9511),
                        net: other_net_uuid,
                        from: Point::new(200_000, 500_000),
                        to: Point::new(1_000_000, 500_000),
                        width: 150_000,
                        layer: 3,
                    },
                ),
                (
                    Uuid::from_u128(0x9512),
                    Track {
                        uuid: Uuid::from_u128(0x9512),
                        net: other_net_uuid,
                        from: Point::new(0, 700_000),
                        to: Point::new(800_000, 700_000),
                        width: 150_000,
                        layer: 5,
                    },
                ),
            ]),
            vias: HashMap::from([
                (
                    via_a_uuid,
                    Via {
                        uuid: via_a_uuid,
                        net: net_uuid,
                        position: Point::new(900_000, 300_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 1,
                        to_layer: 3,
                    },
                ),
                (
                    via_b_uuid,
                    Via {
                        uuid: via_b_uuid,
                        net: net_uuid,
                        position: Point::new(900_000, 700_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 3,
                        to_layer: 5,
                    },
                ),
                (
                    guide_via_top_uuid,
                    Via {
                        uuid: guide_via_top_uuid,
                        net: net_uuid,
                        position: Point::new(900_000, 100_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 1,
                        to_layer: 2,
                    },
                ),
                (
                    guide_via_mid_uuid,
                    Via {
                        uuid: guide_via_mid_uuid,
                        net: net_uuid,
                        position: Point::new(100_000, 300_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 2,
                        to_layer: 3,
                    },
                ),
                (
                    guide_via_bottom_uuid,
                    Via {
                        uuid: guide_via_bottom_uuid,
                        net: net_uuid,
                        position: Point::new(100_000, 700_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 4,
                        to_layer: 5,
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
        via_a_uuid,
        via_b_uuid,
    )
}

#[test]
fn route_path_candidate_orthogonal_graph_two_via_reports_deterministic_path_using_authored_via_pair()
 {
    let (board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_a_uuid, via_b_uuid) =
        orthogonal_graph_two_via_board();

    let report = board
        .route_path_candidate_orthogonal_graph_two_via(
            net_uuid,
            anchor_top_uuid,
            anchor_bottom_uuid,
        )
        .expect("orthogonal graph two-via should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.candidate_via_count, 5);
    assert_eq!(report.summary.matching_via_pair_count, 1);
    assert_eq!(report.summary.available_via_pair_count, 1);
    assert_eq!(
        report.path.as_ref().map(|path| path.via_a_uuid),
        Some(via_a_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_b_uuid),
        Some(via_b_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.segments.len()),
        Some(3)
    );
}

#[test]
fn route_path_candidate_orthogonal_graph_two_via_reports_no_path_when_middle_graph_is_cut() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, _, _) =
        orthogonal_graph_two_via_board();
    board.tracks.insert(
        Uuid::from_u128(0x9513),
        Track {
            uuid: Uuid::from_u128(0x9513),
            net: Uuid::from_u128(0x9501),
            from: Point::new(0, 300_000),
            to: Point::new(1_000_000, 300_000),
            width: 150_000,
            layer: 3,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_graph_two_via(
            net_uuid,
            anchor_top_uuid,
            anchor_bottom_uuid,
        )
        .expect("orthogonal graph two-via should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.matching_via_pair_count, 1);
    assert_eq!(report.summary.available_via_pair_count, 0);
    assert!(report.path.is_none());
}
