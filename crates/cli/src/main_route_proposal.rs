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
    pub(crate) selection_profile: String,
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
    pub(crate) selection_profile: String,
    pub(crate) status: String,
    pub(crate) selection_rule: String,
    pub(crate) selected_candidate: Option<String>,
    pub(crate) selected_policy: Option<String>,
    pub(crate) selected_contract: Option<String>,
    pub(crate) explanation: String,
    pub(crate) candidates: Vec<NativeProjectRouteProposalSelectionCandidateView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectRouteStrategyReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) net_uuid: String,
    pub(crate) from_anchor_pad_uuid: String,
    pub(crate) to_anchor_pad_uuid: String,
    pub(crate) objective: String,
    pub(crate) recommended_profile: String,
    pub(crate) recommendation_rule: String,
    pub(crate) explanation: String,
    pub(crate) selector_status: String,
    pub(crate) selector_rule: String,
    pub(crate) selected_candidate: Option<String>,
    pub(crate) selected_policy: Option<String>,
    pub(crate) selected_contract: Option<String>,
    pub(crate) selected_actions: Option<usize>,
    pub(crate) next_step_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectRouteStrategyComparisonEntryView {
    pub(crate) objective: String,
    pub(crate) profile: String,
    pub(crate) proposal_available: bool,
    pub(crate) selector_status: String,
    pub(crate) selected_candidate: Option<String>,
    pub(crate) selected_policy: Option<String>,
    pub(crate) selected_contract: Option<String>,
    pub(crate) selected_actions: Option<usize>,
    pub(crate) distinction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectRouteStrategyCompareView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) net_uuid: String,
    pub(crate) from_anchor_pad_uuid: String,
    pub(crate) to_anchor_pad_uuid: String,
    pub(crate) comparison_rule: String,
    pub(crate) recommended_objective: String,
    pub(crate) recommended_profile: String,
    pub(crate) recommendation_reason: String,
    pub(crate) next_step_command: String,
    pub(crate) entries: Vec<NativeProjectRouteStrategyComparisonEntryView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectRouteStrategyDeltaProfileView {
    pub(crate) objective: String,
    pub(crate) profile: String,
    pub(crate) proposal_available: bool,
    pub(crate) selected_candidate: Option<String>,
    pub(crate) selected_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectRouteStrategyDeltaView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) net_uuid: String,
    pub(crate) from_anchor_pad_uuid: String,
    pub(crate) to_anchor_pad_uuid: String,
    pub(crate) compared_objectives: Vec<String>,
    pub(crate) compared_profiles: Vec<String>,
    pub(crate) outcomes_match: bool,
    pub(crate) outcome_relation: String,
    pub(crate) delta_classification: String,
    pub(crate) recommendation_summary: String,
    pub(crate) material_difference: String,
    pub(crate) recommended_objective: String,
    pub(crate) recommended_profile: String,
    pub(crate) profiles: Vec<NativeProjectRouteStrategyDeltaProfileView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectRouteStrategyBatchRequestIdentityView {
    pub(crate) request_id: String,
    pub(crate) fixture_id: String,
    pub(crate) project_root: String,
    pub(crate) net_uuid: String,
    pub(crate) from_anchor_pad_uuid: String,
    pub(crate) to_anchor_pad_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectRouteStrategyBatchEntryView {
    pub(crate) identity: NativeProjectRouteStrategyBatchRequestIdentityView,
    pub(crate) route_strategy_report: NativeProjectRouteStrategyReportView,
    pub(crate) route_strategy_compare: NativeProjectRouteStrategyCompareView,
    pub(crate) route_strategy_delta: NativeProjectRouteStrategyDeltaView,
    pub(crate) recommended_profile: String,
    pub(crate) delta_classification: String,
    pub(crate) outcomes_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectRouteStrategyBatchSummaryView {
    pub(crate) total_evaluated_requests: usize,
    pub(crate) recommendation_counts_by_profile: BTreeMap<String, usize>,
    pub(crate) delta_classification_counts: BTreeMap<String, usize>,
    pub(crate) same_outcome_count: usize,
    pub(crate) different_outcome_count: usize,
    pub(crate) proposal_available_count: usize,
    pub(crate) no_proposal_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectRouteStrategyBatchEvaluateView {
    pub(crate) action: String,
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) requests_manifest_path: String,
    pub(crate) requests_manifest_kind: String,
    pub(crate) requests_manifest_version: u32,
    pub(crate) summary: NativeProjectRouteStrategyBatchSummaryView,
    pub(crate) results: Vec<NativeProjectRouteStrategyBatchEntryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteStrategyBatchResultMalformedEntryView {
    pub(crate) result_index: usize,
    pub(crate) request_id: Option<String>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteStrategyBatchResultInspectionView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) requests_manifest_kind: String,
    pub(crate) requests_manifest_version: u32,
    pub(crate) summary: NativeProjectRouteStrategyBatchSummaryView,
    pub(crate) results: Vec<NativeProjectRouteStrategyBatchEntryView>,
    pub(crate) malformed_entries: Vec<NativeProjectRouteStrategyBatchResultMalformedEntryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteStrategyBatchResultValidationView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) kind: Option<String>,
    pub(crate) source_version: Option<u32>,
    pub(crate) version: Option<u32>,
    pub(crate) structurally_valid: bool,
    pub(crate) version_compatible: bool,
    pub(crate) missing_required_fields: Vec<String>,
    pub(crate) request_result_count_matches_summary: bool,
    pub(crate) recommendation_counts_match_summary: bool,
    pub(crate) delta_classification_counts_match_summary: bool,
    pub(crate) outcome_counts_match_summary: bool,
    pub(crate) proposal_counts_match_summary: bool,
    pub(crate) malformed_entries: Vec<NativeProjectRouteStrategyBatchResultMalformedEntryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSelectedRouteProposalExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) selection_profile: String,
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
    pub(crate) selection_profile: String,
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
        format!("selection_profile: {}", report.selection_profile),
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
        format!("selection_profile: {}", report.selection_profile),
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

pub(super) fn render_native_route_strategy_report_text(
    report: &NativeProjectRouteStrategyReportView,
) -> String {
    vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("net_uuid: {}", report.net_uuid),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!("objective: {}", report.objective),
        format!("recommended_profile: {}", report.recommended_profile),
        format!("recommendation_rule: {}", report.recommendation_rule),
        format!("explanation: {}", report.explanation),
        format!("selector_status: {}", report.selector_status),
        format!("selector_rule: {}", report.selector_rule),
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
        format!("next_step_command: {}", report.next_step_command),
    ]
    .join("\n")
}

