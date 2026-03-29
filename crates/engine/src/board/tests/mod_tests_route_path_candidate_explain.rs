use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x110);
    let other_net_uuid = Uuid::from_u128(0x111);
    let class_uuid = Uuid::from_u128(0x112);
    let anchor_a_uuid = Uuid::from_u128(0x130);
    let anchor_b_uuid = Uuid::from_u128(0x131);
    let anchor_c_uuid = Uuid::from_u128(0x132);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate-explain".into(),
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
                        package: Uuid::from_u128(0x201),
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
                        package: Uuid::from_u128(0x202),
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
                        package: Uuid::from_u128(0x203),
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
    )
}

#[test]
fn route_path_candidate_explain_reports_selected_span_and_rule_for_found_path() {
    let (board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, _) = demo_board();

    let report = board
        .route_path_candidate_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateExplainKind::DeterministicPathFound
    );
    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.matching_span_count, 2);
    assert_eq!(
        report.selected_span.as_ref().map(|span| span.layer),
        Some(1)
    );
    assert_eq!(
        report
            .selected_span
            .as_ref()
            .map(|span| (span.from, span.to)),
        Some((Point::new(100_000, 100_000), Point::new(500_000, 500_000)))
    );
}

#[test]
fn route_path_candidate_explain_preserves_selected_span_orientation_for_reversed_anchor_order() {
    let (board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, _) = demo_board();

    let report = board
        .route_path_candidate_explain(net_uuid, anchor_b_uuid, anchor_a_uuid)
        .expect("explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateExplainKind::DeterministicPathFound
    );
    assert_eq!(
        report.selected_span.as_ref().map(|span| span.layer),
        Some(1)
    );
    assert_eq!(
        report
            .selected_span
            .as_ref()
            .map(|span| (span.from, span.to)),
        Some((Point::new(500_000, 500_000), Point::new(100_000, 100_000)))
    );
}

#[test]
fn route_path_candidate_explain_reports_no_matching_corridor_span() {
    let (board, net_uuid, _, anchor_a_uuid, _, anchor_c_uuid) = demo_board();

    let report = board
        .route_path_candidate_explain(net_uuid, anchor_a_uuid, anchor_c_uuid)
        .expect("explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateExplainKind::NoMatchingCorridorSpan
    );
    assert!(report.selected_span.is_none());
    assert!(report.blocked_matching_spans.is_empty());
}

#[test]
fn route_path_candidate_explain_reports_blocked_matching_spans_and_reasons() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, _) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0x500),
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
        .route_path_candidate_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateExplainKind::AllMatchingSpansBlocked
    );
    assert_eq!(report.summary.blocked_span_count, 2);
    assert_eq!(report.blocked_matching_spans.len(), 2);
    assert!(
        report
            .blocked_matching_spans
            .iter()
            .all(|span| !span.blockages.is_empty())
    );
}

#[test]
fn route_path_candidate_explain_classifies_all_matching_spans_blocked_even_with_irrelevant_available_spans()
 {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, anchor_c_uuid) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0x501),
        polygon: Polygon::new(vec![
            Point::new(260_000, 200_000),
            Point::new(340_000, 200_000),
            Point::new(340_000, 420_000),
            Point::new(260_000, 420_000),
        ]),
        layers: vec![1, 3],
        kind: "route".into(),
    });

    let blocked_report = board
        .route_path_candidate_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("blocked explain should succeed");
    let irrelevant_pair_report = board
        .route_path_candidate_explain(net_uuid, anchor_b_uuid, anchor_c_uuid)
        .expect("irrelevant explain should succeed");

    assert_eq!(
        blocked_report.explanation_kind,
        RoutePathCandidateExplainKind::AllMatchingSpansBlocked
    );
    assert_eq!(blocked_report.summary.matching_span_count, 2);
    assert_eq!(blocked_report.summary.blocked_span_count, 2);
    assert_eq!(blocked_report.blocked_matching_spans.len(), 2);
    assert_eq!(
        irrelevant_pair_report.explanation_kind,
        RoutePathCandidateExplainKind::DeterministicPathFound
    );
}

#[test]
fn route_path_candidate_explain_selected_span_matches_fallback_path_candidate_span() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, _) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0x502),
        polygon: Polygon::new(vec![
            Point::new(260_000, 200_000),
            Point::new(340_000, 200_000),
            Point::new(340_000, 420_000),
            Point::new(260_000, 420_000),
        ]),
        layers: vec![1],
        kind: "route".into(),
    });

    let path_report = board
        .route_path_candidate(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("path candidate should succeed");
    let explain_report = board
        .route_path_candidate_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("explain should succeed");

    assert_eq!(
        path_report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        explain_report.explanation_kind,
        RoutePathCandidateExplainKind::DeterministicPathFound
    );
    assert_eq!(path_report.path.as_ref().map(|path| path.layer), Some(3));
    assert_eq!(
        explain_report.selected_span.as_ref().map(|span| span.layer),
        Some(3)
    );
    assert_eq!(
        explain_report
            .selected_span
            .as_ref()
            .map(|span| (span.from, span.to)),
        path_report
            .path
            .as_ref()
            .map(|path| (path.points[0], path.points[1]))
    );
}
