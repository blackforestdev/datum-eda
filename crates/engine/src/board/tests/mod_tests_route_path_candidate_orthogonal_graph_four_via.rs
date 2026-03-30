use crate::board::{RoutePathCandidateStatus, Track};
use crate::ir::geometry::Point;
use uuid::Uuid;

use super::route_path_candidate_four_via::demo_board;

#[test]
fn route_path_candidate_orthogonal_graph_four_via_reports_deterministic_path() {
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
    ) = demo_board();

    let report = board
        .route_path_candidate_orthogonal_graph_four_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("orthogonal graph four-via path candidate should succeed");

    assert_eq!(report.status, RoutePathCandidateStatus::DeterministicPathFound);
    assert_eq!(report.summary.candidate_via_count, 4);
    assert_eq!(report.summary.matching_via_quadruple_count, 1);
    assert_eq!(report.summary.available_via_quadruple_count, 1);
    assert_eq!(report.path.as_ref().map(|path| path.via_a_uuid), Some(via_a_uuid));
    assert_eq!(report.path.as_ref().map(|path| path.via_b_uuid), Some(via_b_uuid));
    assert_eq!(report.path.as_ref().map(|path| path.via_c_uuid), Some(via_c_uuid));
    assert_eq!(report.path.as_ref().map(|path| path.via_d_uuid), Some(via_d_uuid));
    assert_eq!(report.path.as_ref().map(|path| path.segments.len()), Some(5));
}

#[test]
fn route_path_candidate_orthogonal_graph_four_via_reports_no_path_when_middle_layer_is_cut() {
    let (mut board, net_uuid, other_net_uuid, anchor_top_uuid, anchor_bottom_uuid, _, _, _, _) =
        demo_board();
    board.tracks.insert(
        Uuid::from_u128(0xf7b),
        Track {
            uuid: Uuid::from_u128(0xf7b),
            net: other_net_uuid,
            from: Point::new(0, 600_000),
            to: Point::new(1_000_000, 600_000),
            width: 150_000,
            layer: 7,
        },
    );

    let report = board
        .route_path_candidate_orthogonal_graph_four_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("orthogonal graph four-via path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.matching_via_quadruple_count, 1);
    assert_eq!(report.summary.blocked_via_quadruple_count, 1);
    assert_eq!(report.summary.available_via_quadruple_count, 0);
    assert!(report.path.is_none());
}
