use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};

use super::route_path_candidate_orthogonal_graph_spine::build_orthogonal_graph_candidate_spine;
pub use super::route_path_candidate_orthogonal_graph_spine::{
    RoutePathCandidateOrthogonalGraphPath, RoutePathCandidateOrthogonalGraphSegmentEvidence,
    RoutePathCandidateOrthogonalGraphSummary,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphSummary,
    pub path: Option<RoutePathCandidateOrthogonalGraphPath>,
    pub segment_evidence: Vec<RoutePathCandidateOrthogonalGraphSegmentEvidence>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphReport, RoutePathCandidateError> {
        let spine = build_orthogonal_graph_candidate_spine(
            self,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;

        Ok(RoutePathCandidateOrthogonalGraphReport {
            contract: "m5_route_path_candidate_orthogonal_graph_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: spine.selection_rule,
            status: spine.status,
            net_uuid: spine.net_uuid,
            net_name: spine.net_name,
            from_anchor_pad_uuid: spine.from_anchor_pad_uuid,
            to_anchor_pad_uuid: spine.to_anchor_pad_uuid,
            candidate_copper_layers: spine.candidate_copper_layers,
            summary: spine.summary,
            path: spine.path,
            segment_evidence: spine.segment_evidence,
        })
    }
}
