use crate::board::{
    RoutePathCandidateOrthogonalGraphFiveViaExplainKind, RoutePathCandidateStatus, Track,
};
use crate::ir::geometry::Point;
use uuid::Uuid;

use super::route_path_candidate_five_via::demo_board;

#[test]
fn route_path_candidate_orthogonal_graph_five_via_explain_reports_selected_quintuple() {
    let (
        board,
        net_uuid,
        _,
        anchor_top_uuid,
        anchor_bottom_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
        via_e_uuid,
    ) = demo_board();

    let report = board
        .route_path_candidate_orthogonal_graph_five_via_explain(
            net_uuid,
            anchor_top_uuid,
            anchor_bottom_uuid,
        )
        .expect("orthogonal graph five-via explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphFiveViaExplainKind::DeterministicPathFound
    );
    assert_eq!(
        report
            .selected_quintuple
            .as_ref()
            .map(|path| path.via_a_uuid),
        Some(via_a_uuid)
    );
    assert_eq!(
        report
            .selected_quintuple
            .as_ref()
            .map(|path| path.via_b_uuid),
        Some(via_b_uuid)
    );
    assert_eq!(
        report
            .selected_quintuple
            .as_ref()
            .map(|path| path.via_c_uuid),
        Some(via_c_uuid)
    );
    assert_eq!(
        report
            .selected_quintuple
            .as_ref()
            .map(|path| path.via_d_uuid),
        Some(via_d_uuid)
    );
    assert_eq!(
        report
            .selected_quintuple
            .as_ref()
            .map(|path| path.via_e_uuid),
        Some(via_e_uuid)
    );
}

#[test]
fn route_path_candidate_orthogonal_graph_five_via_explain_reports_blocked_quintuple() {
    let (mut board, net_uuid, other_net_uuid, anchor_top_uuid, anchor_bottom_uuid, _, _, _, _, _) =
        demo_board();
    board.tracks.insert(
        Uuid::from_u128(0xfdd),
        Track {
            uuid: Uuid::from_u128(0xfdd),
            net: other_net_uuid,
            from: Point::new(0, 680_000),
            to: Point::new(1_000_000, 680_000),
            width: 150_000,
            layer: 9,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_graph_five_via_explain(
            net_uuid,
            anchor_top_uuid,
            anchor_bottom_uuid,
        )
        .expect("orthogonal graph five-via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphFiveViaExplainKind::AllMatchingViaQuintuplesBlocked
    );
    assert!(report.selected_quintuple.is_none());
    assert_eq!(report.blocked_matching_quintuples.len(), 1);
}