pub(super) fn render_native_route_strategy_compare_text(
    report: &NativeProjectRouteStrategyCompareView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("net_uuid: {}", report.net_uuid),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!("comparison_rule: {}", report.comparison_rule),
        format!("recommended_objective: {}", report.recommended_objective),
        format!("recommended_profile: {}", report.recommended_profile),
        format!("recommendation_reason: {}", report.recommendation_reason),
        format!("next_step_command: {}", report.next_step_command),
        format!("entries: {}", report.entries.len()),
    ];
    for entry in &report.entries {
        lines.push(String::new());
        lines.push(format!("objective: {}", entry.objective));
        lines.push(format!("profile: {}", entry.profile));
        lines.push(format!("proposal_available: {}", entry.proposal_available));
        lines.push(format!("selector_status: {}", entry.selector_status));
        lines.push(format!(
            "selected_candidate: {}",
            entry
                .selected_candidate
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "selected_policy: {}",
            entry
                .selected_policy
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "selected_contract: {}",
            entry
                .selected_contract
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "selected_actions: {}",
            entry
                .selected_actions
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!("distinction: {}", entry.distinction));
    }
    lines.join("\n")
}

pub(super) fn render_native_route_strategy_delta_text(
    report: &NativeProjectRouteStrategyDeltaView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("net_uuid: {}", report.net_uuid),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!(
            "compared_objectives: {}",
            report.compared_objectives.join(", ")
        ),
        format!("compared_profiles: {}", report.compared_profiles.join(", ")),
        format!("outcomes_match: {}", report.outcomes_match),
        format!("outcome_relation: {}", report.outcome_relation),
        format!("delta_classification: {}", report.delta_classification),
        format!("recommendation_summary: {}", report.recommendation_summary),
        format!("material_difference: {}", report.material_difference),
        format!("recommended_objective: {}", report.recommended_objective),
        format!("recommended_profile: {}", report.recommended_profile),
        format!("profiles: {}", report.profiles.len()),
    ];
    for profile in &report.profiles {
        lines.push(String::new());
        lines.push(format!("objective: {}", profile.objective));
        lines.push(format!("profile: {}", profile.profile));
        lines.push(format!(
            "proposal_available: {}",
            profile.proposal_available
        ));
        lines.push(format!(
            "selected_candidate: {}",
            profile
                .selected_candidate
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "selected_policy: {}",
            profile
                .selected_policy
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
    }
    lines.join("\n")
}

pub(super) fn render_native_route_strategy_batch_evaluate_text(
    report: &NativeProjectRouteStrategyBatchEvaluateView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("kind: {}", report.kind),
        format!("version: {}", report.version),
        format!("requests_manifest_path: {}", report.requests_manifest_path),
        format!("requests_manifest_kind: {}", report.requests_manifest_kind),
        format!(
            "requests_manifest_version: {}",
            report.requests_manifest_version
        ),
        format!(
            "total_evaluated_requests: {}",
            report.summary.total_evaluated_requests
        ),
        format!("same_outcome_count: {}", report.summary.same_outcome_count),
        format!(
            "different_outcome_count: {}",
            report.summary.different_outcome_count
        ),
        format!(
            "proposal_available_count: {}",
            report.summary.proposal_available_count
        ),
        format!("no_proposal_count: {}", report.summary.no_proposal_count),
        format!("results: {}", report.results.len()),
    ];
    for (profile, count) in &report.summary.recommendation_counts_by_profile {
        lines.push(format!("recommendation_count[{profile}]: {count}"));
    }
    for (classification, count) in &report.summary.delta_classification_counts {
        lines.push(format!(
            "delta_classification_count[{classification}]: {count}"
        ));
    }
    for entry in &report.results {
        lines.push(String::new());
        lines.push(format!("request_id: {}", entry.identity.request_id));
        lines.push(format!("fixture_id: {}", entry.identity.fixture_id));
        lines.push(format!("project_root: {}", entry.identity.project_root));
        lines.push(format!("net_uuid: {}", entry.identity.net_uuid));
        lines.push(format!(
            "from_anchor_pad_uuid: {}",
            entry.identity.from_anchor_pad_uuid
        ));
        lines.push(format!(
            "to_anchor_pad_uuid: {}",
            entry.identity.to_anchor_pad_uuid
        ));
        lines.push(format!(
            "recommended_profile: {}",
            entry.recommended_profile
        ));
        lines.push(format!(
            "delta_classification: {}",
            entry.delta_classification
        ));
        lines.push(format!("outcomes_match: {}", entry.outcomes_match));
    }
    lines.join("\n")
}

