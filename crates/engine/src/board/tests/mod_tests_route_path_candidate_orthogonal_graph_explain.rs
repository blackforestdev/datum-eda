use crate::board::{RoutePathCandidateOrthogonalGraphExplainKind, RoutePathCandidateStatus};

use super::route_path_candidate_orthogonal_graph::orthogonal_graph_board;

#[test]
fn route_path_candidate_orthogonal_graph_explain_reports_selected_path_reasoning() {
    let (board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_graph_board();

    let report = board
        .route_path_candidate_orthogonal_graph_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal graph explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphExplainKind::DeterministicPathFound
    );
    assert_eq!(
        report.selected_path.as_ref().map(|path| path.points.len()),
        Some(7)
    );
    assert_eq!(
        report
            .selected_path
            .as_ref()
            .map(|path| path.cost.bend_count),
        Some(5)
    );
    assert_eq!(report.segment_evidence.len(), 1);
    assert_eq!(report.segment_evidence[0].layer_segment_index, 0);
    assert_eq!(report.segment_evidence[0].layer_segment_count, 1);
    assert_eq!(report.segment_evidence[0].bend_count, 5);
    assert_eq!(report.segment_evidence[0].point_count, 7);
    assert_eq!(report.segment_evidence[0].track_action_count, 6);
    assert!(report.blocked_edges.len() > 0);
}

#[test]
fn route_path_candidate_orthogonal_graph_explain_reports_blocked_graph() {
    let (mut board, net_uuid, _, anchor_a_uuid, anchor_b_uuid) = orthogonal_graph_board();
    board.tracks.insert(
        uuid::Uuid::from_u128(0x930c),
        crate::board::Track {
            uuid: uuid::Uuid::from_u128(0x930c),
            net: uuid::Uuid::from_u128(0x9301),
            from: crate::ir::geometry::Point::new(0, 600_000),
            to: crate::ir::geometry::Point::new(1_000_000, 600_000),
            width: 150_000,
            layer: 1,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_graph_explain(net_uuid, anchor_a_uuid, anchor_b_uuid)
        .expect("orthogonal graph explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphExplainKind::AllGraphPathsBlocked
    );
    assert!(report.selected_path.is_none());
    assert!(report.segment_evidence.is_empty());
    assert!(report.blocked_edges.len() > 0);
}
