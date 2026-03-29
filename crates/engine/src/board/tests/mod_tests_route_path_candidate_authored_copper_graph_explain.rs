use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x3300);
    let other_net_uuid = Uuid::from_u128(0x3301);
    let class_uuid = Uuid::from_u128(0x3302);
    let anchor_a_uuid = Uuid::from_u128(0x3303);
    let anchor_b_uuid = Uuid::from_u128(0x3304);
    let track_uuid = Uuid::from_u128(0x3305);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate-authored-copper-graph-explain".into(),
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
                        package: Uuid::from_u128(0x3310),
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
                        package: Uuid::from_u128(0x3311),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 100_000),
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
                    from: Point::new(100_000, 100_000),
                    to: Point::new(900_000, 100_000),
                    width: 150_000,
                    layer: 1,
                },
            )]),
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
        track_uuid,
    )
}

#[test]
fn route_path_candidate_authored_copper_graph_explain_reports_selected_path_reasoning() {
    let (board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, track_uuid) = demo_board();

    let report = board
        .route_path_candidate_authored_copper_graph_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("authored copper graph explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateAuthoredCopperGraphExplainKind::DeterministicPathFound
    );
    assert_eq!(report.summary.candidate_track_count, 1);
    assert_eq!(report.summary.candidate_via_count, 0);
    assert_eq!(
        report.selected_path.as_ref().map(|path| path.steps.len()),
        Some(1)
    );
    assert_eq!(
        report
            .selected_path
            .as_ref()
            .map(|path| path.steps[0].object_uuid),
        Some(track_uuid)
    );
}

#[test]
fn route_path_candidate_authored_copper_graph_explain_reports_no_existing_path() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid, track_uuid) = demo_board();
    board.tracks.remove(&track_uuid);

    let report = board
        .route_path_candidate_authored_copper_graph_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("authored copper graph explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateAuthoredCopperGraphExplainKind::NoExistingAuthoredCopperPath
    );
    assert!(report.selected_path.is_none());
}
