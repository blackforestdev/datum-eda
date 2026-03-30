use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RoutePathCandidateAuthoredCopperPlusOneGapPath,
    RoutePathCandidateAuthoredCopperPlusOneGapReport,
    RoutePathCandidateAuthoredCopperPlusOneGapSummary, RoutePathCandidateError,
    RoutePathCandidateStatus, StackupLayer,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateAuthoredCopperPlusOneGapExplainKind {
    DeterministicPathFound,
    NoExactOneGapPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperPlusOneGapExplainSelectedPath {
    pub path: RoutePathCandidateAuthoredCopperPlusOneGapPath,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperPlusOneGapExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateAuthoredCopperPlusOneGapExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateAuthoredCopperPlusOneGapSummary,
    pub selected_path: Option<RoutePathCandidateAuthoredCopperPlusOneGapExplainSelectedPath>,
}

impl Board {
    pub fn route_path_candidate_authored_copper_plus_one_gap_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateAuthoredCopperPlusOneGapExplainReport, RoutePathCandidateError>
    {
        let report = self.route_path_candidate_authored_copper_plus_one_gap(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
        Ok(map_plus_one_gap_explain_report(report))
    }
}

fn map_plus_one_gap_explain_report(
    report: RoutePathCandidateAuthoredCopperPlusOneGapReport,
) -> RoutePathCandidateAuthoredCopperPlusOneGapExplainReport {
    let explanation_kind = explanation_kind(&report);
    RoutePathCandidateAuthoredCopperPlusOneGapExplainReport {
        contract: "m5_route_path_candidate_authored_copper_plus_one_gap_explain_v1".to_string(),
        persisted_native_board_state_only: true,
        selection_rule: report.selection_rule,
        status: report.status.clone(),
        explanation_kind,
        net_uuid: report.net_uuid,
        net_name: report.net_name,
        from_anchor_pad_uuid: report.from_anchor_pad_uuid,
        to_anchor_pad_uuid: report.to_anchor_pad_uuid,
        candidate_copper_layers: report.candidate_copper_layers,
        summary: report.summary,
        selected_path: report.path.map(|path| {
            RoutePathCandidateAuthoredCopperPlusOneGapExplainSelectedPath {
                selection_reason: format!(
                    "selected because it is the first exact-one-gap path found under the deterministic selection rule with {} steps and {} gap step",
                    path.steps.len(),
                    path.steps.iter().filter(|step| step.object_uuid.is_none()).count()
                ),
                path,
            }
        }),
    }
}

fn explanation_kind(
    report: &RoutePathCandidateAuthoredCopperPlusOneGapReport,
) -> RoutePathCandidateAuthoredCopperPlusOneGapExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateAuthoredCopperPlusOneGapExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateAuthoredCopperPlusOneGapExplainKind::NoExactOneGapPath
        }
    }
}
