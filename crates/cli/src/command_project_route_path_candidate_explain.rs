use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{RoutePathCandidateExplainKind, RoutePathCandidateExplainReport};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateExplainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_explain(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_explain_text(
    report: &RoutePathCandidateExplainReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("status: {}", render_status(report)),
        format!("explanation_kind: {}", render_kind(report)),
        format!("net_uuid: {}", report.net_uuid),
        format!("net_name: {}", report.net_name),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!("selection_rule: {}", report.selection_rule),
        format!(
            "candidate_copper_layers: {}",
            report.summary.candidate_copper_layer_count
        ),
        format!("matching_spans: {}", report.summary.matching_span_count),
        format!("available_spans: {}", report.summary.available_span_count),
        format!("blocked_spans: {}", report.summary.blocked_span_count),
    ];

    if let Some(span) = &report.selected_span {
        lines.push(format!("selected_span_layer: {}", span.layer));
        lines.push(format!("selected_span_pair_index: {}", span.pair_index));
        lines.push(format!("selection_reason: {}", span.selection_reason));
    } else {
        lines.push("selected_span_layer: none".to_string());
        lines.push("selected_span_pair_index: none".to_string());
    }

    lines.push(format!(
        "blocked_matching_span_count: {}",
        report.blocked_matching_spans.len()
    ));
    lines.join("\n")
}

fn render_status(report: &RoutePathCandidateExplainReport) -> &'static str {
    match report.status {
        eda_engine::board::RoutePathCandidateStatus::DeterministicPathFound => {
            "deterministic_path_found"
        }
        eda_engine::board::RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}

fn render_kind(report: &RoutePathCandidateExplainReport) -> &'static str {
    match report.explanation_kind {
        RoutePathCandidateExplainKind::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateExplainKind::NoMatchingCorridorSpan => "no_matching_corridor_span",
        RoutePathCandidateExplainKind::AllMatchingSpansBlocked => "all_matching_spans_blocked",
    }
}
