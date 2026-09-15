use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationExportReportView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) project_uuid: String,
    pub(crate) actions: usize,
    pub(crate) reviews: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationArtifactInspectionView {
    pub(crate) artifact_path: String,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) project_uuid: String,
    pub(crate) project_name: String,
    pub(crate) actions: usize,
    pub(crate) reviews: usize,
    pub(crate) add_component_actions: usize,
    pub(crate) remove_component_actions: usize,
    pub(crate) update_component_actions: usize,
    pub(crate) deferred_reviews: usize,
    pub(crate) rejected_reviews: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationArtifactValidationView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) actions: usize,
    pub(crate) reviews: usize,
    pub(crate) matches_expected: bool,
    pub(crate) canonical_bytes_match: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationArtifactComparisonActionView {
    pub(crate) action_id: String,
    pub(crate) proposal_action: String,
    pub(crate) reference: String,
    pub(crate) reason: String,
    pub(crate) status: String,
    pub(crate) review_decision: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationArtifactComparisonView {
    pub(crate) artifact_path: String,
    pub(crate) project_root: String,
    pub(crate) kind: String,
    pub(crate) artifact_version: u32,
    pub(crate) current_project_uuid: String,
    pub(crate) artifact_project_uuid: String,
    pub(crate) artifact_actions: usize,
    pub(crate) applicable_actions: usize,
    pub(crate) drifted_actions: usize,
    pub(crate) stale_actions: usize,
    pub(crate) actions: Vec<NativeProjectForwardAnnotationArtifactComparisonActionView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationArtifactFilterView {
    pub(crate) action: String,
    pub(crate) input_artifact_path: String,
    pub(crate) output_artifact_path: String,
    pub(crate) project_root: String,
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) artifact_actions: usize,
    pub(crate) applicable_actions: usize,
    pub(crate) filtered_reviews: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationArtifactApplyPlanActionView {
    pub(crate) action_id: String,
    pub(crate) proposal_action: String,
    pub(crate) reference: String,
    pub(crate) reason: String,
    pub(crate) applicability: String,
    pub(crate) execution: String,
    pub(crate) review_decision: Option<String>,
    pub(crate) required_inputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationArtifactApplyPlanView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) project_root: String,
    pub(crate) kind: String,
    pub(crate) artifact_version: u32,
    pub(crate) artifact_actions: usize,
    pub(crate) self_sufficient_actions: usize,
    pub(crate) requires_input_actions: usize,
    pub(crate) not_applicable_actions: usize,
    pub(crate) actions: Vec<NativeProjectForwardAnnotationArtifactApplyPlanActionView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationArtifactApplyView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) project_root: String,
    pub(crate) artifact_actions: usize,
    pub(crate) applied_actions: usize,
    pub(crate) skipped_deferred_actions: usize,
    pub(crate) skipped_rejected_actions: usize,
    pub(crate) applied: Vec<NativeProjectForwardAnnotationApplyReportView>,
    pub(crate) skipped: Vec<NativeProjectForwardAnnotationBatchApplySkippedActionView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationArtifactReviewImportView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) project_root: String,
    pub(crate) imported_reviews: usize,
    pub(crate) skipped_missing_live_actions: usize,
    pub(crate) total_artifact_reviews: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationArtifactReviewReplaceView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) project_root: String,
    pub(crate) replaced_reviews: usize,
    pub(crate) removed_existing_reviews: usize,
    pub(crate) skipped_missing_live_actions: usize,
    pub(crate) total_artifact_reviews: usize,
}
