use crate::board::{RoutePathCandidateOrthogonalTwoBendExplainKind, RoutePathCandidateStatus};
use crate::ir::geometry::Point;

use super::route_path_candidate_orthogonal_two_bend::orthogonal_two_bend_board;

#[test]
fn route_path_candidate_orthogonal_two_bend_explain_reports_selected_path_reasoning() {
    let (board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_two_bend_board();

    let report = board
        .route_path_candidate_orthogonal_two_bend_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal two-bend explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalTwoBendExplainKind::DeterministicPathFound
    );
    assert_eq!(
        report
            .selected_path
            .as_ref()
            .map(|entry| entry.points.clone()),
        Some(vec![
            Point::new(100_000, 100_000),
            Point::new(0, 100_000),
            Point::new(0, 900_000),
            Point::new(900_000, 900_000),
        ])
    );
}

#[test]
fn route_path_candidate_orthogonal_two_bend_explain_reports_no_same_layer_candidate() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_two_bend_board();
    board
        .pads
        .get_mut(&anchor_b_uuid)
        .expect("anchor exists")
        .layer = 3;

    let report = board
        .route_path_candidate_orthogonal_two_bend_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal two-bend explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalTwoBendExplainKind::NoSameLayerTwoBendCandidate
    );
    assert_eq!(report.summary.candidate_path_count, 0);
}
