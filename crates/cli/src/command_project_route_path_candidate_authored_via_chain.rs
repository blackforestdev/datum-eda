use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{RoutePathCandidateAuthoredViaChainReport, RoutePathCandidateStatus};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_authored_via_chain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateAuthoredViaChainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_authored_via_chain(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_authored_via_chain_text(
    report: &RoutePathCandidateAuthoredViaChainReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("selection_rule: {}", report.selection_rule),
        format!("status: {}", render_status(report)),
        format!("net_uuid: {}", report.net_uuid),
        format!("net_name: {}", report.net_name),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!(
            "candidate_copper_layers: {}",
            report.summary.candidate_copper_layer_count
        ),
        format!("candidate_vias: {}", report.summary.candidate_via_count),
        format!(
            "matching_via_chains: {}",
            report.summary.matching_via_chain_count
        ),
        format!(
            "available_via_chains: {}",
            report.summary.available_via_chain_count
        ),
        format!(
            "blocked_via_chains: {}",
            report.summary.blocked_via_chain_count
        ),
    ];

    if let Some(path) = &report.path {
        let via_chain = path
            .via_chain
            .iter()
            .map(|via| via.via_uuid.to_string())
            .collect::<Vec<_>>()
            .join(",");
        lines.push(format!("path_via_chain_uuids: {}", via_chain));
        lines.push(format!("path_segments: {}", path.segments.len()));
    } else {
        lines.push("path_via_chain_uuids: none".to_string());
        lines.push("path_segments: 0".to_string());
    }

    lines.join("\n")
}

fn render_status(report: &RoutePathCandidateAuthoredViaChainReport) -> &'static str {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}
