use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateOrthogonalGraphThreeViaReport, RoutePathCandidateStatus,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_graph_three_via(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalGraphThreeViaReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_orthogonal_graph_three_via(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_graph_three_via_text(
    report: &RoutePathCandidateOrthogonalGraphThreeViaReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!("status: {}", render_status(report.status.clone())),
        format!(
            "matching_via_triples: {}",
            report.summary.matching_via_triple_count
        ),
        format!(
            "available_via_triples: {}",
            report.summary.available_via_triple_count
        ),
    ];
    if let Some(path) = &report.path {
        lines.push(format!("via_a_uuid: {}", path.via_a_uuid));
        lines.push(format!("via_b_uuid: {}", path.via_b_uuid));
        lines.push(format!("via_c_uuid: {}", path.via_c_uuid));
        lines.push(format!("path_segments: {}", path.segments.len()));
    }
    lines.join("\n")
}

fn render_status(status: RoutePathCandidateStatus) -> &'static str {
    match status {
        RoutePathCandidateStatus::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}
