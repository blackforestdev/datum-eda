use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::RoutePathCandidateOrthogonalGraphTwoViaReport;
use uuid::Uuid;

use super::command_project_route_path_candidate_orthogonal_graph_spine::{
    render_route_path_candidate_status, with_native_project_board,
};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_graph_two_via(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalGraphTwoViaReport> {
    with_native_project_board(root, |board| {
        board
            .route_path_candidate_orthogonal_graph_two_via(
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
            .map_err(|err| anyhow!(err))
    })
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_graph_two_via_text(
    report: &RoutePathCandidateOrthogonalGraphTwoViaReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "status: {}",
            render_route_path_candidate_status(report.status.clone())
        ),
        format!(
            "matching_via_pairs: {}",
            report.summary.matching_via_pair_count
        ),
        format!(
            "available_via_pairs: {}",
            report.summary.available_via_pair_count
        ),
    ];
    if let Some(path) = &report.path {
        lines.push(format!("via_a_uuid: {}", path.via_a_uuid));
        lines.push(format!("via_b_uuid: {}", path.via_b_uuid));
        lines.push(format!("path_segments: {}", path.segments.len()));
    }
    lines.join("\n")
}
