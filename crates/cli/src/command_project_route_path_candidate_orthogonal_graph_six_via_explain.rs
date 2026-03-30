use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateOrthogonalGraphSixViaExplainKind,
    RoutePathCandidateOrthogonalGraphSixViaExplainReport, RoutePathCandidateStatus,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_graph_six_via_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalGraphSixViaExplainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_orthogonal_graph_six_via_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_graph_six_via_explain_text(
    report: &RoutePathCandidateOrthogonalGraphSixViaExplainReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!("status: {}", render_status(report.status.clone())),
        format!("explanation_kind: {}", render_kind(&report.explanation_kind)),
        format!(
            "matching_via_sextuples: {}",
            report.summary.matching_via_sextuple_count
        ),
    ];
    if let Some(selected) = &report.selected_sextuple {
        lines.push(format!("via_a_uuid: {}", selected.via_a_uuid));
        lines.push(format!("via_b_uuid: {}", selected.via_b_uuid));
        lines.push(format!("via_c_uuid: {}", selected.via_c_uuid));
        lines.push(format!("via_d_uuid: {}", selected.via_d_uuid));
        lines.push(format!("via_e_uuid: {}", selected.via_e_uuid));
        lines.push(format!("via_f_uuid: {}", selected.via_f_uuid));
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

fn render_kind(kind: &RoutePathCandidateOrthogonalGraphSixViaExplainKind) -> &'static str {
    match kind {
        RoutePathCandidateOrthogonalGraphSixViaExplainKind::DeterministicPathFound => {
            "deterministic_path_found"
        }
        RoutePathCandidateOrthogonalGraphSixViaExplainKind::NoMatchingAuthoredViaSextuple => {
            "no_matching_authored_via_sextuple"
        }
        RoutePathCandidateOrthogonalGraphSixViaExplainKind::AllMatchingViaSextuplesBlocked => {
            "all_matching_via_sextuples_blocked"
        }
    }
}
