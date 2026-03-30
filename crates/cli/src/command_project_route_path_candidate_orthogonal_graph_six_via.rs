use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateOrthogonalGraphSixViaReport, RoutePathCandidateStatus,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_graph_six_via(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalGraphSixViaReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_orthogonal_graph_six_via(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_graph_six_via_text(
    report: &RoutePathCandidateOrthogonalGraphSixViaReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!("status: {}", render_status(report.status.clone())),
        format!(
            "matching_via_sextuples: {}",
            report.summary.matching_via_sextuple_count
        ),
        format!(
            "available_via_sextuples: {}",
            report.summary.available_via_sextuple_count
        ),
    ];
    if let Some(path) = &report.path {
        lines.push(format!("via_a_uuid: {}", path.via_a_uuid));
        lines.push(format!("via_b_uuid: {}", path.via_b_uuid));
        lines.push(format!("via_c_uuid: {}", path.via_c_uuid));
        lines.push(format!("via_d_uuid: {}", path.via_d_uuid));
        lines.push(format!("via_e_uuid: {}", path.via_e_uuid));
        lines.push(format!("via_f_uuid: {}", path.via_f_uuid));
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
