use crate::board::*;
use crate::ir::geometry::{Point, Polygon};
use uuid::Uuid;

use super::route_path_candidate_authored_copper_plus_one_gap::plus_one_gap_board;

#[test]
fn route_path_candidate_authored_copper_plus_one_gap_explain_reports_selected_path() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, track_a_uuid, track_b_uuid) =
        plus_one_gap_board();
    let report = board
        .route_path_candidate_authored_copper_plus_one_gap_explain(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("query should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert!(matches!(
        report.explanation_kind,
        RoutePathCandidateAuthoredCopperPlusOneGapExplainKind::DeterministicPathFound
    ));
    let path = report.selected_path.expect("selected path should exist");
    assert!(path.selection_reason.contains("first exact-one-gap path found"));
    assert_eq!(path.path.steps.len(), 3);
    assert_eq!(
        path.path
            .steps
            .iter()
            .map(|step| step.object_uuid)
            .collect::<Vec<_>>(),
        vec![Some(track_a_uuid), None, Some(track_b_uuid)]
    );
}

#[test]
fn route_path_candidate_authored_copper_plus_one_gap_explain_reports_no_exact_one_gap_path() {
    let (mut board, net_uuid, from_pad_uuid, to_pad_uuid, _, _) = plus_one_gap_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0xa130),
        polygon: Polygon::new(vec![
            Point::new(750_000, 400_000),
            Point::new(1_250_000, 400_000),
            Point::new(1_250_000, 600_000),
            Point::new(750_000, 600_000),
        ]),
        layers: vec![1],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate_authored_copper_plus_one_gap_explain(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("query should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert!(matches!(
        report.explanation_kind,
        RoutePathCandidateAuthoredCopperPlusOneGapExplainKind::NoExactOneGapPath
    ));
    assert!(report.selected_path.is_none());
}
