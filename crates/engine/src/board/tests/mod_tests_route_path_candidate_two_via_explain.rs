use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0xf10);
    let other_net_uuid = Uuid::from_u128(0xf11);
    let class_uuid = Uuid::from_u128(0xf12);
    let anchor_top_uuid = Uuid::from_u128(0xf13);
    let anchor_bottom_uuid = Uuid::from_u128(0xf14);
    let via_a_uuid = Uuid::from_u128(0xf15);
    let via_b_uuid = Uuid::from_u128(0xf16);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate-two-via-explain".into(),
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
                        package: Uuid::from_u128(0xf20),
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
                        package: Uuid::from_u128(0xf21),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 900_000),
                        layer: 5,
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
                        position: Point::new(300_000, 300_000),
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
                        position: Point::new(700_000, 700_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 3,
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
fn route_path_candidate_two_via_explain_reports_selected_pair_for_found_path() {
    let (board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_a_uuid, via_b_uuid) =
        demo_board();

    let report = board
        .route_path_candidate_two_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("two-via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateTwoViaExplainKind::DeterministicPathFound
    );
    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.matching_via_pair_count, 1);
    assert_eq!(
        report.selected_pair.as_ref().map(|entry| entry.via_a_uuid),
        Some(via_a_uuid)
    );
    assert_eq!(
        report.selected_pair.as_ref().map(|entry| entry.via_b_uuid),
        Some(via_b_uuid)
    );
    assert_eq!(
        report
            .selected_pair
            .as_ref()
            .map(|entry| entry.intermediate_layer),
        Some(3)
    );
    assert_eq!(
        report
            .selected_pair
            .as_ref()
            .map(|entry| entry.middle_segment.points.clone()),
        Some(vec![
            Point::new(300_000, 300_000),
            Point::new(700_000, 700_000)
        ])
    );
}

#[test]
fn route_path_candidate_two_via_explain_reports_no_matching_authored_via_pair() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_a_uuid, via_b_uuid) =
        demo_board();
    board.vias.insert(
        via_a_uuid,
        Via {
            uuid: via_a_uuid,
            net: net_uuid,
            position: Point::new(300_000, 300_000),
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
            position: Point::new(700_000, 700_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 3,
        },
    );

    let report = board
        .route_path_candidate_two_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("two-via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateTwoViaExplainKind::NoMatchingAuthoredViaPair
    );
    assert!(report.selected_pair.is_none());
    assert!(report.blocked_matching_pairs.is_empty());
}

#[test]
fn route_path_candidate_two_via_explain_reports_blocked_matching_pairs_and_reasons() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, _, _) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0xf17),
        polygon: Polygon::new(vec![
            Point::new(450_000, 450_000),
            Point::new(550_000, 450_000),
            Point::new(550_000, 550_000),
            Point::new(450_000, 550_000),
        ]),
        layers: vec![3],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate_two_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("two-via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateTwoViaExplainKind::AllMatchingViaPairsBlocked
    );
    assert_eq!(report.summary.blocked_via_pair_count, 1);
    assert_eq!(report.blocked_matching_pairs.len(), 1);
    assert!(report.selected_pair.is_none());
    assert!(!report.blocked_matching_pairs[0].middle_blockages.is_empty());
}

#[test]
fn route_path_candidate_two_via_explain_selected_pair_matches_fallback_path_candidate_pair() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, first_via_a_uuid, via_b_uuid) =
        demo_board();
    let second_via_a_uuid = Uuid::from_u128(0xf18);
    board.vias.insert(
        second_via_a_uuid,
        Via {
            uuid: second_via_a_uuid,
            net: net_uuid,
            position: Point::new(200_000, 600_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 3,
        },
    );
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0xf19),
        polygon: Polygon::new(vec![
            Point::new(250_000, 250_000),
            Point::new(350_000, 250_000),
            Point::new(350_000, 350_000),
            Point::new(250_000, 350_000),
        ]),
        layers: vec![1, 3],
        kind: "route".into(),
    });

    let path_report = board
        .route_path_candidate_two_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("two-via path should succeed");
    let explain_report = board
        .route_path_candidate_two_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("two-via explain should succeed");

    assert_eq!(
        path_report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        explain_report.explanation_kind,
        RoutePathCandidateTwoViaExplainKind::DeterministicPathFound
    );
    assert_eq!(
        path_report.path.as_ref().map(|path| path.via_a_uuid),
        Some(second_via_a_uuid)
    );
    assert_ne!(
        path_report.path.as_ref().map(|path| path.via_a_uuid),
        Some(first_via_a_uuid)
    );
    assert_eq!(
        path_report.path.as_ref().map(|path| path.via_b_uuid),
        Some(via_b_uuid)
    );
    assert_eq!(
        explain_report
            .selected_pair
            .as_ref()
            .map(|entry| entry.via_a_uuid),
        path_report.path.as_ref().map(|path| path.via_a_uuid)
    );
    assert_eq!(
        explain_report
            .selected_pair
            .as_ref()
            .map(|entry| entry.via_b_uuid),
        path_report.path.as_ref().map(|path| path.via_b_uuid)
    );
    assert_eq!(
        explain_report
            .selected_pair
            .as_ref()
            .map(|entry| entry.middle_segment.points.clone()),
        path_report
            .path
            .as_ref()
            .map(|path| path.segments[1].points.clone())
    );
}
