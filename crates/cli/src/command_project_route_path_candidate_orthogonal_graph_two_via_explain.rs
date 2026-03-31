use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateOrthogonalGraphTwoViaExplainKind,
    RoutePathCandidateOrthogonalGraphTwoViaExplainReport,
};
use uuid::Uuid;

use super::command_project_route_path_candidate_orthogonal_graph_spine::{
    render_route_path_candidate_status, with_native_project_board,
};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_graph_two_via_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalGraphTwoViaExplainReport> {
    with_native_project_board(root, |board| {
        board
            .route_path_candidate_orthogonal_graph_two_via_explain(
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
            .map_err(|err| anyhow!(err))
    })
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_graph_two_via_explain_text(
    report: &RoutePathCandidateOrthogonalGraphTwoViaExplainReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "status: {}",
            render_route_path_candidate_status(report.status.clone())
        ),
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
