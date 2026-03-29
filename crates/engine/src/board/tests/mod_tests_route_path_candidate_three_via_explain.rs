use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0xf50);
    let other_net_uuid = Uuid::from_u128(0xf51);
    let class_uuid = Uuid::from_u128(0xf52);
    let anchor_top_uuid = Uuid::from_u128(0xf53);
    let anchor_bottom_uuid = Uuid::from_u128(0xf54);
    let via_a_uuid = Uuid::from_u128(0xf55);
    let via_b_uuid = Uuid::from_u128(0xf56);
    let via_c_uuid = Uuid::from_u128(0xf57);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate-three-via-explain".into(),
            stackup: Stackup {
                layers: vec![
                    StackupLayer { id: 1, name: "Top".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                    StackupLayer { id: 2, name: "Core A".into(), layer_type: StackupLayerType::Dielectric, thickness_nm: 1_000_000 },
                    StackupLayer { id: 3, name: "Inner 1".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                    StackupLayer { id: 4, name: "Core B".into(), layer_type: StackupLayerType::Dielectric, thickness_nm: 1_000_000 },
                    StackupLayer { id: 5, name: "Inner 2".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                    StackupLayer { id: 6, name: "Core C".into(), layer_type: StackupLayerType::Dielectric, thickness_nm: 1_000_000 },
                    StackupLayer { id: 7, name: "Bottom".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
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
                        package: Uuid::from_u128(0xf60),
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
                        package: Uuid::from_u128(0xf61),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 900_000),
                        layer: 7,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::new(),
            vias: HashMap::from([
                (
                    via_a_uuid,
                    Via {
                        uuid: via_a_uuid,
                        net: net_uuid,
                        position: Point::new(250_000, 250_000),
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
                        position: Point::new(500_000, 500_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 3,
                        to_layer: 5,
                    },
                ),
                (
                    via_c_uuid,
                    Via {
                        uuid: via_c_uuid,
                        net: net_uuid,
                        position: Point::new(750_000, 750_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 5,
                        to_layer: 7,
                    },
                ),
            ]),
            zones: HashMap::new(),
            nets: HashMap::from([
                (
                    net_uuid,
                    Net { uuid: net_uuid, name: "SIG".into(), class: class_uuid },
                ),
                (
                    other_net_uuid,
                    Net { uuid: other_net_uuid, name: "OTHER".into(), class: class_uuid },
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
        via_c_uuid,
    )
}

#[test]
fn route_path_candidate_three_via_explain_reports_selected_triple_for_found_path() {
    let (board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_a_uuid, via_b_uuid, via_c_uuid) =
        demo_board();

    let report = board
        .route_path_candidate_three_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("three-via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateThreeViaExplainKind::DeterministicPathFound
    );
    assert_eq!(report.status, RoutePathCandidateStatus::DeterministicPathFound);
    assert_eq!(report.summary.matching_via_triple_count, 1);
    assert_eq!(report.selected_triple.as_ref().map(|entry| entry.via_a_uuid), Some(via_a_uuid));
    assert_eq!(report.selected_triple.as_ref().map(|entry| entry.via_b_uuid), Some(via_b_uuid));
    assert_eq!(report.selected_triple.as_ref().map(|entry| entry.via_c_uuid), Some(via_c_uuid));
    assert_eq!(
        report
            .selected_triple
            .as_ref()
            .map(|entry| entry.first_middle_segment.points.clone()),
        Some(vec![Point::new(250_000, 250_000), Point::new(500_000, 500_000)])
    );
    assert_eq!(
        report
            .selected_triple
            .as_ref()
            .map(|entry| entry.second_middle_segment.points.clone()),
        Some(vec![Point::new(500_000, 500_000), Point::new(750_000, 750_000)])
    );
}

#[test]
fn route_path_candidate_three_via_explain_reports_no_matching_authored_via_triple() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_a_uuid, via_b_uuid, via_c_uuid) =
        demo_board();
    board.vias.insert(
        via_a_uuid,
        Via {
            uuid: via_a_uuid,
            net: net_uuid,
            position: Point::new(250_000, 250_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 5,
        },
    );
    board.vias.insert(
        via_b_uuid,
        Via {
            uuid: via_b_uuid,
            net: net_uuid,
            position: Point::new(500_000, 500_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 3,
            to_layer: 7,
        },
    );
    board.vias.insert(
        via_c_uuid,
        Via {
            uuid: via_c_uuid,
            net: net_uuid,
            position: Point::new(750_000, 750_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 7,
        },
    );

    let report = board
        .route_path_candidate_three_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("three-via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateThreeViaExplainKind::NoMatchingAuthoredViaTriple
    );
    assert!(report.selected_triple.is_none());
    assert!(report.blocked_matching_triples.is_empty());
}

#[test]
fn route_path_candidate_three_via_explain_reports_blocked_matching_triples_and_reasons() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, _, _, _) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0xf58),
        polygon: Polygon::new(vec![
            Point::new(450_000, 450_000),
            Point::new(550_000, 450_000),
            Point::new(550_000, 550_000),
            Point::new(450_000, 550_000),
        ]),
        layers: vec![5],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate_three_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("three-via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateThreeViaExplainKind::AllMatchingViaTriplesBlocked
    );
    assert_eq!(report.summary.blocked_via_triple_count, 1);
    assert_eq!(report.blocked_matching_triples.len(), 1);
    assert!(report.selected_triple.is_none());
    assert!(!report.blocked_matching_triples[0]
        .second_middle_blockages
        .is_empty());
}
