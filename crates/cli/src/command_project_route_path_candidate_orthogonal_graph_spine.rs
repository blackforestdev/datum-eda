use std::path::Path;

use anyhow::Result;
use eda_engine::board::{Board, RoutePathCandidateStatus};

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn with_native_project_board<T, F>(root: &Path, f: F) -> Result<T>
where
    F: FnOnce(&Board) -> Result<T>,
{
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    f(&board)
}

pub(crate) fn render_route_path_candidate_status(status: RoutePathCandidateStatus) -> &'static str {
    match status {
        RoutePathCandidateStatus::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}

pub(crate) fn push_orthogonal_graph_segment_evidence_lines<I>(
    lines: &mut Vec<String>,
    segment_evidence: I,
) where
    I: IntoIterator<Item = (usize, usize, String, usize, usize, usize)>,
{
    let segment_evidence = segment_evidence.into_iter().collect::<Vec<_>>();
    lines.push(format!("segment_evidence: {}", segment_evidence.len()));
    for (
        layer_segment_index,
        layer_segment_count,
        layer,
        bend_count,
        point_count,
        track_action_count,
    ) in segment_evidence
    {
        lines.push(String::new());
        lines.push(format!("layer_segment_index: {}", layer_segment_index));
        lines.push(format!("layer_segment_count: {}", layer_segment_count));
        lines.push(format!("layer: {}", layer));
        lines.push(format!("bend_count: {}", bend_count));
        lines.push(format!("point_count: {}", point_count));
        lines.push(format!("track_action_count: {}", track_action_count));
    }
}
