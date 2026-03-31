use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::RoutePathCandidateOrthogonalGraphPathCost;
use super::route_path_candidate_orthogonal_graph_via_spine::build_orthogonal_graph_via_candidate_spine;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub cost: RoutePathCandidateOrthogonalGraphPathCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaPath {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub segments: Vec<RoutePathCandidateOrthogonalGraphViaSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_count: usize,
    pub blocked_via_count: usize,
    pub available_via_count: usize,
    pub path_segment_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub component_selection_rules: Vec<String>,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphViaSummary,
    pub path: Option<RoutePathCandidateOrthogonalGraphViaPath>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_via(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphViaReport, RoutePathCandidateError> {
        let spine = build_orthogonal_graph_via_candidate_spine(
            self,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;

        Ok(RoutePathCandidateOrthogonalGraphViaReport {
            contract: "m5_route_path_candidate_orthogonal_graph_via_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: spine.selection_rule,
            component_selection_rules: spine.component_selection_rules,
            status: spine.status,
            net_uuid: spine.net_uuid,
            net_name: spine.net_name,
            from_anchor_pad_uuid: spine.from_anchor_pad_uuid,
            to_anchor_pad_uuid: spine.to_anchor_pad_uuid,
            candidate_copper_layers: spine.candidate_copper_layers,
            summary: spine.summary,
            path: spine.path,
        })
    }
}
