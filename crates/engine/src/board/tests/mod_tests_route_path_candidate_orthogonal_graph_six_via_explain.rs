use crate::board::{
    RoutePathCandidateOrthogonalGraphSixViaExplainKind, RoutePathCandidateStatus, Track,
};
use crate::ir::geometry::Point;
use uuid::Uuid;

use super::route_path_candidate_six_via::demo_board;

#[test]
fn route_path_candidate_orthogonal_graph_six_via_explain_reports_selected_sextuple() {
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
        via_f_uuid,
    ) = demo_board();

    let report = board
        .route_path_candidate_orthogonal_graph_six_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("orthogonal graph six-via explain should succeed");

    assert_eq!(report.status, RoutePathCandidateStatus::DeterministicPathFound);
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphSixViaExplainKind::DeterministicPathFound
    );
    assert_eq!(report.selected_sextuple.as_ref().map(|path| path.via_a_uuid), Some(via_a_uuid));
    assert_eq!(report.selected_sextuple.as_ref().map(|path| path.via_b_uuid), Some(via_b_uuid));
    assert_eq!(report.selected_sextuple.as_ref().map(|path| path.via_c_uuid), Some(via_c_uuid));
    assert_eq!(report.selected_sextuple.as_ref().map(|path| path.via_d_uuid), Some(via_d_uuid));
    assert_eq!(report.selected_sextuple.as_ref().map(|path| path.via_e_uuid), Some(via_e_uuid));
    assert_eq!(report.selected_sextuple.as_ref().map(|path| path.via_f_uuid), Some(via_f_uuid));
}

#[test]
fn route_path_candidate_orthogonal_graph_six_via_explain_reports_blocked_sextuple() {
    let (
        mut board,
        net_uuid,
        other_net_uuid,
        anchor_top_uuid,
        anchor_bottom_uuid,
        _,
        _,
        _,
        _,
        _,
        _,
    ) = demo_board();
    board.tracks.insert(
        Uuid::from_u128(0x103e),
        Track {
            uuid: Uuid::from_u128(0x103e),
            net: other_net_uuid,
            from: Point::new(0, 700_000),
            to: Point::new(1_000_000, 700_000),
            width: 150_000,
            layer: 11,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_graph_six_via_explain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("orthogonal graph six-via explain should succeed");

    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateOrthogonalGraphSixViaExplainKind::AllMatchingViaSextuplesBlocked
    );
    assert!(report.selected_sextuple.is_none());
    assert_eq!(report.blocked_matching_sextuples.len(), 1);
}
