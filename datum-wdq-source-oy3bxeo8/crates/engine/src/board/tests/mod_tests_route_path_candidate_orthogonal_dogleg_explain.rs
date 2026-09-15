use crate::board::{RoutePathCandidateOrthogonalDoglegExplainKind, RoutePathCandidateStatus};
use crate::ir::geometry::Point;

use super::route_path_candidate_orthogonal_dogleg::orthogonal_dogleg_board;

#[test]
fn route_path_candidate_orthogonal_dogleg_explain_reports_selected_corner_reasoning() {
    let (board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_dogleg_board();

    let report = board
        .route_path_candidate_orthogonal_dogleg_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal dogleg explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalDoglegExplainKind::DeterministicPathFound
    );
    assert_eq!(
        report.selected_dogleg.as_ref().map(|entry| entry.corner),
        Some(Point::new(100_000, 900_000))
    );
    assert_eq!(report.blocked_doglegs.len(), 1);
}

#[test]
fn route_path_candidate_orthogonal_dogleg_explain_reports_no_same_layer_candidate() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_dogleg_board();
    board
        .pads
        .get_mut(&anchor_b_uuid)
        .expect("anchor exists")
        .layer = 3;

    let report = board
        .route_path_candidate_orthogonal_dogleg_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal dogleg explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalDoglegExplainKind::NoSameLayerDoglegCandidate
    );
    assert_eq!(report.summary.candidate_corner_count, 0);
    assert!(report.selected_dogleg.is_none());
    assert!(report.blocked_doglegs.is_empty());
}
