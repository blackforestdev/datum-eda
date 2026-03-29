use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RoutePathCandidateAuthoredCopperGraphObstacleAwareReport, RoutePathCandidateError,
    RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainKind {
    DeterministicPathFound,
    NoExistingAuthoredCopperPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainStep {
    pub kind:
        super::route_path_candidate_authored_copper_graph_obstacle_aware::RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView,
    pub object_uuid: Uuid,
    pub layer: LayerId,
    pub from: Point,
    pub to: Point,
    pub from_layer: Option<LayerId>,
    pub to_layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainSelectedPath {
    pub steps: Vec<RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainStep>,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_track_count: usize,
    pub candidate_via_count: usize,
    pub blocked_track_count: usize,
    pub blocked_via_count: usize,
    pub path_step_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainSummary,
    pub selected_path: Option<RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainSelectedPath>,
}

impl Board {
    pub fn route_path_candidate_authored_copper_graph_obstacle_aware_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainReport, RoutePathCandidateError>
    {
        let path_candidate = self.route_path_candidate_authored_copper_graph_obstacle_aware(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;

        let selected_path = path_candidate.path.as_ref().map(|path| {
            RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainSelectedPath {
                steps: path
                    .steps
                    .iter()
                    .map(|step| RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainStep {
                        kind: step.kind.clone(),
                        object_uuid: step.object_uuid,
                        layer: step.layer,
                        from: step.from,
                        to: step.to,
                        from_layer: step.from_layer,
                        to_layer: step.to_layer,
                    })
                    .collect(),
                selection_reason: format!(
                    "selected because it is the first minimum-step existing authored-copper path under the deterministic sorted graph traversal rule after excluding persisted target-net track/via edges blocked by current authored obstacle checks, with {} step(s)",
                    path.steps.len()
                ),
            }
        });

        Ok(RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainReport {
            contract: "m5_route_path_candidate_authored_copper_graph_obstacle_aware_explain_v1"
                .to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_track_count: path_candidate.summary.candidate_track_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
                blocked_track_count: path_candidate.summary.blocked_track_count,
                blocked_via_count: path_candidate.summary.blocked_via_count,
                path_step_count: path_candidate.summary.path_step_count,
            },
            selected_path,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateAuthoredCopperGraphObstacleAwareReport,
) -> RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainKind::NoExistingAuthoredCopperPath
        }
    }
}
