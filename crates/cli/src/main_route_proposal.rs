use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalSelectionCandidateView {
    pub(crate) candidate: String,
    pub(crate) policy: Option<String>,
    pub(crate) selected: bool,
    pub(crate) contract: Option<String>,
    pub(crate) actions: Option<usize>,
    pub(crate) selected_path_bend_count: Option<usize>,
    pub(crate) selected_path_point_count: Option<usize>,
    pub(crate) selected_path_segment_count: Option<usize>,
    pub(crate) message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalSelectionView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) net_uuid: String,
    pub(crate) from_anchor_pad_uuid: String,
    pub(crate) to_anchor_pad_uuid: String,
    pub(crate) status: String,
    pub(crate) selection_rule: String,
    pub(crate) attempted_candidates: usize,
    pub(crate) selected_candidate: Option<String>,
    pub(crate) selected_policy: Option<String>,
    pub(crate) selected_contract: Option<String>,
    pub(crate) selected_actions: Option<usize>,
    pub(crate) selected_path_bend_count: Option<usize>,
    pub(crate) selected_path_point_count: Option<usize>,
    pub(crate) selected_path_segment_count: Option<usize>,
    pub(crate) candidates: Vec<NativeProjectRouteProposalSelectionCandidateView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalExplainView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) net_uuid: String,
    pub(crate) from_anchor_pad_uuid: String,
    pub(crate) to_anchor_pad_uuid: String,
    pub(crate) status: String,
    pub(crate) selection_rule: String,
    pub(crate) selected_candidate: Option<String>,
    pub(crate) selected_policy: Option<String>,
    pub(crate) selected_contract: Option<String>,
    pub(crate) explanation: String,
    pub(crate) candidates: Vec<NativeProjectRouteProposalSelectionCandidateView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSelectedRouteProposalExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) selection_rule: String,
    pub(crate) selected_candidate: String,
    pub(crate) selected_policy: Option<String>,
    pub(crate) artifact_path: String,
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) project_uuid: String,
    pub(crate) contract: String,
    pub(crate) actions: usize,
    pub(crate) selected_path_bend_count: usize,
    pub(crate) selected_path_point_count: usize,
    pub(crate) selected_path_segment_count: usize,
    pub(crate) segment_evidence:
        Option<Vec<NativeProjectRouteProposalArtifactInspectionSegmentView>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteApplySelectedView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) selection_rule: String,
    pub(crate) selected_candidate: String,
    pub(crate) selected_policy: Option<String>,
    pub(crate) contract: String,
    pub(crate) proposal_actions: usize,
    pub(crate) applied_actions: usize,
    pub(crate) applied: Vec<NativeProjectBoardTrackMutationReportView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalExportReportView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) project_uuid: String,
    pub(crate) contract: String,
    pub(crate) actions: usize,
    pub(crate) selected_path_bend_count: usize,
    pub(crate) selected_path_point_count: usize,
    pub(crate) selected_path_segment_count: usize,
    pub(crate) segment_evidence:
        Option<Vec<NativeProjectRouteProposalArtifactInspectionSegmentView>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalArtifactInspectionSegmentView {
    pub(crate) layer_segment_index: usize,
    pub(crate) layer_segment_count: usize,
    pub(crate) layer: i32,
    pub(crate) bend_count: usize,
    pub(crate) point_count: usize,
    pub(crate) track_action_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalArtifactInspectionView {
    pub(crate) artifact_path: String,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) project_uuid: String,
    pub(crate) project_name: String,
    pub(crate) contract: String,
    pub(crate) actions: usize,
    pub(crate) draw_track_actions: usize,
    pub(crate) selected_path_bend_count: usize,
    pub(crate) selected_path_point_count: usize,
    pub(crate) selected_path_segment_count: usize,
    pub(crate) segment_evidence:
        Option<Vec<NativeProjectRouteProposalArtifactInspectionSegmentView>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalArtifactApplyView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) project_root: String,
    pub(crate) artifact_actions: usize,
    pub(crate) applied_actions: usize,
    pub(crate) selected_path_bend_count: usize,
    pub(crate) selected_path_point_count: usize,
    pub(crate) selected_path_segment_count: usize,
    pub(crate) applied: Vec<NativeProjectBoardTrackMutationReportView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalArtifactRevalidationSegmentView {
    pub(crate) layer_segment_index: usize,
    pub(crate) layer_segment_count: usize,
    pub(crate) artifact_layer: i32,
    pub(crate) artifact_bend_count: usize,
    pub(crate) artifact_point_count: usize,
    pub(crate) artifact_track_action_count: usize,
    pub(crate) live_layer: Option<i32>,
    pub(crate) live_bend_count: Option<usize>,
    pub(crate) live_point_count: Option<usize>,
    pub(crate) live_track_action_count: Option<usize>,
    pub(crate) matches_live: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalArtifactRevalidationView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) project_root: String,
    pub(crate) contract: String,
    pub(crate) artifact_actions: usize,
    pub(crate) live_actions: Option<usize>,
    pub(crate) matches_live: bool,
    pub(crate) drift_kind: Option<String>,
    pub(crate) drift_message: Option<String>,
    pub(crate) live_rebuild_error: Option<String>,
    pub(crate) selected_path_bend_count: usize,
    pub(crate) selected_path_point_count: usize,
    pub(crate) selected_path_segment_count: usize,
    pub(crate) live_selected_path_bend_count: Option<usize>,
    pub(crate) live_selected_path_point_count: Option<usize>,
    pub(crate) live_selected_path_segment_count: Option<usize>,
    pub(crate) segment_evidence:
        Option<Vec<NativeProjectRouteProposalArtifactRevalidationSegmentView>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteApplyView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) contract: String,
    pub(crate) proposal_actions: usize,
    pub(crate) applied_actions: usize,
    pub(crate) applied: Vec<NativeProjectBoardTrackMutationReportView>,
}

