use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateOrthogonalGraphViaExplainKind,
    RoutePathCandidateOrthogonalGraphViaExplainReport, RoutePathCandidateStatus,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_graph_via_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalGraphViaExplainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_orthogonal_graph_via_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_graph_via_explain_text(
    report: &RoutePathCandidateOrthogonalGraphViaExplainReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("status: {}", render_status(report.status.clone())),
        format!(
            "explanation_kind: {}",
            render_kind(&report.explanation_kind)
        ),
        format!("net_uuid: {}", report.net_uuid),
        format!("net_name: {}", report.net_name),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!("selection_rule: {}", report.selection_rule),
        format!(
            "candidate_copper_layers: {}",
            report.summary.candidate_copper_layer_count
        ),
        format!("candidate_vias: {}", report.summary.candidate_via_count),
        format!("matching_vias: {}", report.summary.matching_via_count),
        format!("available_vias: {}", report.summary.available_via_count),
        format!("blocked_vias: {}", report.summary.blocked_via_count),
    ];
    if let Some(selected) = &report.selected_via {
        lines.push(format!("selected_via_uuid: {}", selected.via_uuid));
        lines.push(format!("selection_reason: {}", selected.selection_reason));
    } else {
        lines.push("selected_via_uuid: none".to_string());
    }
    lines.push(format!(
        "blocked_matching_via_count: {}",
        report.blocked_matching_vias.len()
    ));
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

fn render_kind(kind: &RoutePathCandidateOrthogonalGraphViaExplainKind) -> &'static str {
    match kind {
        RoutePathCandidateOrthogonalGraphViaExplainKind::DeterministicPathFound => {
            "deterministic_path_found"
        }
        RoutePathCandidateOrthogonalGraphViaExplainKind::NoMatchingAuthoredVia => {
            "no_matching_authored_via"
        }
        RoutePathCandidateOrthogonalGraphViaExplainKind::AllMatchingViasBlocked => {
            "all_matching_vias_blocked"
        }
    }
}
