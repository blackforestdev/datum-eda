use crate::board::{RoutePathCandidateOrthogonalGraphTwoViaExplainKind, RoutePathCandidateStatus};
use uuid::Uuid;

use super::route_path_candidate_orthogonal_graph_two_via::orthogonal_graph_two_via_board;

#[test]
fn route_path_candidate_orthogonal_graph_two_via_explain_reports_selected_pair_reasoning() {
    let (board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_a_uuid, via_b_uuid) =
        orthogonal_graph_two_via_board();
    let candidate = board
        .route_path_candidate_orthogonal_graph_two_via(
            net_uuid,
            anchor_top_uuid,
            anchor_bottom_uuid,
        )
        .expect("orthogonal graph two-via candidate should succeed");

    let report = board
        .route_path_candidate_orthogonal_graph_two_via_explain(
            net_uuid,
            anchor_top_uuid,
            anchor_bottom_uuid,
        )
        .expect("orthogonal graph two-via explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphTwoViaExplainKind::DeterministicPathFound
    );
    assert_eq!(
        report.selected_pair.as_ref().map(|pair| pair.via_a_uuid),
        Some(via_a_uuid)
    );
    assert_eq!(
        report.selected_pair.as_ref().map(|pair| pair.via_b_uuid),
        Some(via_b_uuid)
    );
    assert_eq!(
        report
            .selected_pair
            .as_ref()
            .map(|pair| pair.source_segment.points.clone()),
        candidate
            .path
            .as_ref()
            .map(|path| path.segments[0].points.clone())
    );
    assert_eq!(
        report
            .selected_pair
            .as_ref()
            .map(|pair| pair.middle_segment.points.clone()),
        candidate
            .path
            .as_ref()
            .map(|path| path.segments[1].points.clone())
    );
    assert_eq!(
        report
            .selected_pair
            .as_ref()
            .map(|pair| pair.target_segment.points.clone()),
        candidate
            .path
            .as_ref()
            .map(|path| path.segments[2].points.clone())
    );
    assert_eq!(
        report.summary.matching_via_pair_count,
        candidate.summary.matching_via_pair_count
    );
    assert_eq!(
        report.summary.blocked_via_pair_count,
        candidate.summary.blocked_via_pair_count
    );
    assert_eq!(
        report.summary.available_via_pair_count,
        candidate.summary.available_via_pair_count
    );
}

#[test]
fn route_path_candidate_orthogonal_graph_two_via_explain_reports_blocked_pair() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, _, _) =
        orthogonal_graph_two_via_board();
    board.tracks.insert(
        Uuid::from_u128(0x9513),
        crate::board::Track {
            uuid: Uuid::from_u128(0x9513),
            net: Uuid::from_u128(0x9501),
            from: crate::ir::geometry::Point::new(0, 300_000),
            to: crate::ir::geometry::Point::new(1_000_000, 300_000),
            width: 150_000,
            layer: 3,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_graph_two_via_explain(
            net_uuid,
            anchor_top_uuid,
            anchor_bottom_uuid,
        )
        .expect("orthogonal graph two-via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphTwoViaExplainKind::AllMatchingViaPairsBlocked
    );
    assert!(report.selected_pair.is_none());
    assert_eq!(report.blocked_matching_pairs.len(), 1);
}
