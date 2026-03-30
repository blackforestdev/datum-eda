use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateOrthogonalGraphTwoViaExplainKind,
    RoutePathCandidateOrthogonalGraphTwoViaExplainReport, RoutePathCandidateStatus,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_graph_two_via_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalGraphTwoViaExplainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_orthogonal_graph_two_via_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_graph_two_via_explain_text(
    report: &RoutePathCandidateOrthogonalGraphTwoViaExplainReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!("status: {}", render_status(report.status.clone())),
        format!(
            "explanation_kind: {}",
            render_kind(&report.explanation_kind)
        ),
        format!(
            "matching_via_pairs: {}",
            report.summary.matching_via_pair_count
        ),
    ];
    if let Some(selected) = &report.selected_pair {
        lines.push(format!("via_a_uuid: {}", selected.via_a_uuid));
        lines.push(format!("via_b_uuid: {}", selected.via_b_uuid));
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

fn render_kind(kind: &RoutePathCandidateOrthogonalGraphTwoViaExplainKind) -> &'static str {
    match kind {
        RoutePathCandidateOrthogonalGraphTwoViaExplainKind::DeterministicPathFound => {
            "deterministic_path_found"
        }
        RoutePathCandidateOrthogonalGraphTwoViaExplainKind::NoMatchingAuthoredViaPair => {
            "no_matching_authored_via_pair"
        }
        RoutePathCandidateOrthogonalGraphTwoViaExplainKind::AllMatchingViaPairsBlocked => {
            "all_matching_via_pairs_blocked"
        }
    }
}
