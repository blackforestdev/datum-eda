use crate::board::{RoutePathCandidateOrthogonalGraphViaExplainKind, RoutePathCandidateStatus};
use uuid::Uuid;

use super::route_path_candidate_orthogonal_graph_via::orthogonal_graph_via_board;

#[test]
fn route_path_candidate_orthogonal_graph_via_explain_reports_selected_via_reasoning() {
    let (board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_uuid) =
        orthogonal_graph_via_board();
    let candidate = board
        .route_path_candidate_orthogonal_graph_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("orthogonal graph via candidate should succeed");

    let report = board
        .route_path_candidate_orthogonal_graph_via_explain(
            net_uuid,
            anchor_top_uuid,
            anchor_bottom_uuid,
        )
        .expect("orthogonal graph via explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphViaExplainKind::DeterministicPathFound
    );
    assert_eq!(
        report
            .selected_via
            .as_ref()
            .map(|selected| selected.via_uuid),
        Some(via_uuid)
    );
    assert_eq!(
        report
            .selected_via
            .as_ref()
            .map(|selected| selected.source_segment.points.clone()),
        candidate
            .path
            .as_ref()
            .map(|path| path.segments[0].points.clone())
    );
    assert_eq!(
        report
            .selected_via
            .as_ref()
            .map(|selected| selected.target_segment.points.clone()),
        candidate
            .path
            .as_ref()
            .map(|path| path.segments[1].points.clone())
    );
    assert_eq!(
        report.summary.candidate_via_count,
        candidate.summary.candidate_via_count
    );
    assert_eq!(
        report.summary.matching_via_count,
        candidate.summary.matching_via_count
    );
    assert_eq!(
        report.summary.blocked_via_count,
        candidate.summary.blocked_via_count
    );
    assert_eq!(
        report.summary.available_via_count,
        candidate.summary.available_via_count
    );
}

#[test]
fn route_path_candidate_orthogonal_graph_via_explain_reports_no_matching_via() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_uuid) =
        orthogonal_graph_via_board();
    board.vias.remove(&via_uuid);

    let report = board
        .route_path_candidate_orthogonal_graph_via_explain(
            net_uuid,
            anchor_top_uuid,
            anchor_bottom_uuid,
        )
        .expect("orthogonal graph via explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphViaExplainKind::NoMatchingAuthoredVia
    );
}

#[test]
fn route_path_candidate_orthogonal_graph_via_explain_reports_blocked_matching_via() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, _) =
        orthogonal_graph_via_board();
    board.tracks.insert(
        Uuid::from_u128(0x940e),
        crate::board::Track {
            uuid: Uuid::from_u128(0x940e),
            net: Uuid::from_u128(0x9401),
            from: crate::ir::geometry::Point::new(0, 600_000),
            to: crate::ir::geometry::Point::new(1_000_000, 600_000),
            width: 150_000,
            layer: 3,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_graph_via_explain(
            net_uuid,
            anchor_top_uuid,
            anchor_bottom_uuid,
        )
        .expect("orthogonal graph via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphViaExplainKind::AllMatchingViasBlocked
    );
    assert!(report.selected_via.is_none());
    assert_eq!(report.blocked_matching_vias.len(), 1);
}
