//! Complete existing route-action review and producing basis.
use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalReviewView {
    pub(crate) action: String,
    pub(crate) review_source: String,
    pub(crate) status: String,
    pub(crate) explanation: String,
    pub(crate) project_root: Option<String>,
    pub(crate) artifact_path: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) source_version: Option<u32>,
    pub(crate) version: Option<u32>,
    pub(crate) project_uuid: Option<String>,
    pub(crate) producer_revision: Option<String>,
    pub(crate) source_fingerprint: Option<String>,
    pub(crate) producer_proposal: Option<eda_engine::substrate::Proposal>,
    pub(crate) project_name: Option<String>,
    pub(crate) net_uuid: Option<String>,
    pub(crate) from_anchor_pad_uuid: Option<String>,
    pub(crate) to_anchor_pad_uuid: Option<String>,
    pub(crate) selection_profile: Option<String>,
    pub(crate) selection_rule: Option<String>,
    pub(crate) selected_candidate: Option<String>,
    pub(crate) selected_policy: Option<String>,
    pub(crate) contract: String,
    pub(crate) actions: usize,
    pub(crate) draw_track_actions: usize,
    pub(crate) selected_path_bend_count: usize,
    pub(crate) selected_path_point_count: usize,
    pub(crate) selected_path_segment_count: usize,
    pub(crate) segment_evidence:
        Option<Vec<NativeProjectRouteProposalArtifactInspectionSegmentView>>,
    pub(crate) proposal_actions: Vec<NativeProjectRouteProposalActionView>,
}
