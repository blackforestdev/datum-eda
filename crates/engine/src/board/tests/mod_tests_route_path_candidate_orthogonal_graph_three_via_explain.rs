use crate::board::{
    RoutePathCandidateOrthogonalGraphThreeViaExplainKind, RoutePathCandidateStatus, Track,
};
use crate::ir::geometry::Point;
use uuid::Uuid;

use super::route_path_candidate_three_via::demo_board;

#[test]
fn route_path_candidate_orthogonal_graph_three_via_explain_reports_selected_triple() {
    let (board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_a_uuid, via_b_uuid, via_c_uuid) =
        demo_board();

    let report = board
        .route_path_candidate_orthogonal_graph_three_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("orthogonal graph three-via explain should succeed");

    assert_eq!(report.status, RoutePathCandidateStatus::DeterministicPathFound);
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphThreeViaExplainKind::DeterministicPathFound
    );
    assert_eq!(report.selected_triple.as_ref().map(|path| path.via_a_uuid), Some(via_a_uuid));
    assert_eq!(report.selected_triple.as_ref().map(|path| path.via_b_uuid), Some(via_b_uuid));
    assert_eq!(report.selected_triple.as_ref().map(|path| path.via_c_uuid), Some(via_c_uuid));
}

#[test]
fn route_path_candidate_orthogonal_graph_three_via_explain_reports_blocked_triple() {
    let (mut board, net_uuid, other_net_uuid, anchor_top_uuid, anchor_bottom_uuid, _, _, _) =
        demo_board();
    board.tracks.insert(
        Uuid::from_u128(0xf1c),
        Track {
            uuid: Uuid::from_u128(0xf1c),
            net: other_net_uuid,
            from: Point::new(0, 500_000),
            to: Point::new(1_000_000, 500_000),
            width: 150_000,
            layer: 5,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_graph_three_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("orthogonal graph three-via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphThreeViaExplainKind::AllMatchingViaTriplesBlocked
    );
    assert!(report.selected_triple.is_none());
    assert_eq!(report.blocked_matching_triples.len(), 1);
}
