use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError,
    RoutePathCandidateOrthogonalGraphPathCost, RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_via::RoutePathCandidateOrthogonalGraphViaSummary;
use super::route_path_candidate_orthogonal_graph_via_spine::build_orthogonal_graph_via_candidate_spine;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalGraphViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredVia,
    AllMatchingViasBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub cost: RoutePathCandidateOrthogonalGraphPathCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaExplainBlockedVia {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub source_segment: Option<RoutePathCandidateOrthogonalGraphViaExplainSegment>,
    pub target_segment: Option<RoutePathCandidateOrthogonalGraphViaExplainSegment>,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaExplainSelectedVia {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub source_segment: RoutePathCandidateOrthogonalGraphViaExplainSegment,
    pub target_segment: RoutePathCandidateOrthogonalGraphViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_count: usize,
    pub blocked_via_count: usize,
    pub available_via_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateOrthogonalGraphViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub component_selection_rules: Vec<String>,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphViaExplainSummary,
    pub selected_via: Option<RoutePathCandidateOrthogonalGraphViaExplainSelectedVia>,
    pub blocked_matching_vias: Vec<RoutePathCandidateOrthogonalGraphViaExplainBlockedVia>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphViaExplainReport, RoutePathCandidateError> {
        let spine = build_orthogonal_graph_via_candidate_spine(
            self,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
        let selected_via = spine.via_searches.iter().find_map(|search| {
            Some(RoutePathCandidateOrthogonalGraphViaExplainSelectedVia {
                via_uuid: search.via.uuid,
                via_position: search.via.position,
                source_segment: RoutePathCandidateOrthogonalGraphViaExplainSegment {
                    layer: search.source.search.layer,
                    points: search.source.path.as_ref()?.points.clone(),
                    cost: search.source.path.as_ref()?.cost.clone(),
                },
                target_segment: RoutePathCandidateOrthogonalGraphViaExplainSegment {
                    layer: search.target.search.layer,
                    points: search.target.path.as_ref()?.points.clone(),
                    cost: search.target.path.as_ref()?.cost.clone(),
                },
                selection_reason: format!(
                    "selected because it is the first matching authored via whose source-layer and target-layer orthogonal graph searches both yield deterministic paths between layers {} and {}; each layer-side path is the lowest-cost graph path under the accepted ranking rule",
                    search.source.search.layer, search.target.search.layer
                ),
            })
        });

        let blocked_matching_vias = spine
            .via_searches
            .iter()
            .filter(|search| search.source.path.is_none() || search.target.path.is_none())
            .map(
                |search| RoutePathCandidateOrthogonalGraphViaExplainBlockedVia {
                    via_uuid: search.via.uuid,
                    via_position: search.via.position,
                    source_segment: search.source.path.as_ref().map(|path| {
                        RoutePathCandidateOrthogonalGraphViaExplainSegment {
                            layer: path.layer,
                            points: path.points.clone(),
                            cost: path.cost.clone(),
                        }
                    }),
                    target_segment: search.target.path.as_ref().map(|path| {
                        RoutePathCandidateOrthogonalGraphViaExplainSegment {
                            layer: path.layer,
                            points: path.points.clone(),
                            cost: path.cost.clone(),
                        }
                    }),
                    source_blockages: search
                        .source
                        .search
                        .blocked_edges
                        .iter()
                        .flat_map(|edge| edge.blockages.clone())
                        .collect(),
                    target_blockages: search
                        .target
                        .search
                        .blocked_edges
                        .iter()
                        .flat_map(|edge| edge.blockages.clone())
                        .collect(),
                },
            )
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateOrthogonalGraphViaExplainReport {
            contract: "m5_route_path_candidate_orthogonal_graph_via_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: spine.status.clone(),
            explanation_kind: explanation_kind(spine.status.clone(), &spine.summary),
            net_uuid: spine.net_uuid,
            net_name: spine.net_name,
            from_anchor_pad_uuid: spine.from_anchor_pad_uuid,
            to_anchor_pad_uuid: spine.to_anchor_pad_uuid,
            selection_rule: spine.selection_rule,
            component_selection_rules: spine.component_selection_rules,
            candidate_copper_layers: spine.candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalGraphViaExplainSummary {
                candidate_copper_layer_count: spine.summary.candidate_copper_layer_count,
                candidate_via_count: spine.summary.candidate_via_count,
                matching_via_count: spine.summary.matching_via_count,
                blocked_via_count: spine.summary.blocked_via_count,
                available_via_count: spine.summary.available_via_count,
            },
            selected_via,
            blocked_matching_vias,
        })
    }
}

fn explanation_kind(
    status: RoutePathCandidateStatus,
    summary: &RoutePathCandidateOrthogonalGraphViaSummary,
) -> RoutePathCandidateOrthogonalGraphViaExplainKind {
    match status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateOrthogonalGraphViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if summary.matching_via_count == 0 =>
        {
            RoutePathCandidateOrthogonalGraphViaExplainKind::NoMatchingAuthoredVia
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateOrthogonalGraphViaExplainKind::AllMatchingViasBlocked
        }
    }
}
