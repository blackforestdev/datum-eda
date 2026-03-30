use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError,
    RoutePathCandidateOrthogonalGraphReport, RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::{
    OrthogonalGraphEdgeOrientation, candidate_orthogonal_graph_layer_searches,
};

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
    pub blocked_edges: Vec<RoutePathCandidateOrthogonalGraphExplainBlockedEdge>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphExplainReport, RoutePathCandidateError> {
        let path_candidate = self.route_path_candidate_orthogonal_graph(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
        let preflight = self
            .route_preflight(net_uuid)
            .ok_or(RoutePathCandidateError::NetNotFound { net_uuid })?;
        let from_anchor = preflight
            .anchors
            .iter()
            .find(|anchor| anchor.pad_uuid == from_anchor_pad_uuid)
            .ok_or(RoutePathCandidateError::AnchorNotOnNet {
                pad_uuid: from_anchor_pad_uuid,
                net_uuid,
            })?;
        let to_anchor = preflight
            .anchors
            .iter()
            .find(|anchor| anchor.pad_uuid == to_anchor_pad_uuid)
            .ok_or(RoutePathCandidateError::AnchorNotOnNet {
                pad_uuid: to_anchor_pad_uuid,
                net_uuid,
            })?;
        let candidate_copper_layers = preflight
            .candidate_copper_layers
            .iter()
            .filter(|layer| layer.id == from_anchor.layer && layer.id == to_anchor.layer)
            .cloned()
            .collect::<Vec<_>>();
        let searches = candidate_orthogonal_graph_layer_searches(
            self,
            net_uuid,
            from_anchor,
            to_anchor,
            &candidate_copper_layers,
        );
        let selected_path = searches.iter().find_map(|search| {
            search.path.as_ref().map(|points| {
                RoutePathCandidateOrthogonalGraphExplainSelectedPath {
                    layer: search.layer,
                    points: points.clone(),
                    selection_reason: format!(
                        "selected because it is the first reachable same-layer orthogonal graph path under the deterministic graph-search rule on layer {}",
                        search.layer
                    ),
                }
            })
        });
        let blocked_edges = searches
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
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalGraphExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                graph_node_count: path_candidate.summary.graph_node_count,
                graph_edge_count: path_candidate.summary.graph_edge_count,
                blocked_edge_count: path_candidate.summary.blocked_edge_count,
            },
            selected_path,
            blocked_edges,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateOrthogonalGraphReport,
) -> RoutePathCandidateOrthogonalGraphExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateOrthogonalGraphExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.graph_node_count == 0 || report.summary.graph_edge_count == 0 =>
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