pub(super) fn render_native_route_strategy_batch_result_inspection_text(
    report: &NativeProjectRouteStrategyBatchResultInspectionView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("source_version: {}", report.source_version),
        format!("version: {}", report.version),
        format!("requests_manifest_kind: {}", report.requests_manifest_kind),
        format!(
            "requests_manifest_version: {}",
            report.requests_manifest_version
        ),
        format!(
            "total_evaluated_requests: {}",
            report.summary.total_evaluated_requests
        ),
        format!("same_outcome_count: {}", report.summary.same_outcome_count),
        format!(
            "different_outcome_count: {}",
            report.summary.different_outcome_count
        ),
        format!(
            "proposal_available_count: {}",
            report.summary.proposal_available_count
        ),
        format!("no_proposal_count: {}", report.summary.no_proposal_count),
        format!("results: {}", report.results.len()),
        format!("malformed_entries: {}", report.malformed_entries.len()),
    ];
    for (profile, count) in &report.summary.recommendation_counts_by_profile {
        lines.push(format!("recommendation_count[{profile}]: {count}"));
    }
    for (classification, count) in &report.summary.delta_classification_counts {
        lines.push(format!(
            "delta_classification_count[{classification}]: {count}"
        ));
    }
    for entry in &report.results {
        lines.push(String::new());
        lines.push(format!("request_id: {}", entry.identity.request_id));
        lines.push(format!("fixture_id: {}", entry.identity.fixture_id));
        lines.push(format!("project_root: {}", entry.identity.project_root));
        lines.push(format!("net_uuid: {}", entry.identity.net_uuid));
        lines.push(format!(
            "from_anchor_pad_uuid: {}",
            entry.identity.from_anchor_pad_uuid
        ));
        lines.push(format!(
            "to_anchor_pad_uuid: {}",
            entry.identity.to_anchor_pad_uuid
        ));
        lines.push(format!(
            "recommended_profile: {}",
            entry.recommended_profile
        ));
        lines.push(format!(
            "delta_classification: {}",
            entry.delta_classification
        ));
        lines.push(format!("outcomes_match: {}", entry.outcomes_match));
    }
    for malformed in &report.malformed_entries {
        lines.push(String::new());
        lines.push(format!(
            "malformed_result_index: {}",
            malformed.result_index
        ));
        lines.push(format!(
            "malformed_request_id: {}",
            malformed
                .request_id
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
        for issue in &malformed.issues {
            lines.push(format!("malformed_issue: {issue}"));
        }
    }
    lines.join("\n")
}

pub(super) fn render_native_route_strategy_batch_result_validation_text(
    report: &NativeProjectRouteStrategyBatchResultValidationView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!(
            "kind: {}",
            report.kind.clone().unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "source_version: {}",
            report
                .source_version
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "version: {}",
            report
                .version
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!("structurally_valid: {}", report.structurally_valid),
        format!("version_compatible: {}", report.version_compatible),
        format!(
            "request_result_count_matches_summary: {}",
            report.request_result_count_matches_summary
        ),
        format!(
            "recommendation_counts_match_summary: {}",
            report.recommendation_counts_match_summary
        ),
        format!(
            "delta_classification_counts_match_summary: {}",
            report.delta_classification_counts_match_summary
        ),
        format!(
            "outcome_counts_match_summary: {}",
            report.outcome_counts_match_summary
        ),
        format!(
            "proposal_counts_match_summary: {}",
            report.proposal_counts_match_summary
        ),
        format!(
            "missing_required_fields: {}",
            report.missing_required_fields.len()
        ),
        format!("malformed_entries: {}", report.malformed_entries.len()),
    ];
    for field in &report.missing_required_fields {
        lines.push(format!("missing_required_field: {field}"));
    }
    for malformed in &report.malformed_entries {
        lines.push(String::new());
        lines.push(format!(
            "malformed_result_index: {}",
            malformed.result_index
        ));
        lines.push(format!(
            "malformed_request_id: {}",
            malformed
                .request_id
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ));
        for issue in &malformed.issues {
            lines.push(format!("malformed_issue: {issue}"));
        }
    }
    lines.join("\n")
}

pub(super) fn render_native_selected_route_proposal_export_text(
    report: &NativeProjectSelectedRouteProposalExportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("selection_profile: {}", report.selection_profile),
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
        format!("selection_profile: {}", report.selection_profile),
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
