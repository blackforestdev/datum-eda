use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError,
    RoutePathCandidateOrthogonalGraphPathCost, RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph::RoutePathCandidateOrthogonalGraphSegmentEvidence;
use super::route_path_candidate_orthogonal_graph::RoutePathCandidateOrthogonalGraphSummary;
use super::route_path_candidate_orthogonal_graph_selection::OrthogonalGraphEdgeOrientation;
use super::route_path_candidate_orthogonal_graph_spine::build_orthogonal_graph_candidate_spine;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalGraphExplainKind {
    DeterministicPathFound,
    NoSameLayerGraphCandidate,
    AllGraphPathsBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalGraphEdgeOrientationView {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphExplainSelectedPath {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub cost: RoutePathCandidateOrthogonalGraphPathCost,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphExplainBlockedEdge {
    pub layer: LayerId,
    pub from: Point,
    pub to: Point,
    pub orientation: RoutePathCandidateOrthogonalGraphEdgeOrientationView,
    pub blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub graph_node_count: usize,
    pub graph_edge_count: usize,
    pub blocked_edge_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateOrthogonalGraphExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphExplainSummary,
    pub selected_path: Option<RoutePathCandidateOrthogonalGraphExplainSelectedPath>,
    pub segment_evidence: Vec<RoutePathCandidateOrthogonalGraphSegmentEvidence>,
    pub blocked_edges: Vec<RoutePathCandidateOrthogonalGraphExplainBlockedEdge>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphExplainReport, RoutePathCandidateError> {
        let spine = build_orthogonal_graph_candidate_spine(
            self,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
        let selected_path = spine.searches.iter().find_map(|search| {
            search.path.as_ref().map(|points| {
                RoutePathCandidateOrthogonalGraphExplainSelectedPath {
                    layer: search.layer,
                    points: points.clone(),
                    cost: RoutePathCandidateOrthogonalGraphPathCost {
                        bend_count: points.len().saturating_sub(2),
                        segment_count: points.len().saturating_sub(1),
                        point_count: points.len(),
                    },
                    selection_reason: format!(
                        "selected because it is the lowest-cost reachable same-layer orthogonal graph path under the deterministic graph-search rule on layer {}",
                        search.layer
                    ),
                }
            })
        });
        let blocked_edges = spine
            .searches
            .iter()
            .flat_map(|search| {
                search.blocked_edges.iter().cloned().map(|edge| {
                    RoutePathCandidateOrthogonalGraphExplainBlockedEdge {
                        layer: edge.layer,
                        from: edge.from,
                        to: edge.to,
                        orientation: render_orientation(edge.orientation),
                        blockages: edge.blockages,
                    }
                })
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateOrthogonalGraphExplainReport {
            contract: "m5_route_path_candidate_orthogonal_graph_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: spine.status.clone(),
            explanation_kind: explanation_kind(spine.status.clone(), &spine.summary),
            net_uuid: spine.net_uuid,
            net_name: spine.net_name,
            from_anchor_pad_uuid: spine.from_anchor_pad_uuid,
            to_anchor_pad_uuid: spine.to_anchor_pad_uuid,
            selection_rule: spine.selection_rule,
            candidate_copper_layers: spine.candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalGraphExplainSummary {
                candidate_copper_layer_count: spine.summary.candidate_copper_layer_count,
                graph_node_count: spine.summary.graph_node_count,
                graph_edge_count: spine.summary.graph_edge_count,
                blocked_edge_count: spine.summary.blocked_edge_count,
            },
            selected_path,
            segment_evidence: spine.segment_evidence,
            blocked_edges,
        })
    }
}

fn explanation_kind(
    status: RoutePathCandidateStatus,
    summary: &RoutePathCandidateOrthogonalGraphSummary,
) -> RoutePathCandidateOrthogonalGraphExplainKind {
    match status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateOrthogonalGraphExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if summary.graph_node_count == 0 || summary.graph_edge_count == 0 =>
        {
            RoutePathCandidateOrthogonalGraphExplainKind::NoSameLayerGraphCandidate
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateOrthogonalGraphExplainKind::AllGraphPathsBlocked
        }
    }
}

fn render_orientation(
    orientation: OrthogonalGraphEdgeOrientation,
) -> RoutePathCandidateOrthogonalGraphEdgeOrientationView {
    match orientation {
        OrthogonalGraphEdgeOrientation::Horizontal => {
            RoutePathCandidateOrthogonalGraphEdgeOrientationView::Horizontal
        }
        OrthogonalGraphEdgeOrientation::Vertical => {
            RoutePathCandidateOrthogonalGraphEdgeOrientationView::Vertical
        }
    }
}
