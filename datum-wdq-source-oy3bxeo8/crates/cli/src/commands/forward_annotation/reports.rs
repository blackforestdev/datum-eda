use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationProposalView {
    pub(crate) domain: &'static str,
    pub(crate) total_actions: usize,
    pub(crate) add_component_actions: usize,
    pub(crate) remove_component_actions: usize,
    pub(crate) update_component_actions: usize,
    pub(crate) add_component_group: Vec<NativeProjectForwardAnnotationProposalActionView>,
    pub(crate) remove_component_group: Vec<NativeProjectForwardAnnotationProposalActionView>,
    pub(crate) update_component_group: Vec<NativeProjectForwardAnnotationProposalActionView>,
    pub(crate) actions: Vec<NativeProjectForwardAnnotationProposalActionView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectForwardAnnotationProposalActionView {
    pub(crate) action_id: String,
    pub(crate) action: String,
    pub(crate) reference: String,
    pub(crate) symbol_uuid: Option<String>,
    pub(crate) component_uuid: Option<String>,
    pub(crate) reason: String,
    pub(crate) schematic_value: Option<String>,
    pub(crate) board_value: Option<String>,
    pub(crate) schematic_part_uuid: Option<String>,
    pub(crate) board_part_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationReviewView {
    pub(crate) domain: &'static str,
    pub(crate) total_reviews: usize,
    pub(crate) deferred_actions: usize,
    pub(crate) rejected_actions: usize,
    pub(crate) actions: Vec<NativeProjectForwardAnnotationReviewActionView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectForwardAnnotationReviewActionView {
    pub(crate) action_id: String,
    pub(crate) decision: String,
    pub(crate) proposal_action: String,
    pub(crate) reference: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationReviewReportView {
    pub(crate) action: String,
    pub(crate) action_id: String,
    pub(crate) decision: String,
    pub(crate) proposal_action: String,
    pub(crate) reference: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationApplyReportView {
    pub(crate) action: String,
    pub(crate) action_id: String,
    pub(crate) proposal_action: String,
    pub(crate) reason: String,
    pub(crate) component_report: NativeProjectBoardComponentMutationReportView,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationBatchApplySkippedActionView {
    pub(crate) action_id: String,
    pub(crate) proposal_action: String,
    pub(crate) reference: String,
    pub(crate) reason: String,
    pub(crate) skip_reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationBatchApplyReportView {
    pub(crate) action: String,
    pub(crate) domain: &'static str,
    pub(crate) proposal_actions: usize,
    pub(crate) applied_actions: usize,
    pub(crate) skipped_deferred_actions: usize,
    pub(crate) skipped_rejected_actions: usize,
    pub(crate) skipped_requires_input_actions: usize,
    pub(crate) applied: Vec<NativeProjectForwardAnnotationApplyReportView>,
    pub(crate) skipped: Vec<NativeProjectForwardAnnotationBatchApplySkippedActionView>,
}