pub(super) fn render_native_route_proposal_export_text(
    report: &NativeProjectRouteProposalExportReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("version: {}", report.version),
        format!("project_uuid: {}", report.project_uuid),
        format!("contract: {}", report.contract),
        format!("actions: {}", report.actions),
        format!(
            "selected_path_bend_count: {}",
            report.selected_path_bend_count
        ),
        format!(
            "selected_path_point_count: {}",
            report.selected_path_point_count
        ),
        format!(
            "selected_path_segment_count: {}",
            report.selected_path_segment_count
        ),
        format!(
            "segment_evidence: {}",
            report
                .segment_evidence
                .as_ref()
                .map(|value| value.len().to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
    ];
    if let Some(segment_evidence) = &report.segment_evidence {
        for segment in segment_evidence {
            lines.push(String::new());
            lines.push(format!(
                "layer_segment_index: {}",
                segment.layer_segment_index
            ));
            lines.push(format!(
                "layer_segment_count: {}",
                segment.layer_segment_count
            ));
            lines.push(format!("layer: {}", segment.layer));
            lines.push(format!("bend_count: {}", segment.bend_count));
            lines.push(format!("point_count: {}", segment.point_count));
            lines.push(format!(
                "track_action_count: {}",
                segment.track_action_count
            ));
        }
    }
    lines.join("\n")
}

pub(super) fn render_native_route_proposal_selection_text(
    report: &NativeProjectRouteProposalSelectionView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("net_uuid: {}", report.net_uuid),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!("status: {}", report.status),
        format!("selection_rule: {}", report.selection_rule),
        format!("attempted_candidates: {}", report.attempted_candidates),
        format!(
            "selected_candidate: {}",
            report
                .selected_candidate
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "selected_policy: {}",
            report
                .selected_policy
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "selected_contract: {}",
            report
                .selected_contract
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "selected_actions: {}",
            report
                .selected_actions
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "selected_path_bend_count: {}",
            report
                .selected_path_bend_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "selected_path_point_count: {}",
            report
                .selected_path_point_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "selected_path_segment_count: {}",
            report
                .selected_path_segment_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!("candidates: {}", report.candidates.len()),
    ];
    for candidate in &report.candidates {
        lines.push(String::new());
        lines.push(format!("candidate: {}", candidate.candidate));
        lines.push(format!(
            "policy: {}",
            candidate
                .policy
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!("selected: {}", candidate.selected));
        lines.push(format!(
            "contract: {}",
            candidate
                .contract
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "actions: {}",
            candidate
                .actions
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "selected_path_bend_count: {}",
            candidate
                .selected_path_bend_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "selected_path_point_count: {}",
            candidate
                .selected_path_point_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "selected_path_segment_count: {}",
            candidate
                .selected_path_segment_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "message: {}",
            candidate
                .message
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
    }
    lines.join("\n")
}

pub(super) fn render_native_route_proposal_explain_text(
    report: &NativeProjectRouteProposalExplainView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("net_uuid: {}", report.net_uuid),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!("status: {}", report.status),
        format!("selection_rule: {}", report.selection_rule),
        format!(
            "selected_candidate: {}",
            report
                .selected_candidate
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "selected_policy: {}",
            report
                .selected_policy
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "selected_contract: {}",
            report
                .selected_contract
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!("explanation: {}", report.explanation),
        format!("candidates: {}", report.candidates.len()),
    ];
    for candidate in &report.candidates {
        lines.push(String::new());
        lines.push(format!("candidate: {}", candidate.candidate));
        lines.push(format!(
            "policy: {}",
            candidate
                .policy
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!("selected: {}", candidate.selected));
        lines.push(format!(
            "contract: {}",
            candidate
                .contract
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "message: {}",
            candidate
                .message
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
    }
    lines.join("\n")
}

pub(super) fn render_native_selected_route_proposal_export_text(
    report: &NativeProjectSelectedRouteProposalExportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("selection_rule: {}", report.selection_rule),
        format!("selected_candidate: {}", report.selected_candidate),
        format!(
            "selected_policy: {}",
            report
                .selected_policy
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("version: {}", report.version),
        format!("project_uuid: {}", report.project_uuid),
        format!("contract: {}", report.contract),
        format!("actions: {}", report.actions),
        format!(
            "selected_path_bend_count: {}",
            report.selected_path_bend_count
        ),
        format!(
            "selected_path_point_count: {}",
            report.selected_path_point_count
        ),
        format!(
            "selected_path_segment_count: {}",
            report.selected_path_segment_count
        ),
        format!(
            "segment_evidence: {}",
            report
                .segment_evidence
                .as_ref()
                .map(|value| value.len().to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
    ];
    if let Some(segment_evidence) = &report.segment_evidence {
        for segment in segment_evidence {
            lines.push(String::new());
            lines.push(format!(
                "layer_segment_index: {}",
                segment.layer_segment_index
            ));
            lines.push(format!(
                "layer_segment_count: {}",
                segment.layer_segment_count
            ));
            lines.push(format!("layer: {}", segment.layer));
            lines.push(format!("bend_count: {}", segment.bend_count));
            lines.push(format!("point_count: {}", segment.point_count));
            lines.push(format!(
                "track_action_count: {}",
                segment.track_action_count
            ));
        }
    }
    lines.join("\n")
}

pub(super) fn render_native_route_apply_selected_text(
    report: &NativeProjectRouteApplySelectedView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("selection_rule: {}", report.selection_rule),
        format!("selected_candidate: {}", report.selected_candidate),
        format!(
            "selected_policy: {}",
            report
                .selected_policy
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!("contract: {}", report.contract),
        format!("proposal_actions: {}", report.proposal_actions),
        format!("applied_actions: {}", report.applied_actions),
    ];
    for applied in &report.applied {
        lines.push(String::new());
        lines.push(format!("track_uuid: {}", applied.track_uuid));
        lines.push(format!("net_uuid: {}", applied.net_uuid));
        lines.push(format!("from_x_nm: {}", applied.from_x_nm));
        lines.push(format!("from_y_nm: {}", applied.from_y_nm));
        lines.push(format!("to_x_nm: {}", applied.to_x_nm));
        lines.push(format!("to_y_nm: {}", applied.to_y_nm));
        lines.push(format!("width_nm: {}", applied.width_nm));
        lines.push(format!("layer: {}", applied.layer));
    }
    lines.join("\n")
}

pub(super) fn render_native_route_proposal_artifact_inspection_text(
    report: &NativeProjectRouteProposalArtifactInspectionView,
) -> String {
    let mut lines = vec![
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("source_version: {}", report.source_version),
        format!("version: {}", report.version),
        format!("migration_applied: {}", report.migration_applied),
        format!("project_uuid: {}", report.project_uuid),
        format!("project_name: {}", report.project_name),
        format!("contract: {}", report.contract),
        format!("actions: {}", report.actions),
        format!("draw_track_actions: {}", report.draw_track_actions),
        format!(
            "selected_path_bend_count: {}",
            report.selected_path_bend_count
        ),
        format!(
            "selected_path_point_count: {}",
            report.selected_path_point_count
        ),
        format!(
            "selected_path_segment_count: {}",
            report.selected_path_segment_count
        ),
        format!(
            "segment_evidence: {}",
            report
                .segment_evidence
                .as_ref()
                .map(|value| value.len().to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
    ];
    if let Some(segment_evidence) = &report.segment_evidence {
        for segment in segment_evidence {
            lines.push(String::new());
            lines.push(format!(
                "layer_segment_index: {}",
                segment.layer_segment_index
            ));
            lines.push(format!(
                "layer_segment_count: {}",
                segment.layer_segment_count
            ));
            lines.push(format!("layer: {}", segment.layer));
            lines.push(format!("bend_count: {}", segment.bend_count));
            lines.push(format!("point_count: {}", segment.point_count));
            lines.push(format!(
                "track_action_count: {}",
                segment.track_action_count
            ));
        }
    }
    lines.join("\n")
}

pub(super) fn render_native_route_proposal_artifact_apply_text(
    report: &NativeProjectRouteProposalArtifactApplyView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("artifact_actions: {}", report.artifact_actions),
        format!("applied_actions: {}", report.applied_actions),
        format!(
            "selected_path_bend_count: {}",
            report.selected_path_bend_count
        ),
        format!(
            "selected_path_point_count: {}",
            report.selected_path_point_count
        ),
        format!(
            "selected_path_segment_count: {}",
            report.selected_path_segment_count
        ),
    ];
    for applied in &report.applied {
        lines.push(String::new());
        lines.push(format!("track_uuid: {}", applied.track_uuid));
        lines.push(format!("net_uuid: {}", applied.net_uuid));
        lines.push(format!("from_x_nm: {}", applied.from_x_nm));
        lines.push(format!("from_y_nm: {}", applied.from_y_nm));
        lines.push(format!("to_x_nm: {}", applied.to_x_nm));
        lines.push(format!("to_y_nm: {}", applied.to_y_nm));
        lines.push(format!("width_nm: {}", applied.width_nm));
        lines.push(format!("layer: {}", applied.layer));
    }
    lines.join("\n")
}

pub(super) fn render_native_route_proposal_artifact_revalidation_text(
    report: &NativeProjectRouteProposalArtifactRevalidationView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("contract: {}", report.contract),
        format!("artifact_actions: {}", report.artifact_actions),
        format!(
            "live_actions: {}",
            report
                .live_actions
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!("matches_live: {}", report.matches_live),
        format!(
            "drift_kind: {}",
            report
                .drift_kind
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "drift_message: {}",
            report
                .drift_message
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "live_rebuild_error: {}",
            report
                .live_rebuild_error
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "selected_path_bend_count: {}",
            report.selected_path_bend_count
        ),
        format!(
            "selected_path_point_count: {}",
            report.selected_path_point_count
        ),
        format!(
            "selected_path_segment_count: {}",
            report.selected_path_segment_count
        ),
        format!(
            "live_selected_path_bend_count: {}",
            report
                .live_selected_path_bend_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "live_selected_path_point_count: {}",
            report
                .live_selected_path_point_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "live_selected_path_segment_count: {}",
            report
                .live_selected_path_segment_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "segment_evidence: {}",
            report
                .segment_evidence
                .as_ref()
                .map(|value| value.len().to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
    ];
    if let Some(segment_evidence) = &report.segment_evidence {
        for segment in segment_evidence {
            lines.push(String::new());
            lines.push(format!(
                "layer_segment_index: {}",
                segment.layer_segment_index
            ));
            lines.push(format!(
                "layer_segment_count: {}",
                segment.layer_segment_count
            ));
            lines.push(format!("artifact_layer: {}", segment.artifact_layer));
            lines.push(format!(
                "artifact_bend_count: {}",
                segment.artifact_bend_count
            ));
            lines.push(format!(
                "artifact_point_count: {}",
                segment.artifact_point_count
            ));
            lines.push(format!(
                "artifact_track_action_count: {}",
                segment.artifact_track_action_count
            ));
            lines.push(format!(
                "live_layer: {}",
                segment
                    .live_layer
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ));
            lines.push(format!(
                "live_bend_count: {}",
                segment
                    .live_bend_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ));
            lines.push(format!(
                "live_point_count: {}",
                segment
                    .live_point_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ));
            lines.push(format!(
                "live_track_action_count: {}",
                segment
                    .live_track_action_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ));
            lines.push(format!("matches_live: {}", segment.matches_live));
        }
    }
    lines.join("\n")
}

pub(super) fn render_native_route_apply_text(report: &NativeProjectRouteApplyView) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("contract: {}", report.contract),
        format!("proposal_actions: {}", report.proposal_actions),
        format!("applied_actions: {}", report.applied_actions),
    ];
    for applied in &report.applied {
        lines.push(String::new());
        lines.push(format!("track_uuid: {}", applied.track_uuid));
        lines.push(format!("net_uuid: {}", applied.net_uuid));
        lines.push(format!("from_x_nm: {}", applied.from_x_nm));
        lines.push(format!("from_y_nm: {}", applied.from_y_nm));
        lines.push(format!("to_x_nm: {}", applied.to_x_nm));
        lines.push(format!("to_y_nm: {}", applied.to_y_nm));
        lines.push(format!("width_nm: {}", applied.width_nm));
        lines.push(format!("layer: {}", applied.layer));
    }
    lines.join("\n")
}
