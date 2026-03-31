use std::path::Path;

use super::*;
use crate::NativeProjectRouteApplyCandidateArg;
use crate::NativeProjectRouteApplySelectedView;
use crate::NativeProjectRouteProposalArtifactInspectionSegmentView;
use crate::NativeProjectRouteProposalArtifactRevalidationSegmentView;
use crate::NativeProjectRouteProposalArtifactRevalidationView;
use crate::NativeProjectRouteProposalExplainView;
use crate::NativeProjectRouteProposalProfileArg;
use crate::NativeProjectRouteProposalReviewView;
use crate::NativeProjectRouteProposalSelectionCandidateView;
use crate::NativeProjectRouteProposalSelectionView;
use crate::NativeProjectRouteStrategyBatchEntryView;
use crate::NativeProjectRouteStrategyBatchEvaluateView;
use crate::NativeProjectRouteStrategyBatchRequestIdentityView;
use crate::NativeProjectRouteStrategyBatchResultComparisonArtifactView;
use crate::NativeProjectRouteStrategyBatchResultComparisonCountDeltaView;
use crate::NativeProjectRouteStrategyBatchResultComparisonRequestChangeView;
use crate::NativeProjectRouteStrategyBatchResultComparisonView;
use crate::NativeProjectRouteStrategyBatchResultGateView;
use crate::NativeProjectRouteStrategyBatchResultInspectionView;
use crate::NativeProjectRouteStrategyBatchResultMalformedEntryView;
use crate::NativeProjectRouteStrategyBatchResultValidationView;
use crate::NativeProjectRouteStrategyBatchResultsIndexEntryView;
use crate::NativeProjectRouteStrategyBatchResultsIndexGateSummaryView;
use crate::NativeProjectRouteStrategyBatchResultsIndexSummaryView;
use crate::NativeProjectRouteStrategyBatchResultsIndexView;
use crate::NativeProjectRouteStrategyBatchSummaryView;
use crate::NativeProjectRouteStrategyCompareView;
use crate::NativeProjectRouteStrategyComparisonEntryView;
use crate::NativeProjectRouteStrategyCuratedBaselineCaptureView;
use crate::NativeProjectRouteStrategyCuratedFixtureSuiteEntryView;
use crate::NativeProjectRouteStrategyCuratedFixtureSuiteView;
use crate::NativeProjectRouteStrategyDeltaProfileView;
use crate::NativeProjectRouteStrategyDeltaView;
use crate::NativeProjectRouteStrategyReportView;
use crate::NativeProjectSelectedRouteProposalExportView;
use crate::cli_args::NativeProjectRouteStrategyBatchGatePolicyArg;
use crate::cli_args::NativeRoutePathCandidateAuthoredCopperGraphPolicy;
use eda_engine::board::{
    RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphPolicyStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView,
    RoutePathCandidateAuthoredCopperPlusOneGapStepKindView, RoutePathCandidateStatus,
};
use serde_json::Value;

const ROUTE_PROPOSAL_ARTIFACT_KIND: &str = "native_route_proposal_artifact";
const ROUTE_PROPOSAL_ARTIFACT_VERSION: u32 = 1;
const ROUTE_STRATEGY_BATCH_REQUESTS_KIND: &str = "native_route_strategy_batch_requests";
const ROUTE_STRATEGY_BATCH_REQUESTS_VERSION: u32 = 1;
const ROUTE_STRATEGY_BATCH_RESULT_KIND: &str = "native_route_strategy_batch_result_artifact";
const ROUTE_STRATEGY_BATCH_RESULT_VERSION: u32 = 1;
const ROUTE_STRATEGY_CURATED_FIXTURE_SUITE_ID: &str = "m6_route_strategy_curated_fixture_suite_v1";
const ROUTE_PROPOSAL_REASON_AUTHORED_COPPER_PLUS_ONE_GAP: &str = "authored_copper_plus_one_gap";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE: &str = "route_path_candidate";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_VIA: &str = "route_path_candidate_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_TWO_VIA: &str = "route_path_candidate_two_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_THREE_VIA: &str = "route_path_candidate_three_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FOUR_VIA: &str = "route_path_candidate_four_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FIVE_VIA: &str = "route_path_candidate_five_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_SIX_VIA: &str = "route_path_candidate_six_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN: &str =
    "route_path_candidate_authored_via_chain";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG: &str =
    "route_path_candidate_orthogonal_dogleg";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND: &str =
    "route_path_candidate_orthogonal_two_bend";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH: &str =
    "route_path_candidate_orthogonal_graph";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA: &str =
    "route_path_candidate_orthogonal_graph_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA: &str =
    "route_path_candidate_orthogonal_graph_two_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA: &str =
    "route_path_candidate_orthogonal_graph_three_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FOUR_VIA: &str =
    "route_path_candidate_orthogonal_graph_four_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FIVE_VIA: &str =
    "route_path_candidate_orthogonal_graph_five_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA: &str =
    "route_path_candidate_orthogonal_graph_six_via";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_zone_aware";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_zone_obstacle_aware";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE: &str =
    "route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE_LAYER_BALANCE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_obstacle_aware";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_PLAIN: &str =
    "route_path_candidate_authored_copper_graph_policy_plain";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_policy_zone_aware";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_OBSTACLE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_policy_obstacle_aware";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_policy_zone_obstacle_aware";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_AWARE: &str =
    "route_path_candidate_authored_copper_graph_policy_zone_obstacle_topology_aware";
const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_LAYER_BALANCE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_policy_zone_obstacle_topology_layer_balance_aware";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeProjectRouteProposalActionView {
    pub(crate) action_id: String,
    pub(crate) proposal_action: String,
    pub(crate) reason: String,
    pub(crate) contract: String,
    pub(crate) net_uuid: Uuid,
    pub(crate) net_name: String,
    pub(crate) from_anchor_pad_uuid: Uuid,
    pub(crate) to_anchor_pad_uuid: Uuid,
    pub(crate) layer: i32,
    pub(crate) width_nm: i64,
    pub(crate) from: Point,
    pub(crate) to: Point,
    pub(crate) reused_via_uuid: Option<Uuid>,
    #[serde(default)]
    pub(crate) reused_via_uuids: Vec<Uuid>,
    #[serde(default)]
    pub(crate) reused_object_kind: Option<String>,
    #[serde(default)]
    pub(crate) reused_object_uuid: Option<Uuid>,
    #[serde(default)]
    pub(crate) reused_object_from_layer: Option<i32>,
    #[serde(default)]
    pub(crate) reused_object_to_layer: Option<i32>,
    #[serde(default)]
    pub(crate) selected_path_bend_count: usize,
    pub(crate) selected_path_point_count: usize,
    pub(crate) selected_path_segment_index: usize,
    pub(crate) selected_path_segment_count: usize,
    #[serde(default)]
    pub(crate) selected_path_layer_segment_index: Option<usize>,
    #[serde(default)]
    pub(crate) selected_path_layer_segment_count: Option<usize>,
    #[serde(default)]
    pub(crate) selected_path_layer_segment_bend_count: Option<usize>,
    #[serde(default)]
    pub(crate) selected_path_layer_segment_point_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RouteProposalArtifact {
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) project_uuid: Uuid,
    pub(crate) project_name: String,
    pub(crate) contract: String,
    pub(crate) actions: Vec<NativeProjectRouteProposalActionView>,
}

pub(crate) struct LoadedRouteProposalArtifact {
    pub(crate) artifact_path: PathBuf,
    pub(crate) source_version: u32,
    pub(crate) artifact: RouteProposalArtifact,
}

struct LoadedRouteStrategyBatchResultArtifact {
    artifact_path: PathBuf,
    source_version: u32,
    artifact: NativeProjectRouteStrategyBatchEvaluateView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RouteStrategyBatchRequestsManifest {
    kind: String,
    version: u32,
    requests: Vec<RouteStrategyBatchRequestInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RouteStrategyBatchRequestInput {
    request_id: String,
    fixture_id: String,
    project_root: PathBuf,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
}

struct CuratedRouteStrategyFixtureSpec {
    request_id: &'static str,
    fixture_id: &'static str,
    coverage_labels: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrthogonalGraphArtifactDriftKind {
    CandidateAvailabilityChanged,
    DeterministicCostWinnerChanged,
    GeometryChanged,
}

impl OrthogonalGraphArtifactDriftKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::CandidateAvailabilityChanged => "candidate_availability_changed",
            Self::DeterministicCostWinnerChanged => "deterministic_cost_winner_changed",
            Self::GeometryChanged => "geometry_changed",
        }
    }
}

struct RouteProposalArtifactRevalidationState {
    live_actions: Result<Vec<NativeProjectRouteProposalActionView>>,
    matches_live: bool,
    drift_kind: Option<OrthogonalGraphArtifactDriftKind>,
    drift_message: Option<String>,
}

#[derive(Debug, Clone)]
struct OrthogonalGraphArtifactSegmentFacts {
    layer_segment_index: usize,
    layer_segment_count: usize,
    layer: i32,
    bend_count: usize,
    point_count: usize,
    track_action_count: usize,
}

pub(crate) fn export_native_project_route_path_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    candidate: NativeProjectRouteApplyCandidateArg,
    policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    let actions = build_route_path_proposal_actions(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        candidate,
        policy,
    )?;
    export_route_proposal_artifact(root, output_path, "export_route_path_proposal", actions)
}

fn export_route_proposal_artifact(
    root: &Path,
    output_path: &Path,
    action: &str,
    actions: Vec<NativeProjectRouteProposalActionView>,
) -> Result<NativeProjectRouteProposalExportReportView> {
    let project = load_native_project(root)?;
    let segment_evidence =
        orthogonal_graph_route_proposal_artifact_inspection_segment_evidence(&actions);
    let selected_path_bend_count = actions
        .first()
        .map(|action| action.selected_path_bend_count)
        .unwrap_or(0);
    let selected_path_point_count = actions
        .first()
        .map(|action| action.selected_path_point_count)
        .unwrap_or(0);
    let selected_path_segment_count = actions
        .first()
        .map(|action| action.selected_path_segment_count)
        .unwrap_or(0);
    let artifact = RouteProposalArtifact {
        kind: ROUTE_PROPOSAL_ARTIFACT_KIND.to_string(),
        version: ROUTE_PROPOSAL_ARTIFACT_VERSION,
        project_uuid: project.manifest.uuid,
        project_name: project.manifest.name.clone(),
        contract: actions[0].contract.clone(),
        actions,
    };
    write_canonical_json(output_path, &artifact)?;
    Ok(NativeProjectRouteProposalExportReportView {
        action: action.to_string(),
        artifact_path: output_path.display().to_string(),
        kind: artifact.kind,
        version: artifact.version,
        project_uuid: artifact.project_uuid.to_string(),
        contract: artifact.contract,
        actions: artifact.actions.len(),
        selected_path_bend_count,
        selected_path_point_count,
        selected_path_segment_count,
        segment_evidence,
    })
}

fn build_route_path_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    candidate: NativeProjectRouteApplyCandidateArg,
    policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    if policy.is_some() && candidate != NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph {
        bail!(
            "export-route-path-proposal --policy is supported only for candidate authored-copper-graph"
        );
    }

    match candidate {
        NativeProjectRouteApplyCandidateArg::RoutePathCandidate => {
            build_route_path_candidate_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateVia => {
            build_route_path_candidate_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateTwoVia => {
            build_route_path_candidate_two_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateThreeVia => {
            build_route_path_candidate_three_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFourVia => {
            build_route_path_candidate_four_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFiveVia => {
            build_route_path_candidate_five_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateSixVia => {
            build_route_path_candidate_six_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateAuthoredViaChain => {
            build_route_path_candidate_authored_via_chain_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalDogleg => {
            build_route_path_candidate_orthogonal_dogleg_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalTwoBend => {
            build_route_path_candidate_orthogonal_two_bend_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraph => {
            build_route_path_candidate_orthogonal_graph_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphVia => {
            build_route_path_candidate_orthogonal_graph_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphTwoVia => {
            build_route_path_candidate_orthogonal_graph_two_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphThreeVia => {
            build_route_path_candidate_orthogonal_graph_three_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFourVia => {
            build_route_path_candidate_orthogonal_graph_four_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFiveVia => {
            build_route_path_candidate_orthogonal_graph_five_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphSixVia => {
            build_route_path_candidate_orthogonal_graph_six_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperPlusOneGap => {
            build_plus_one_gap_route_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph => {
            let policy = policy.ok_or_else(|| {
                anyhow::anyhow!(
                    "export-route-path-proposal candidate authored-copper-graph requires --policy"
                )
            })?;
            build_route_path_candidate_authored_copper_graph_policy_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                policy,
            )
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct RouteProposalSelectionCandidateSpec {
    candidate: NativeProjectRouteApplyCandidateArg,
    policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
}

struct RouteProposalSelectionOutcome {
    report: NativeProjectRouteProposalSelectionView,
    selected_spec: Option<RouteProposalSelectionCandidateSpec>,
}

pub(crate) fn select_native_project_route_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    profile: NativeProjectRouteProposalProfileArg,
) -> Result<NativeProjectRouteProposalSelectionView> {
    Ok(run_native_project_route_proposal_selection(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        profile,
    )?
    .report)
}

pub(crate) fn explain_native_project_route_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    profile: NativeProjectRouteProposalProfileArg,
) -> Result<NativeProjectRouteProposalExplainView> {
    let selection = run_native_project_route_proposal_selection(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        profile,
    )?;
    let explanation = if let Some(selected_candidate) = &selection.report.selected_candidate {
        format!(
            "selected {} because it was the first candidate in deterministic order that produced a valid proposal action set",
            selected_candidate
        )
    } else {
        "no candidate produced a valid proposal action set under current authored constraints"
            .to_string()
    };
    Ok(NativeProjectRouteProposalExplainView {
        action: "route_proposal_explain".to_string(),
        project_root: selection.report.project_root,
        net_uuid: selection.report.net_uuid,
        from_anchor_pad_uuid: selection.report.from_anchor_pad_uuid,
        to_anchor_pad_uuid: selection.report.to_anchor_pad_uuid,
        selection_profile: selection.report.selection_profile,
        status: selection.report.status,
        selection_rule: selection.report.selection_rule,
        selected_candidate: selection.report.selected_candidate,
        selected_policy: selection.report.selected_policy,
        selected_contract: selection.report.selected_contract,
        explanation,
        candidates: selection.report.candidates,
    })
}

pub(crate) fn review_native_project_route_proposal(
    root: Option<&Path>,
    net_uuid: Option<Uuid>,
    from_anchor_pad_uuid: Option<Uuid>,
    to_anchor_pad_uuid: Option<Uuid>,
    profile: NativeProjectRouteProposalProfileArg,
    artifact_path: Option<&Path>,
) -> Result<NativeProjectRouteProposalReviewView> {
    if let Some(artifact_path) = artifact_path {
        return review_route_proposal_artifact(artifact_path);
    }
    let root = root.ok_or_else(|| {
        anyhow::anyhow!("review-route-proposal requires <dir> when --artifact is not provided")
    })?;
    let net_uuid = net_uuid.ok_or_else(|| {
        anyhow::anyhow!("review-route-proposal requires --net when --artifact is not provided")
    })?;
    let from_anchor_pad_uuid = from_anchor_pad_uuid.ok_or_else(|| {
        anyhow::anyhow!(
            "review-route-proposal requires --from-anchor when --artifact is not provided"
        )
    })?;
    let to_anchor_pad_uuid = to_anchor_pad_uuid.ok_or_else(|| {
        anyhow::anyhow!(
            "review-route-proposal requires --to-anchor when --artifact is not provided"
        )
    })?;
    review_selected_native_project_route_proposal(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        profile,
    )
}

pub(crate) fn report_native_project_route_strategy(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    objective: NativeProjectRouteProposalProfileArg,
) -> Result<NativeProjectRouteStrategyReportView> {
    let selection = run_native_project_route_proposal_selection(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        objective,
    )?;
    let objective_name = route_proposal_profile_name(objective).to_string();
    let recommendation_rule = format!(
        "objective {} maps directly to selector profile {} using the accepted deterministic M6 objective/profile table",
        objective_name, objective_name
    );
    let objective_explanation = match objective {
        NativeProjectRouteProposalProfileArg::Default => {
            "objective default preserves the accepted selector family order and recommends selector profile default without reprioritizing candidate families".to_string()
        }
        NativeProjectRouteProposalProfileArg::AuthoredCopperPriority => {
            "objective authored-copper-priority recommends the existing authored-copper-priority selector profile, which prepends the accepted authored-copper-graph policy family ahead of the unchanged default family order".to_string()
        }
    };
    let explanation = if let Some(selected_candidate) = &selection.report.selected_candidate {
        format!(
            "{} Current selector result under that profile chooses {}.",
            objective_explanation, selected_candidate
        )
    } else {
        format!(
            "{} Current selector result under that profile finds no selectable route proposal.",
            objective_explanation
        )
    };
    let next_step_command = format!(
        "project route-proposal {} --net {} --from-anchor {} --to-anchor {} --profile {}",
        root.display(),
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        objective_name
    );
    Ok(NativeProjectRouteStrategyReportView {
        action: "route_strategy_report".to_string(),
        project_root: root.display().to_string(),
        net_uuid: net_uuid.to_string(),
        from_anchor_pad_uuid: from_anchor_pad_uuid.to_string(),
        to_anchor_pad_uuid: to_anchor_pad_uuid.to_string(),
        objective: objective_name.clone(),
        recommended_profile: objective_name,
        recommendation_rule,
        explanation,
        selector_status: selection.report.status,
        selector_rule: selection.report.selection_rule,
        selected_candidate: selection.report.selected_candidate,
        selected_policy: selection.report.selected_policy,
        selected_contract: selection.report.selected_contract,
        selected_actions: selection.report.selected_actions,
        next_step_command,
    })
}

fn review_selected_native_project_route_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    profile: NativeProjectRouteProposalProfileArg,
) -> Result<NativeProjectRouteProposalReviewView> {
    let selection = run_native_project_route_proposal_selection(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        profile,
    )?;
    let selected_spec = selection.selected_spec.ok_or_else(|| {
        anyhow::anyhow!(
            "route-proposal found no selectable route under current authored constraints"
        )
    })?;
    let actions = build_route_path_proposal_actions(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selected_spec.candidate,
        selected_spec.policy,
    )?;
    super::command_project_route_apply::validate_route_proposal_actions(&actions)?;
    let first_action = actions.first().ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal review candidate {} produced no actions",
            route_proposal_selection_spec_name(&selected_spec)
        )
    })?;
    let segment_evidence =
        orthogonal_graph_route_proposal_artifact_inspection_segment_evidence(&actions);
    Ok(NativeProjectRouteProposalReviewView {
        action: "review_route_proposal".to_string(),
        review_source: "selected_route_proposal".to_string(),
        status: "deterministic_route_proposal_ready".to_string(),
        explanation: format!(
            "reviewing the currently selected deterministic route proposal chosen by profile {}",
            route_proposal_profile_name(profile)
        ),
        project_root: Some(root.display().to_string()),
        artifact_path: None,
        kind: None,
        source_version: None,
        version: None,
        project_uuid: None,
        project_name: None,
        net_uuid: Some(net_uuid.to_string()),
        from_anchor_pad_uuid: Some(from_anchor_pad_uuid.to_string()),
        to_anchor_pad_uuid: Some(to_anchor_pad_uuid.to_string()),
        selection_profile: Some(route_proposal_profile_name(profile).to_string()),
        selection_rule: Some(selection.report.selection_rule),
        selected_candidate: Some(route_apply_candidate_name(selected_spec.candidate).to_string()),
        selected_policy: selected_spec
            .policy
            .map(route_authored_copper_graph_policy_name),
        contract: first_action.contract.clone(),
        actions: actions.len(),
        draw_track_actions: actions
            .iter()
            .filter(|action| action.proposal_action == "draw_track")
            .count(),
        selected_path_bend_count: first_action.selected_path_bend_count,
        selected_path_point_count: first_action.selected_path_point_count,
        selected_path_segment_count: first_action.selected_path_segment_count,
        segment_evidence,
        proposal_actions: actions,
    })
}

pub(crate) fn compare_native_project_route_strategy(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<NativeProjectRouteStrategyCompareView> {
    let profiles = accepted_route_strategy_profiles();
    let comparison_rule = format!(
        "compare accepted objectives/profiles in deterministic order {} and recommend the first profile in that same order that yields a selectable proposal; if multiple profiles yield proposals, keep the earlier baseline-preserving profile",
        profiles
            .iter()
            .map(|profile| route_proposal_profile_name(*profile))
            .collect::<Vec<_>>()
            .join(" > ")
    );
    let mut entries = Vec::with_capacity(profiles.len());
    for profile in profiles {
        let report = report_native_project_route_strategy(
            root,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
        )?;
        entries.push(NativeProjectRouteStrategyComparisonEntryView {
            objective: report.objective,
            profile: report.recommended_profile,
            proposal_available: report.selected_candidate.is_some(),
            selector_status: report.selector_status,
            selected_candidate: report.selected_candidate,
            selected_policy: report.selected_policy,
            selected_contract: report.selected_contract,
            selected_actions: report.selected_actions,
            distinction: route_strategy_profile_distinction(profile).to_string(),
        });
    }
    let recommended_entry = entries
        .iter()
        .find(|entry| entry.proposal_available)
        .unwrap_or_else(|| &entries[0]);
    let recommendation_reason = if recommended_entry.proposal_available {
        if recommended_entry.profile == "default" {
            "recommended default because it yields a proposal while preserving the baseline accepted selector order".to_string()
        } else {
            format!(
                "recommended {} because earlier accepted profiles yielded no proposal and this profile yields the first selectable proposal in deterministic comparison order",
                recommended_entry.profile
            )
        }
    } else {
        "no accepted profile yields a proposal; recommend default as the baseline inspection profile because it preserves the accepted selector order".to_string()
    };
    let next_step_command = format!(
        "project route-proposal {} --net {} --from-anchor {} --to-anchor {} --profile {}",
        root.display(),
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        recommended_entry.profile
    );
    let recommended_objective = recommended_entry.objective.clone();
    let recommended_profile = recommended_entry.profile.clone();
    Ok(NativeProjectRouteStrategyCompareView {
        action: "route_strategy_compare".to_string(),
        project_root: root.display().to_string(),
        net_uuid: net_uuid.to_string(),
        from_anchor_pad_uuid: from_anchor_pad_uuid.to_string(),
        to_anchor_pad_uuid: to_anchor_pad_uuid.to_string(),
        comparison_rule,
        recommended_objective,
        recommended_profile,
        recommendation_reason,
        next_step_command,
        entries,
    })
}

pub(crate) fn report_native_project_route_strategy_delta(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<NativeProjectRouteStrategyDeltaView> {
    let comparison = compare_native_project_route_strategy(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    let default_entry = comparison
        .entries
        .iter()
        .find(|entry| entry.profile == "default")
        .ok_or_else(|| anyhow::anyhow!("route strategy comparison missing default profile"))?;
    let authored_entry = comparison
        .entries
        .iter()
        .find(|entry| entry.profile == "authored-copper-priority")
        .ok_or_else(|| {
            anyhow::anyhow!("route strategy comparison missing authored-copper-priority profile")
        })?;
    let outcomes_match = default_entry.proposal_available == authored_entry.proposal_available
        && default_entry.selected_candidate == authored_entry.selected_candidate
        && default_entry.selected_policy == authored_entry.selected_policy;
    let delta_classification =
        if !default_entry.proposal_available && !authored_entry.proposal_available {
            "no_proposal_under_any_profile"
        } else if !default_entry.proposal_available && authored_entry.proposal_available {
            "proposal_available_only_under_authored_copper_priority"
        } else if outcomes_match {
            "same_outcome"
        } else if default_entry.selected_candidate == authored_entry.selected_candidate {
            "different_policy_same_family"
        } else {
            "different_candidate_family"
        };
    let outcome_relation = if outcomes_match {
        "identical".to_string()
    } else {
        "different".to_string()
    };
    let material_difference = match delta_classification {
        "no_proposal_under_any_profile" => {
            "neither accepted profile currently yields a selectable route proposal".to_string()
        }
        "proposal_available_only_under_authored_copper_priority" => {
            "only authored-copper-priority currently yields a selectable proposal because the baseline default profile does not find one".to_string()
        }
        "same_outcome" => {
            "both accepted profiles currently resolve to the same live selector outcome, so changing profiles would not change the proposed route".to_string()
        }
        "different_policy_same_family" => {
            "both accepted profiles currently select the same candidate family but with different bounded policy details".to_string()
        }
        "different_candidate_family" => {
            "the accepted profiles currently resolve to different candidate families, so the choice changes whether the engine prefers baseline synthesis or authored-copper reuse first".to_string()
        }
        _ => unreachable!("unhandled delta classification"),
    };
    let profiles = comparison
        .entries
        .iter()
        .map(|entry| NativeProjectRouteStrategyDeltaProfileView {
            objective: entry.objective.clone(),
            profile: entry.profile.clone(),
            proposal_available: entry.proposal_available,
            selected_candidate: entry.selected_candidate.clone(),
            selected_policy: entry.selected_policy.clone(),
        })
        .collect();
    Ok(NativeProjectRouteStrategyDeltaView {
        action: "route_strategy_delta".to_string(),
        project_root: comparison.project_root,
        net_uuid: comparison.net_uuid,
        from_anchor_pad_uuid: comparison.from_anchor_pad_uuid,
        to_anchor_pad_uuid: comparison.to_anchor_pad_uuid,
        compared_objectives: comparison
            .entries
            .iter()
            .map(|entry| entry.objective.clone())
            .collect(),
        compared_profiles: comparison
            .entries
            .iter()
            .map(|entry| entry.profile.clone())
            .collect(),
        outcomes_match,
        outcome_relation,
        delta_classification: delta_classification.to_string(),
        recommendation_summary: comparison.recommendation_reason,
        material_difference,
        recommended_objective: comparison.recommended_objective,
        recommended_profile: comparison.recommended_profile,
        profiles,
    })
}

fn write_route_strategy_batch_requests_manifest(
    path: &Path,
    requests: &[RouteStrategyBatchRequestInput],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    write_canonical_json(
        path,
        &serde_json::json!({
            "kind": ROUTE_STRATEGY_BATCH_REQUESTS_KIND,
            "version": ROUTE_STRATEGY_BATCH_REQUESTS_VERSION,
            "requests": requests,
        }),
    )
}

fn write_route_strategy_fixture_board(root: &Path, board: &Value) -> Result<()> {
    write_canonical_json(&root.join("board/board.json"), board)
}

fn seed_curated_route_strategy_same_outcome_fixture(root: &Path) -> Result<(Uuid, Uuid, Uuid)> {
    create_native_project(
        root,
        Some("Route Strategy Curated Same Outcome Demo".to_string()),
    )?;

    let target_net_uuid = Uuid::from_u128(0xc200);
    let other_net_uuid = Uuid::from_u128(0xc201);
    let class_uuid = Uuid::from_u128(0xc202);
    let package_a_uuid = Uuid::from_u128(0xc203);
    let package_b_uuid = Uuid::from_u128(0xc204);
    let anchor_a_uuid = Uuid::from_u128(0xc205);
    let anchor_b_uuid = Uuid::from_u128(0xc206);
    write_route_strategy_fixture_board(
        root,
        &serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::from_u128(0xc207),
            "name": "Route Strategy Curated Same Outcome Demo Board",
            "stackup": {
                "layers": [
                    { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                    { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                    { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                ]
            },
            "outline": {
                "vertices": [
                    { "x": 0, "y": 0 },
                    { "x": 5000000, "y": 0 },
                    { "x": 5000000, "y": 3000000 },
                    { "x": 0, "y": 3000000 }
                ],
                "closed": true
            },
            "packages": {},
            "pads": {
                anchor_a_uuid.to_string(): {
                    "uuid": anchor_a_uuid,
                    "package": package_a_uuid,
                    "name": "1",
                    "net": target_net_uuid,
                    "position": { "x": 500000, "y": 600000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 450000,
                    "width": 0,
                    "height": 0
                },
                anchor_b_uuid.to_string(): {
                    "uuid": anchor_b_uuid,
                    "package": package_b_uuid,
                    "name": "1",
                    "net": target_net_uuid,
                    "position": { "x": 4500000, "y": 2400000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 450000,
                    "width": 0,
                    "height": 0
                }
            },
            "tracks": {},
            "vias": {},
            "zones": {},
            "nets": {
                target_net_uuid.to_string(): {
                    "uuid": target_net_uuid,
                    "name": "SIG",
                    "class": class_uuid
                },
                other_net_uuid.to_string(): {
                    "uuid": other_net_uuid,
                    "name": "GND",
                    "class": class_uuid
                }
            },
            "net_classes": {
                class_uuid.to_string(): {
                    "uuid": class_uuid,
                    "name": "Default",
                    "clearance": 150000,
                    "track_width": 200000,
                    "via_drill": 300000,
                    "via_diameter": 600000,
                    "diffpair_width": 0,
                    "diffpair_gap": 0
                }
            },
            "keepouts": [],
            "dimensions": [],
            "texts": []
        }),
    )?;
    Ok((target_net_uuid, anchor_a_uuid, anchor_b_uuid))
}

fn seed_curated_route_strategy_profile_divergence_fixture(
    root: &Path,
) -> Result<(Uuid, Uuid, Uuid)> {
    create_native_project(
        root,
        Some("Route Strategy Curated Profile Divergence Demo".to_string()),
    )?;

    let target_net_uuid = Uuid::from_u128(0xc280);
    let class_uuid = Uuid::from_u128(0xc281);
    let package_a_uuid = Uuid::from_u128(0xc282);
    let package_b_uuid = Uuid::from_u128(0xc283);
    let anchor_a_uuid = Uuid::from_u128(0xc284);
    let anchor_b_uuid = Uuid::from_u128(0xc285);
    let authored_track_uuid = Uuid::from_u128(0xc286);
    write_route_strategy_fixture_board(
        root,
        &serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::from_u128(0xc287),
            "name": "Route Strategy Curated Profile Divergence Demo Board",
            "stackup": {
                "layers": [
                    { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                ]
            },
            "outline": {
                "vertices": [
                    { "x": 0, "y": 0 },
                    { "x": 4000000, "y": 0 },
                    { "x": 4000000, "y": 1000000 },
                    { "x": 0, "y": 1000000 }
                ],
                "closed": true
            },
            "packages": {},
            "pads": {
                anchor_a_uuid.to_string(): {
                    "uuid": anchor_a_uuid,
                    "package": package_a_uuid,
                    "name": "1",
                    "net": target_net_uuid,
                    "position": { "x": 500000, "y": 500000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 400000,
                    "width": 0,
                    "height": 0
                },
                anchor_b_uuid.to_string(): {
                    "uuid": anchor_b_uuid,
                    "package": package_b_uuid,
                    "name": "1",
                    "net": target_net_uuid,
                    "position": { "x": 3500000, "y": 500000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 400000,
                    "width": 0,
                    "height": 0
                }
            },
            "tracks": {
                authored_track_uuid.to_string(): {
                    "uuid": authored_track_uuid,
                    "net": target_net_uuid,
                    "from": { "x": 500000, "y": 500000 },
                    "to": { "x": 3500000, "y": 500000 },
                    "width": 200000,
                    "layer": 1
                }
            },
            "vias": {},
            "zones": {},
            "nets": {
                target_net_uuid.to_string(): {
                    "uuid": target_net_uuid,
                    "name": "SIG",
                    "class": class_uuid
                }
            },
            "net_classes": {
                class_uuid.to_string(): {
                    "uuid": class_uuid,
                    "name": "Default",
                    "clearance": 150000,
                    "track_width": 200000,
                    "via_drill": 300000,
                    "via_diameter": 600000,
                    "diffpair_width": 0,
                    "diffpair_gap": 0
                }
            },
            "keepouts": [],
            "dimensions": [],
            "texts": []
        }),
    )?;
    Ok((target_net_uuid, anchor_a_uuid, anchor_b_uuid))
}

fn seed_curated_route_strategy_via_fixture(root: &Path) -> Result<(Uuid, Uuid, Uuid)> {
    create_native_project(root, Some("Route Strategy Curated Via Demo".to_string()))?;

    let target_net_uuid = Uuid::from_u128(0xa10);
    let class_uuid = Uuid::from_u128(0xa11);
    let package_a_uuid = Uuid::from_u128(0xa12);
    let package_b_uuid = Uuid::from_u128(0xa13);
    let anchor_a_uuid = Uuid::from_u128(0xa14);
    let anchor_b_uuid = Uuid::from_u128(0xa15);
    let via_uuid = Uuid::from_u128(0xa16);
    write_route_strategy_fixture_board(
        root,
        &serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::from_u128(0xa17),
            "name": "Route Strategy Curated Via Demo Board",
            "stackup": {
                "layers": [
                    { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                    { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                    { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                ]
            },
            "outline": {
                "vertices": [
                    { "x": 0, "y": 0 },
                    { "x": 5000000, "y": 0 },
                    { "x": 5000000, "y": 3000000 },
                    { "x": 0, "y": 3000000 }
                ],
                "closed": true
            },
            "packages": {},
            "pads": {
                anchor_a_uuid.to_string(): {
                    "uuid": anchor_a_uuid,
                    "package": package_a_uuid,
                    "name": "1",
                    "net": target_net_uuid,
                    "position": { "x": 500000, "y": 600000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 450000,
                    "width": 0,
                    "height": 0
                },
                anchor_b_uuid.to_string(): {
                    "uuid": anchor_b_uuid,
                    "package": package_b_uuid,
                    "name": "1",
                    "net": target_net_uuid,
                    "position": { "x": 4500000, "y": 2400000 },
                    "layer": 3,
                    "shape": "circle",
                    "diameter": 450000,
                    "width": 0,
                    "height": 0
                }
            },
            "tracks": {},
            "vias": {
                via_uuid.to_string(): {
                    "uuid": via_uuid,
                    "net": target_net_uuid,
                    "position": { "x": 2500000, "y": 1500000 },
                    "drill": 300000,
                    "diameter": 600000,
                    "from_layer": 1,
                    "to_layer": 3
                }
            },
            "zones": {},
            "nets": {
                target_net_uuid.to_string(): {
                    "uuid": target_net_uuid,
                    "name": "SIG",
                    "class": class_uuid
                }
            },
            "net_classes": {
                class_uuid.to_string(): {
                    "uuid": class_uuid,
                    "name": "Default",
                    "clearance": 150000,
                    "track_width": 200000,
                    "via_drill": 300000,
                    "via_diameter": 600000,
                    "diffpair_width": 0,
                    "diffpair_gap": 0
                }
            },
            "keepouts": [],
            "dimensions": [],
            "texts": []
        }),
    )?;
    Ok((target_net_uuid, anchor_a_uuid, anchor_b_uuid))
}

fn seed_curated_route_strategy_no_proposal_fixture(root: &Path) -> Result<(Uuid, Uuid, Uuid)> {
    let (net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid) =
        seed_curated_route_strategy_same_outcome_fixture(root)?;
    let board_path = root.join("board/board.json");
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&board_path)
            .with_context(|| format!("failed to read {}", board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", board_path.display()))?;
    board["nets"][net_uuid.to_string()]["class"] = Value::Null;
    write_route_strategy_fixture_board(root, &board)?;
    Ok((net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid))
}

pub(crate) fn write_route_strategy_curated_fixture_suite(
    out_dir: &Path,
    manifest_path_override: Option<&Path>,
) -> Result<NativeProjectRouteStrategyCuratedFixtureSuiteView> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let manifest_path = manifest_path_override
        .map(PathBuf::from)
        .unwrap_or_else(|| out_dir.join("route-strategy-batch-requests.json"));

    let fixture_specs = [
        CuratedRouteStrategyFixtureSpec {
            request_id: "same-outcome-default",
            fixture_id: "same-outcome-default",
            coverage_labels: &["same_outcome", "baseline_route_path_candidate"],
        },
        CuratedRouteStrategyFixtureSpec {
            request_id: "profile-divergence-authored-copper",
            fixture_id: "profile-divergence-authored-copper",
            coverage_labels: &[
                "different_candidate_family",
                "authored_copper_reuse_priority",
            ],
        },
        CuratedRouteStrategyFixtureSpec {
            request_id: "no-proposal-null-net-class",
            fixture_id: "no-proposal-null-net-class",
            coverage_labels: &["no_proposal_under_any_profile", "null_net_class"],
        },
        CuratedRouteStrategyFixtureSpec {
            request_id: "via-available",
            fixture_id: "via-available",
            coverage_labels: &["same_outcome", "cross_layer_routable"],
        },
    ];

    let mut requests = Vec::with_capacity(fixture_specs.len());
    let mut fixtures = Vec::with_capacity(fixture_specs.len());
    for spec in fixture_specs {
        let fixture_root = out_dir.join(spec.fixture_id);
        let (net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid) = match spec.fixture_id {
            "same-outcome-default" => {
                seed_curated_route_strategy_same_outcome_fixture(&fixture_root)?
            }
            "profile-divergence-authored-copper" => {
                seed_curated_route_strategy_profile_divergence_fixture(&fixture_root)?
            }
            "no-proposal-null-net-class" => {
                seed_curated_route_strategy_no_proposal_fixture(&fixture_root)?
            }
            "via-available" => seed_curated_route_strategy_via_fixture(&fixture_root)?,
            _ => unreachable!("unsupported curated route-strategy fixture"),
        };
        requests.push(RouteStrategyBatchRequestInput {
            request_id: spec.request_id.to_string(),
            fixture_id: spec.fixture_id.to_string(),
            project_root: fixture_root.clone(),
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        });
        fixtures.push(NativeProjectRouteStrategyCuratedFixtureSuiteEntryView {
            request_id: spec.request_id.to_string(),
            fixture_id: spec.fixture_id.to_string(),
            project_root: fixture_root.display().to_string(),
            net_uuid: net_uuid.to_string(),
            from_anchor_pad_uuid: from_anchor_pad_uuid.to_string(),
            to_anchor_pad_uuid: to_anchor_pad_uuid.to_string(),
            coverage_labels: spec
                .coverage_labels
                .iter()
                .map(|label| label.to_string())
                .collect(),
        });
    }

    write_route_strategy_batch_requests_manifest(&manifest_path, &requests)?;
    Ok(NativeProjectRouteStrategyCuratedFixtureSuiteView {
        action: "write_route_strategy_curated_fixture_suite".to_string(),
        suite_id: ROUTE_STRATEGY_CURATED_FIXTURE_SUITE_ID.to_string(),
        out_dir: out_dir.display().to_string(),
        requests_manifest_path: manifest_path.display().to_string(),
        requests_manifest_kind: ROUTE_STRATEGY_BATCH_REQUESTS_KIND.to_string(),
        requests_manifest_version: ROUTE_STRATEGY_BATCH_REQUESTS_VERSION,
        total_fixtures: fixtures.len(),
        total_requests: fixtures.len(),
        fixtures,
        next_step_command: format!(
            "project route-strategy-batch-evaluate --requests {}",
            manifest_path.display()
        ),
    })
}

pub(crate) fn capture_route_strategy_curated_baseline(
    out_dir: &Path,
    manifest_path_override: Option<&Path>,
    result_path_override: Option<&Path>,
) -> Result<NativeProjectRouteStrategyCuratedBaselineCaptureView> {
    let suite = write_route_strategy_curated_fixture_suite(out_dir, manifest_path_override)?;
    let result_artifact_path = result_path_override
        .map(PathBuf::from)
        .unwrap_or_else(|| out_dir.join("route-strategy-batch-result.json"));
    let report =
        evaluate_native_project_route_strategy_batch(Path::new(&suite.requests_manifest_path))?;
    write_canonical_json(&result_artifact_path, &report)?;
    Ok(NativeProjectRouteStrategyCuratedBaselineCaptureView {
        action: "capture_route_strategy_curated_baseline".to_string(),
        suite_id: suite.suite_id,
        out_dir: suite.out_dir,
        requests_manifest_path: suite.requests_manifest_path.clone(),
        result_artifact_path: result_artifact_path.display().to_string(),
        requests_manifest_kind: suite.requests_manifest_kind,
        requests_manifest_version: suite.requests_manifest_version,
        result_kind: report.kind.clone(),
        result_version: report.version,
        total_fixtures: suite.total_fixtures,
        total_requests: report.summary.total_evaluated_requests,
        summary: report.summary,
        next_inspect_command: format!(
            "project inspect-route-strategy-batch-result {}",
            result_artifact_path.display()
        ),
        next_gate_example_command: format!(
            "project gate-route-strategy-batch-result {} {} --policy strict_identical",
            result_artifact_path.display(),
            result_artifact_path.display()
        ),
    })
}

pub(crate) fn evaluate_native_project_route_strategy_batch(
    requests_manifest_path: &Path,
) -> Result<NativeProjectRouteStrategyBatchEvaluateView> {
    let manifest = load_route_strategy_batch_requests_manifest(requests_manifest_path)?;
    let mut results = Vec::with_capacity(manifest.requests.len());
    let mut recommendation_counts_by_profile = BTreeMap::new();
    let mut delta_classification_counts = BTreeMap::new();
    let mut same_outcome_count = 0usize;
    let mut different_outcome_count = 0usize;
    let mut proposal_available_count = 0usize;
    let mut no_proposal_count = 0usize;

    for request in manifest.requests {
        let strategy_compare = compare_native_project_route_strategy(
            &request.project_root,
            request.net_uuid,
            request.from_anchor_pad_uuid,
            request.to_anchor_pad_uuid,
        )?;
        let recommended_profile_arg =
            route_proposal_profile_from_name(&strategy_compare.recommended_profile)?;
        let strategy_report = report_native_project_route_strategy(
            &request.project_root,
            request.net_uuid,
            request.from_anchor_pad_uuid,
            request.to_anchor_pad_uuid,
            recommended_profile_arg,
        )?;
        let strategy_delta = report_native_project_route_strategy_delta(
            &request.project_root,
            request.net_uuid,
            request.from_anchor_pad_uuid,
            request.to_anchor_pad_uuid,
        )?;

        let recommended_profile = strategy_compare.recommended_profile.clone();
        *recommendation_counts_by_profile
            .entry(recommended_profile.clone())
            .or_insert(0) += 1;
        *delta_classification_counts
            .entry(strategy_delta.delta_classification.clone())
            .or_insert(0) += 1;
        if strategy_delta.outcomes_match {
            same_outcome_count += 1;
        } else {
            different_outcome_count += 1;
        }
        if strategy_compare
            .entries
            .iter()
            .any(|entry| entry.proposal_available)
        {
            proposal_available_count += 1;
        } else {
            no_proposal_count += 1;
        }

        results.push(NativeProjectRouteStrategyBatchEntryView {
            identity: NativeProjectRouteStrategyBatchRequestIdentityView {
                request_id: request.request_id,
                fixture_id: request.fixture_id,
                project_root: request.project_root.display().to_string(),
                net_uuid: request.net_uuid.to_string(),
                from_anchor_pad_uuid: request.from_anchor_pad_uuid.to_string(),
                to_anchor_pad_uuid: request.to_anchor_pad_uuid.to_string(),
            },
            route_strategy_report: strategy_report,
            route_strategy_compare: strategy_compare,
            route_strategy_delta: strategy_delta.clone(),
            recommended_profile,
            delta_classification: strategy_delta.delta_classification,
            outcomes_match: strategy_delta.outcomes_match,
        });
    }

    Ok(NativeProjectRouteStrategyBatchEvaluateView {
        action: "route_strategy_batch_evaluate".to_string(),
        kind: ROUTE_STRATEGY_BATCH_RESULT_KIND.to_string(),
        version: ROUTE_STRATEGY_BATCH_RESULT_VERSION,
        requests_manifest_path: requests_manifest_path.display().to_string(),
        requests_manifest_kind: ROUTE_STRATEGY_BATCH_REQUESTS_KIND.to_string(),
        requests_manifest_version: ROUTE_STRATEGY_BATCH_REQUESTS_VERSION,
        summary: NativeProjectRouteStrategyBatchSummaryView {
            total_evaluated_requests: results.len(),
            recommendation_counts_by_profile,
            delta_classification_counts,
            same_outcome_count,
            different_outcome_count,
            proposal_available_count,
            no_proposal_count,
        },
        results,
    })
}

fn load_route_strategy_batch_requests_manifest(
    requests_manifest_path: &Path,
) -> Result<RouteStrategyBatchRequestsManifest> {
    let raw = std::fs::read_to_string(requests_manifest_path).with_context(|| {
        format!(
            "failed to read route strategy batch requests manifest {}",
            requests_manifest_path.display()
        )
    })?;
    let manifest: RouteStrategyBatchRequestsManifest =
        serde_json::from_str(&raw).with_context(|| {
            format!(
                "failed to parse route strategy batch requests manifest {}",
                requests_manifest_path.display()
            )
        })?;
    if manifest.kind != ROUTE_STRATEGY_BATCH_REQUESTS_KIND {
        anyhow::bail!(
            "route strategy batch requests manifest {} has unsupported kind {}; expected {}",
            requests_manifest_path.display(),
            manifest.kind,
            ROUTE_STRATEGY_BATCH_REQUESTS_KIND
        );
    }
    if manifest.version != ROUTE_STRATEGY_BATCH_REQUESTS_VERSION {
        anyhow::bail!(
            "route strategy batch requests manifest {} has unsupported version {}; expected {}",
            requests_manifest_path.display(),
            manifest.version,
            ROUTE_STRATEGY_BATCH_REQUESTS_VERSION
        );
    }
    if manifest.requests.is_empty() {
        anyhow::bail!(
            "route strategy batch requests manifest {} must contain at least one request",
            requests_manifest_path.display()
        );
    }
    Ok(manifest)
}

fn load_route_strategy_batch_result_artifact(
    artifact_path: &Path,
) -> Result<LoadedRouteStrategyBatchResultArtifact> {
    let raw = std::fs::read_to_string(artifact_path).with_context(|| {
        format!(
            "failed to read route strategy batch result artifact {}",
            artifact_path.display()
        )
    })?;
    let value: Value = serde_json::from_str(&raw).with_context(|| {
        format!(
            "failed to parse route strategy batch result artifact {}",
            artifact_path.display()
        )
    })?;
    let version = value
        .get("version")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "route strategy batch result artifact {} is missing required numeric version",
                artifact_path.display()
            )
        })? as u32;
    match version {
        ROUTE_STRATEGY_BATCH_RESULT_VERSION => {
            let artifact =
                serde_json::from_value::<NativeProjectRouteStrategyBatchEvaluateView>(value)
                    .with_context(|| {
                        format!(
                            "failed to parse route strategy batch result artifact {}",
                            artifact_path.display()
                        )
                    })?;
            if artifact.kind != ROUTE_STRATEGY_BATCH_RESULT_KIND {
                bail!(
                    "unsupported route strategy batch result artifact kind '{}' in {}",
                    artifact.kind,
                    artifact_path.display()
                );
            }
            Ok(LoadedRouteStrategyBatchResultArtifact {
                artifact_path: artifact_path.to_path_buf(),
                source_version: ROUTE_STRATEGY_BATCH_RESULT_VERSION,
                artifact,
            })
        }
        _ => bail!(
            "unsupported route strategy batch result artifact version {} in {}",
            version,
            artifact_path.display()
        ),
    }
}

fn batch_result_required_fields_missing(value: &Value) -> Vec<String> {
    let mut missing = Vec::new();
    for field in [
        "action",
        "kind",
        "version",
        "requests_manifest_path",
        "requests_manifest_kind",
        "requests_manifest_version",
        "summary",
        "results",
    ] {
        if value.get(field).is_none() {
            missing.push(field.to_string());
        }
    }
    missing
}

fn batch_result_artifact_identity_from_value(
    artifact_path: &Path,
    value: &Value,
) -> NativeProjectRouteStrategyBatchResultComparisonArtifactView {
    NativeProjectRouteStrategyBatchResultComparisonArtifactView {
        artifact_path: artifact_path.display().to_string(),
        kind: value
            .get("kind")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        version: value
            .get("version")
            .and_then(Value::as_u64)
            .map(|v| v as u32),
        requests_manifest_kind: value
            .get("requests_manifest_kind")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        requests_manifest_version: value
            .get("requests_manifest_version")
            .and_then(Value::as_u64)
            .map(|v| v as u32),
    }
}

fn batch_result_count_delta(
    before: usize,
    after: usize,
) -> NativeProjectRouteStrategyBatchResultComparisonCountDeltaView {
    NativeProjectRouteStrategyBatchResultComparisonCountDeltaView {
        before,
        after,
        change: after as isize - before as isize,
    }
}

fn collect_route_strategy_batch_result_artifact_paths(
    dir: Option<&Path>,
    artifacts: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if let Some(dir) = dir {
        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("failed to read artifact directory {}", dir.display()))?
        {
            let path = entry?.path();
            if path.is_file() {
                paths.push(path);
            }
        }
    } else {
        paths.extend(artifacts.iter().cloned());
    }
    if paths.is_empty() {
        bail!("route strategy batch result summary requires --dir or at least one --artifact");
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn file_modified_unix_seconds(path: &Path) -> Option<i64> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let seconds = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    i64::try_from(seconds).ok()
}

fn validate_route_strategy_batch_result_entry(
    index: usize,
    value: &Value,
) -> Option<NativeProjectRouteStrategyBatchResultMalformedEntryView> {
    let mut issues = Vec::new();
    let request_id = value
        .get("identity")
        .and_then(|identity| identity.get("request_id"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    if value.get("identity").is_none() {
        issues.push("missing identity".to_string());
    } else {
        for field in [
            "request_id",
            "fixture_id",
            "project_root",
            "net_uuid",
            "from_anchor_pad_uuid",
            "to_anchor_pad_uuid",
        ] {
            if value
                .get("identity")
                .and_then(|identity| identity.get(field))
                .is_none()
            {
                issues.push(format!("missing identity.{field}"));
            }
        }
    }
    for field in [
        "route_strategy_report",
        "route_strategy_compare",
        "route_strategy_delta",
        "recommended_profile",
        "delta_classification",
        "outcomes_match",
    ] {
        if value.get(field).is_none() {
            issues.push(format!("missing {field}"));
        }
    }
    if issues.is_empty() {
        None
    } else {
        Some(NativeProjectRouteStrategyBatchResultMalformedEntryView {
            result_index: index,
            request_id,
            issues,
        })
    }
}

fn inspect_route_strategy_batch_result_value(
    artifact_path: &Path,
    value: &Value,
) -> Result<NativeProjectRouteStrategyBatchResultInspectionView> {
    let loaded = load_route_strategy_batch_result_artifact(artifact_path)?;
    let malformed_entries = value
        .get("results")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .enumerate()
                .filter_map(|(index, entry)| {
                    validate_route_strategy_batch_result_entry(index, entry)
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(NativeProjectRouteStrategyBatchResultInspectionView {
        action: "inspect_route_strategy_batch_result".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        kind: loaded.artifact.kind,
        source_version: loaded.source_version,
        version: loaded.artifact.version,
        requests_manifest_kind: loaded.artifact.requests_manifest_kind,
        requests_manifest_version: loaded.artifact.requests_manifest_version,
        summary: loaded.artifact.summary,
        results: loaded.artifact.results,
        malformed_entries,
    })
}

pub(crate) fn inspect_route_strategy_batch_result(
    artifact_path: &Path,
) -> Result<NativeProjectRouteStrategyBatchResultInspectionView> {
    let raw = std::fs::read_to_string(artifact_path).with_context(|| {
        format!(
            "failed to read route strategy batch result artifact {}",
            artifact_path.display()
        )
    })?;
    let value: Value = serde_json::from_str(&raw).with_context(|| {
        format!(
            "failed to parse route strategy batch result artifact {}",
            artifact_path.display()
        )
    })?;
    inspect_route_strategy_batch_result_value(artifact_path, &value)
}

pub(crate) fn validate_route_strategy_batch_result(
    artifact_path: &Path,
) -> Result<NativeProjectRouteStrategyBatchResultValidationView> {
    let raw = std::fs::read_to_string(artifact_path).with_context(|| {
        format!(
            "failed to read route strategy batch result artifact {}",
            artifact_path.display()
        )
    })?;
    let value: Value = serde_json::from_str(&raw).with_context(|| {
        format!(
            "failed to parse route strategy batch result artifact {}",
            artifact_path.display()
        )
    })?;
    let missing_required_fields = batch_result_required_fields_missing(&value);
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let version = value
        .get("version")
        .and_then(Value::as_u64)
        .map(|v| v as u32);
    let version_compatible = kind.as_deref() == Some(ROUTE_STRATEGY_BATCH_RESULT_KIND)
        && version == Some(ROUTE_STRATEGY_BATCH_RESULT_VERSION);
    let malformed_entries = value
        .get("results")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .enumerate()
                .filter_map(|(index, entry)| {
                    validate_route_strategy_batch_result_entry(index, entry)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let parsed_artifact = if version_compatible && missing_required_fields.is_empty() {
        serde_json::from_value::<NativeProjectRouteStrategyBatchEvaluateView>(value.clone()).ok()
    } else {
        None
    };

    let (
        request_result_count_matches_summary,
        recommendation_counts_match_summary,
        delta_classification_counts_match_summary,
        outcome_counts_match_summary,
        proposal_counts_match_summary,
    ) = if let Some(report) = parsed_artifact.as_ref() {
        let total = report.results.len();
        let recommendation_total: usize = report
            .summary
            .recommendation_counts_by_profile
            .values()
            .sum();
        let delta_total: usize = report.summary.delta_classification_counts.values().sum();
        (
            report.summary.total_evaluated_requests == total,
            recommendation_total == total,
            delta_total == total,
            report.summary.same_outcome_count + report.summary.different_outcome_count == total,
            report.summary.proposal_available_count + report.summary.no_proposal_count == total,
        )
    } else {
        (false, false, false, false, false)
    };
    let structurally_valid = version_compatible
        && missing_required_fields.is_empty()
        && malformed_entries.is_empty()
        && request_result_count_matches_summary
        && recommendation_counts_match_summary
        && delta_classification_counts_match_summary
        && outcome_counts_match_summary
        && proposal_counts_match_summary;

    Ok(NativeProjectRouteStrategyBatchResultValidationView {
        action: "validate_route_strategy_batch_result".to_string(),
        artifact_path: artifact_path.display().to_string(),
        kind,
        source_version: version.filter(|value| *value == ROUTE_STRATEGY_BATCH_RESULT_VERSION),
        version,
        structurally_valid,
        version_compatible,
        missing_required_fields,
        request_result_count_matches_summary,
        recommendation_counts_match_summary,
        delta_classification_counts_match_summary,
        outcome_counts_match_summary,
        proposal_counts_match_summary,
        malformed_entries,
    })
}

pub(crate) fn compare_route_strategy_batch_result(
    before_artifact_path: &Path,
    after_artifact_path: &Path,
) -> Result<NativeProjectRouteStrategyBatchResultComparisonView> {
    let before_raw = std::fs::read_to_string(before_artifact_path).with_context(|| {
        format!(
            "failed to read route strategy batch result artifact {}",
            before_artifact_path.display()
        )
    })?;
    let after_raw = std::fs::read_to_string(after_artifact_path).with_context(|| {
        format!(
            "failed to read route strategy batch result artifact {}",
            after_artifact_path.display()
        )
    })?;
    let before_value: Value = serde_json::from_str(&before_raw).with_context(|| {
        format!(
            "failed to parse route strategy batch result artifact {}",
            before_artifact_path.display()
        )
    })?;
    let after_value: Value = serde_json::from_str(&after_raw).with_context(|| {
        format!(
            "failed to parse route strategy batch result artifact {}",
            after_artifact_path.display()
        )
    })?;

    let before_artifact =
        batch_result_artifact_identity_from_value(before_artifact_path, &before_value);
    let after_artifact =
        batch_result_artifact_identity_from_value(after_artifact_path, &after_value);
    let compatibility_rule = "artifacts are compatible only when both use kind native_route_strategy_batch_result_artifact, version 1, and the same requests manifest kind/version".to_string();
    let compatible_artifacts = before_artifact.kind.as_deref()
        == Some(ROUTE_STRATEGY_BATCH_RESULT_KIND)
        && after_artifact.kind.as_deref() == Some(ROUTE_STRATEGY_BATCH_RESULT_KIND)
        && before_artifact.version == Some(ROUTE_STRATEGY_BATCH_RESULT_VERSION)
        && after_artifact.version == Some(ROUTE_STRATEGY_BATCH_RESULT_VERSION)
        && before_artifact.requests_manifest_kind == after_artifact.requests_manifest_kind
        && before_artifact.requests_manifest_version == after_artifact.requests_manifest_version;

    if !compatible_artifacts {
        return Ok(NativeProjectRouteStrategyBatchResultComparisonView {
            action: "compare_route_strategy_batch_result".to_string(),
            comparison_classification: "incompatible_artifacts".to_string(),
            compatibility_rule,
            compatible_artifacts,
            before_artifact,
            after_artifact,
            total_request_count_change: batch_result_count_delta(0, 0),
            recommendation_distribution_changes: BTreeMap::new(),
            delta_classification_distribution_changes: BTreeMap::new(),
            same_outcome_count_change: batch_result_count_delta(0, 0),
            different_outcome_count_change: batch_result_count_delta(0, 0),
            proposal_available_count_change: batch_result_count_delta(0, 0),
            no_proposal_count_change: batch_result_count_delta(0, 0),
            added_request_ids: Vec::new(),
            removed_request_ids: Vec::new(),
            common_request_ids: Vec::new(),
            changed_common_requests: Vec::new(),
        });
    }

    let before_loaded = load_route_strategy_batch_result_artifact(before_artifact_path)?;
    let after_loaded = load_route_strategy_batch_result_artifact(after_artifact_path)?;

    let mut recommendation_distribution_changes = BTreeMap::new();
    for profile in before_loaded
        .artifact
        .summary
        .recommendation_counts_by_profile
        .keys()
        .chain(
            after_loaded
                .artifact
                .summary
                .recommendation_counts_by_profile
                .keys(),
        )
    {
        recommendation_distribution_changes
            .entry(profile.clone())
            .or_insert_with(|| {
                batch_result_count_delta(
                    *before_loaded
                        .artifact
                        .summary
                        .recommendation_counts_by_profile
                        .get(profile)
                        .unwrap_or(&0),
                    *after_loaded
                        .artifact
                        .summary
                        .recommendation_counts_by_profile
                        .get(profile)
                        .unwrap_or(&0),
                )
            });
    }

    let mut delta_classification_distribution_changes = BTreeMap::new();
    for classification in before_loaded
        .artifact
        .summary
        .delta_classification_counts
        .keys()
        .chain(
            after_loaded
                .artifact
                .summary
                .delta_classification_counts
                .keys(),
        )
    {
        delta_classification_distribution_changes
            .entry(classification.clone())
            .or_insert_with(|| {
                batch_result_count_delta(
                    *before_loaded
                        .artifact
                        .summary
                        .delta_classification_counts
                        .get(classification)
                        .unwrap_or(&0),
                    *after_loaded
                        .artifact
                        .summary
                        .delta_classification_counts
                        .get(classification)
                        .unwrap_or(&0),
                )
            });
    }

    let before_results: BTreeMap<_, _> = before_loaded
        .artifact
        .results
        .iter()
        .map(|entry| (entry.identity.request_id.clone(), entry))
        .collect();
    let after_results: BTreeMap<_, _> = after_loaded
        .artifact
        .results
        .iter()
        .map(|entry| (entry.identity.request_id.clone(), entry))
        .collect();
    let before_ids: BTreeSet<_> = before_results.keys().cloned().collect();
    let after_ids: BTreeSet<_> = after_results.keys().cloned().collect();
    let added_request_ids: Vec<_> = after_ids.difference(&before_ids).cloned().collect();
    let removed_request_ids: Vec<_> = before_ids.difference(&after_ids).cloned().collect();
    let common_request_ids: Vec<_> = before_ids.intersection(&after_ids).cloned().collect();

    let mut changed_common_requests = Vec::new();
    for request_id in &common_request_ids {
        let before = before_results
            .get(request_id)
            .expect("before common request");
        let after = after_results.get(request_id).expect("after common request");
        let recommendation_changed = before.recommended_profile != after.recommended_profile;
        let delta_classification_changed =
            before.delta_classification != after.delta_classification;
        let before_selected_candidate = before.route_strategy_report.selected_candidate.clone();
        let after_selected_candidate = after.route_strategy_report.selected_candidate.clone();
        let before_selected_policy = before.route_strategy_report.selected_policy.clone();
        let after_selected_policy = after.route_strategy_report.selected_policy.clone();
        let before_selected_contract = before.route_strategy_report.selected_contract.clone();
        let after_selected_contract = after.route_strategy_report.selected_contract.clone();
        let selected_live_outcome_changed = before_selected_candidate != after_selected_candidate
            || before_selected_policy != after_selected_policy
            || before_selected_contract != after_selected_contract;
        if recommendation_changed || delta_classification_changed || selected_live_outcome_changed {
            changed_common_requests.push(
                NativeProjectRouteStrategyBatchResultComparisonRequestChangeView {
                    request_id: request_id.clone(),
                    recommendation_changed,
                    delta_classification_changed,
                    selected_live_outcome_changed,
                    before_recommended_profile: before.recommended_profile.clone(),
                    after_recommended_profile: after.recommended_profile.clone(),
                    before_delta_classification: before.delta_classification.clone(),
                    after_delta_classification: after.delta_classification.clone(),
                    before_selected_candidate,
                    after_selected_candidate,
                    before_selected_policy,
                    after_selected_policy,
                },
            );
        }
    }

    let aggregate_changed = recommendation_distribution_changes
        .values()
        .any(|delta| delta.change != 0)
        || delta_classification_distribution_changes
            .values()
            .any(|delta| delta.change != 0)
        || before_loaded.artifact.summary.total_evaluated_requests
            != after_loaded.artifact.summary.total_evaluated_requests
        || before_loaded.artifact.summary.same_outcome_count
            != after_loaded.artifact.summary.same_outcome_count
        || before_loaded.artifact.summary.different_outcome_count
            != after_loaded.artifact.summary.different_outcome_count
        || before_loaded.artifact.summary.proposal_available_count
            != after_loaded.artifact.summary.proposal_available_count
        || before_loaded.artifact.summary.no_proposal_count
            != after_loaded.artifact.summary.no_proposal_count;
    let comparison_classification = if added_request_ids.is_empty()
        && removed_request_ids.is_empty()
        && changed_common_requests.is_empty()
        && !aggregate_changed
    {
        "identical"
    } else if changed_common_requests.is_empty() {
        "aggregate_only_changed"
    } else {
        "per_request_outcomes_changed"
    }
    .to_string();

    Ok(NativeProjectRouteStrategyBatchResultComparisonView {
        action: "compare_route_strategy_batch_result".to_string(),
        comparison_classification,
        compatibility_rule,
        compatible_artifacts,
        before_artifact,
        after_artifact,
        total_request_count_change: batch_result_count_delta(
            before_loaded.artifact.summary.total_evaluated_requests,
            after_loaded.artifact.summary.total_evaluated_requests,
        ),
        recommendation_distribution_changes,
        delta_classification_distribution_changes,
        same_outcome_count_change: batch_result_count_delta(
            before_loaded.artifact.summary.same_outcome_count,
            after_loaded.artifact.summary.same_outcome_count,
        ),
        different_outcome_count_change: batch_result_count_delta(
            before_loaded.artifact.summary.different_outcome_count,
            after_loaded.artifact.summary.different_outcome_count,
        ),
        proposal_available_count_change: batch_result_count_delta(
            before_loaded.artifact.summary.proposal_available_count,
            after_loaded.artifact.summary.proposal_available_count,
        ),
        no_proposal_count_change: batch_result_count_delta(
            before_loaded.artifact.summary.no_proposal_count,
            after_loaded.artifact.summary.no_proposal_count,
        ),
        added_request_ids,
        removed_request_ids,
        common_request_ids,
        changed_common_requests,
    })
}

fn route_strategy_batch_gate_policy_name(
    policy: NativeProjectRouteStrategyBatchGatePolicyArg,
) -> &'static str {
    match policy {
        NativeProjectRouteStrategyBatchGatePolicyArg::StrictIdentical => "strict_identical",
        NativeProjectRouteStrategyBatchGatePolicyArg::AllowAggregateOnly => "allow_aggregate_only",
        NativeProjectRouteStrategyBatchGatePolicyArg::FailOnRecommendationChange => {
            "fail_on_recommendation_change"
        }
    }
}

pub(crate) fn gate_route_strategy_batch_result(
    before_artifact_path: &Path,
    after_artifact_path: &Path,
    policy: NativeProjectRouteStrategyBatchGatePolicyArg,
) -> Result<NativeProjectRouteStrategyBatchResultGateView> {
    let comparison =
        compare_route_strategy_batch_result(before_artifact_path, after_artifact_path)?;
    let changed_recommendations = comparison
        .changed_common_requests
        .iter()
        .filter(|change| change.recommendation_changed)
        .count();
    let changed_delta_classifications = comparison
        .changed_common_requests
        .iter()
        .filter(|change| change.delta_classification_changed)
        .count();
    let changed_per_request_outcomes = comparison.changed_common_requests.len();
    let mut threshold_facts = BTreeMap::new();
    threshold_facts.insert(
        "changed_recommendations".to_string(),
        changed_recommendations,
    );
    threshold_facts.insert(
        "changed_delta_classifications".to_string(),
        changed_delta_classifications,
    );
    threshold_facts.insert(
        "changed_per_request_outcomes".to_string(),
        changed_per_request_outcomes,
    );
    threshold_facts.insert(
        "added_request_ids".to_string(),
        comparison.added_request_ids.len(),
    );
    threshold_facts.insert(
        "removed_request_ids".to_string(),
        comparison.removed_request_ids.len(),
    );

    let mut pass_fail_reasons = Vec::new();
    let passed = if !comparison.compatible_artifacts {
        pass_fail_reasons.push(
            "failed because the saved artifacts are incompatible under the documented compatibility rule"
                .to_string(),
        );
        false
    } else {
        match policy {
            NativeProjectRouteStrategyBatchGatePolicyArg::StrictIdentical => {
                let ok = comparison.comparison_classification == "identical";
                if ok {
                    pass_fail_reasons
                        .push("passed because the saved artifacts are identical".to_string());
                } else {
                    pass_fail_reasons.push(
                        "failed because strict_identical requires comparison_classification = identical"
                            .to_string(),
                    );
                }
                ok
            }
            NativeProjectRouteStrategyBatchGatePolicyArg::AllowAggregateOnly => {
                let ok = comparison.comparison_classification == "identical"
                    || comparison.comparison_classification == "aggregate_only_changed";
                if ok {
                    pass_fail_reasons.push(
                        "passed because allow_aggregate_only permits identical and aggregate_only_changed results"
                            .to_string(),
                    );
                } else {
                    pass_fail_reasons.push(
                        "failed because allow_aggregate_only rejects per-request outcome changes"
                            .to_string(),
                    );
                }
                ok
            }
            NativeProjectRouteStrategyBatchGatePolicyArg::FailOnRecommendationChange => {
                let ok = changed_recommendations == 0 && comparison.compatible_artifacts;
                if ok {
                    pass_fail_reasons.push(
                        "passed because no common request changed its recommended profile"
                            .to_string(),
                    );
                } else {
                    pass_fail_reasons.push(
                        "failed because at least one common request changed its recommended profile"
                            .to_string(),
                    );
                }
                ok
            }
        }
    };

    Ok(NativeProjectRouteStrategyBatchResultGateView {
        action: "gate_route_strategy_batch_result".to_string(),
        selected_gate_policy: route_strategy_batch_gate_policy_name(policy).to_string(),
        passed,
        comparison_classification: comparison.comparison_classification.clone(),
        pass_fail_reasons,
        threshold_facts,
        changed_recommendations,
        changed_delta_classifications,
        changed_per_request_outcomes,
        comparison,
    })
}

pub(crate) fn summarize_route_strategy_batch_results(
    dir: Option<&Path>,
    artifacts: &[PathBuf],
    baseline: Option<&Path>,
    policy: NativeProjectRouteStrategyBatchGatePolicyArg,
) -> Result<NativeProjectRouteStrategyBatchResultsIndexView> {
    let mut paths = collect_route_strategy_batch_result_artifact_paths(dir, artifacts)?;
    let baseline_path = baseline.map(PathBuf::from);
    if let Some(baseline_path) = &baseline_path
        && !paths.iter().any(|path| path == baseline_path)
    {
        paths.push(baseline_path.clone());
    }

    let ordering_basis = "filesystem_modified_time_then_path".to_string();
    let mut sortable: Vec<_> = paths
        .into_iter()
        .map(|path| {
            let modified = file_modified_unix_seconds(&path);
            (modified.unwrap_or(i64::MIN), path)
        })
        .collect();
    sortable.sort_by(|(left_modified, left_path), (right_modified, right_path)| {
        left_modified
            .cmp(right_modified)
            .then_with(|| left_path.cmp(right_path))
    });

    let mut entries = Vec::new();
    let mut structurally_valid_artifacts = 0usize;
    let mut structurally_invalid_artifacts = 0usize;
    let mut gate_passed_artifacts = 0usize;
    let mut gate_failed_artifacts = 0usize;

    for (index, (modified, path)) in sortable.iter().enumerate() {
        let path_ref = path.as_path();
        let validation = validate_route_strategy_batch_result(path_ref);
        let is_baseline = baseline_path
            .as_ref()
            .map(|value| value == path_ref)
            .unwrap_or(false);
        let (
            kind,
            version,
            requests_manifest_kind,
            requests_manifest_version,
            structurally_valid,
            request_count,
            recommendation_distribution,
            delta_classification_distribution,
            validation_error,
        ) = match validation {
            Ok(validation) => {
                let inspection = inspect_route_strategy_batch_result(path_ref).ok();
                if validation.structurally_valid {
                    structurally_valid_artifacts += 1;
                } else {
                    structurally_invalid_artifacts += 1;
                }
                (
                    validation.kind,
                    validation.version,
                    inspection
                        .as_ref()
                        .map(|i| i.requests_manifest_kind.clone()),
                    inspection.as_ref().map(|i| i.requests_manifest_version),
                    validation.structurally_valid,
                    inspection
                        .as_ref()
                        .map(|inspection| inspection.summary.total_evaluated_requests),
                    inspection.as_ref().map(|inspection| {
                        inspection.summary.recommendation_counts_by_profile.clone()
                    }),
                    inspection
                        .as_ref()
                        .map(|inspection| inspection.summary.delta_classification_counts.clone()),
                    None,
                )
            }
            Err(err) => {
                structurally_invalid_artifacts += 1;
                (
                    None,
                    None,
                    None,
                    None,
                    false,
                    None,
                    None,
                    None,
                    Some(err.to_string()),
                )
            }
        };

        let baseline_gate = if let Some(baseline_path) = &baseline_path {
            if is_baseline {
                None
            } else {
                match gate_route_strategy_batch_result(baseline_path, path_ref, policy) {
                    Ok(gate) => {
                        if gate.passed {
                            gate_passed_artifacts += 1;
                        } else {
                            gate_failed_artifacts += 1;
                        }
                        Some(NativeProjectRouteStrategyBatchResultsIndexGateSummaryView {
                            selected_gate_policy: gate.selected_gate_policy,
                            passed: gate.passed,
                            comparison_classification: gate.comparison_classification,
                            pass_fail_reasons: gate.pass_fail_reasons,
                        })
                    }
                    Err(err) => {
                        gate_failed_artifacts += 1;
                        Some(NativeProjectRouteStrategyBatchResultsIndexGateSummaryView {
                            selected_gate_policy: route_strategy_batch_gate_policy_name(policy)
                                .to_string(),
                            passed: false,
                            comparison_classification: "incompatible_artifacts".to_string(),
                            pass_fail_reasons: vec![err.to_string()],
                        })
                    }
                }
            }
        } else {
            None
        };

        entries.push(NativeProjectRouteStrategyBatchResultsIndexEntryView {
            artifact_path: path_ref.display().to_string(),
            kind,
            version,
            requests_manifest_kind,
            requests_manifest_version,
            file_modified_unix_seconds: if *modified == i64::MIN {
                None
            } else {
                Some(*modified)
            },
            run_order: index + 1,
            structurally_valid,
            request_count,
            recommendation_distribution,
            delta_classification_distribution,
            validation_error,
            is_baseline,
            baseline_gate,
        });
    }

    Ok(NativeProjectRouteStrategyBatchResultsIndexView {
        action: "summarize_route_strategy_batch_results".to_string(),
        ordering_basis,
        baseline_artifact: baseline_path.map(|path| path.display().to_string()),
        selected_gate_policy: baseline
            .map(|_| route_strategy_batch_gate_policy_name(policy).to_string()),
        summary: NativeProjectRouteStrategyBatchResultsIndexSummaryView {
            total_artifacts: entries.len(),
            structurally_valid_artifacts,
            structurally_invalid_artifacts,
            gate_passed_artifacts,
            gate_failed_artifacts,
        },
        artifacts: entries,
    })
}

pub(crate) fn export_selected_native_project_route_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    profile: NativeProjectRouteProposalProfileArg,
    output_path: &Path,
) -> Result<NativeProjectSelectedRouteProposalExportView> {
    let selection = run_native_project_route_proposal_selection(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        profile,
    )?;
    let selected_spec = selection.selected_spec.ok_or_else(|| {
        anyhow::anyhow!(
            "route-proposal found no selectable route under current authored constraints"
        )
    })?;
    let export = export_native_project_route_path_proposal(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selected_spec.candidate,
        selected_spec.policy,
        output_path,
    )?;
    Ok(NativeProjectSelectedRouteProposalExportView {
        action: "export_route_proposal".to_string(),
        project_root: root.display().to_string(),
        selection_profile: route_proposal_profile_name(profile).to_string(),
        selection_rule: selection.report.selection_rule,
        selected_candidate: route_apply_candidate_name(selected_spec.candidate).to_string(),
        selected_policy: selected_spec
            .policy
            .map(route_authored_copper_graph_policy_name),
        artifact_path: export.artifact_path,
        kind: export.kind,
        version: export.version,
        project_uuid: export.project_uuid,
        contract: export.contract,
        actions: export.actions,
        selected_path_bend_count: export.selected_path_bend_count,
        selected_path_point_count: export.selected_path_point_count,
        selected_path_segment_count: export.selected_path_segment_count,
        segment_evidence: export.segment_evidence,
    })
}

pub(crate) fn apply_selected_native_project_route(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    profile: NativeProjectRouteProposalProfileArg,
) -> Result<NativeProjectRouteApplySelectedView> {
    let selection = run_native_project_route_proposal_selection(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        profile,
    )?;
    let selected_spec = selection.selected_spec.ok_or_else(|| {
        anyhow::anyhow!(
            "route-proposal found no selectable route under current authored constraints"
        )
    })?;
    let apply = apply_native_project_route(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selected_spec.candidate,
        selected_spec.policy,
    )?;
    Ok(NativeProjectRouteApplySelectedView {
        action: "route_apply_selected".to_string(),
        project_root: root.display().to_string(),
        selection_profile: route_proposal_profile_name(profile).to_string(),
        selection_rule: selection.report.selection_rule,
        selected_candidate: route_apply_candidate_name(selected_spec.candidate).to_string(),
        selected_policy: selected_spec
            .policy
            .map(route_authored_copper_graph_policy_name),
        contract: apply.contract,
        proposal_actions: apply.proposal_actions,
        applied_actions: apply.applied_actions,
        applied: apply.applied,
    })
}

fn run_native_project_route_proposal_selection(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    profile: NativeProjectRouteProposalProfileArg,
) -> Result<RouteProposalSelectionOutcome> {
    let specs = route_proposal_selection_specs(profile);
    let selection_rule = format!(
        "profile {} selects the first successful candidate in this deterministic order: {}",
        route_proposal_profile_name(profile),
        specs
            .iter()
            .map(route_proposal_selection_spec_name)
            .collect::<Vec<_>>()
            .join(" > ")
    );
    let mut selected_candidate: Option<(usize, RouteProposalSelectionCandidateSpec)> = None;
    let mut candidates = Vec::with_capacity(specs.len());

    for (index, spec) in specs.iter().copied().enumerate() {
        match build_route_path_proposal_actions(
            root,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            spec.candidate,
            spec.policy,
        ) {
            Ok(actions) => {
                super::command_project_route_apply::validate_route_proposal_actions(&actions)?;
                let first_action = actions.first().ok_or_else(|| {
                    anyhow::anyhow!(
                        "route proposal selector candidate {} produced no actions",
                        route_proposal_selection_spec_name(&spec)
                    )
                })?;
                let message = if let Some((winner_index, winner)) = selected_candidate {
                    Some(format!(
                        "available but skipped because earlier candidate {} won at selection index {}",
                        route_proposal_selection_spec_name(&winner),
                        winner_index
                    ))
                } else {
                    selected_candidate = Some((index, spec));
                    None
                };
                candidates.push(NativeProjectRouteProposalSelectionCandidateView {
                    candidate: route_apply_candidate_name(spec.candidate).to_string(),
                    policy: spec.policy.map(route_authored_copper_graph_policy_name),
                    selected: selected_candidate == Some((index, spec)),
                    contract: Some(first_action.contract.clone()),
                    actions: Some(actions.len()),
                    selected_path_bend_count: Some(first_action.selected_path_bend_count),
                    selected_path_point_count: Some(first_action.selected_path_point_count),
                    selected_path_segment_count: Some(first_action.selected_path_segment_count),
                    message,
                });
            }
            Err(error) => {
                candidates.push(NativeProjectRouteProposalSelectionCandidateView {
                    candidate: route_apply_candidate_name(spec.candidate).to_string(),
                    policy: spec.policy.map(route_authored_copper_graph_policy_name),
                    selected: false,
                    contract: None,
                    actions: None,
                    selected_path_bend_count: None,
                    selected_path_point_count: None,
                    selected_path_segment_count: None,
                    message: Some(error.to_string()),
                });
            }
        }
    }

    let selected_view = selected_candidate.and_then(|(winner_index, winner_spec)| {
        candidates
            .get(winner_index)
            .map(|candidate| (winner_spec, candidate.clone()))
    });

    let selected_spec = selected_view.as_ref().map(|(spec, _)| *spec);

    Ok(RouteProposalSelectionOutcome {
        report: NativeProjectRouteProposalSelectionView {
            action: "route_proposal".to_string(),
            project_root: root.display().to_string(),
            net_uuid: net_uuid.to_string(),
            from_anchor_pad_uuid: from_anchor_pad_uuid.to_string(),
            to_anchor_pad_uuid: to_anchor_pad_uuid.to_string(),
            selection_profile: route_proposal_profile_name(profile).to_string(),
            status: if selected_view.is_some() {
                "deterministic_route_proposal_selected".to_string()
            } else {
                "no_route_proposal_under_current_authored_constraints".to_string()
            },
            selection_rule,
            attempted_candidates: candidates.len(),
            selected_candidate: selected_view
                .as_ref()
                .map(|(spec, _)| route_apply_candidate_name(spec.candidate).to_string()),
            selected_policy: selected_view
                .as_ref()
                .and_then(|(spec, _)| spec.policy.map(route_authored_copper_graph_policy_name)),
            selected_contract: selected_view
                .as_ref()
                .and_then(|(_, candidate)| candidate.contract.clone()),
            selected_actions: selected_view
                .as_ref()
                .and_then(|(_, candidate)| candidate.actions),
            selected_path_bend_count: selected_view
                .as_ref()
                .and_then(|(_, candidate)| candidate.selected_path_bend_count),
            selected_path_point_count: selected_view
                .as_ref()
                .and_then(|(_, candidate)| candidate.selected_path_point_count),
            selected_path_segment_count: selected_view
                .as_ref()
                .and_then(|(_, candidate)| candidate.selected_path_segment_count),
            candidates,
        },
        selected_spec,
    })
}

fn route_proposal_selection_specs(
    profile: NativeProjectRouteProposalProfileArg,
) -> Vec<RouteProposalSelectionCandidateSpec> {
    match profile {
        NativeProjectRouteProposalProfileArg::Default => default_route_proposal_selection_specs(),
        NativeProjectRouteProposalProfileArg::AuthoredCopperPriority => {
            authored_copper_priority_route_proposal_selection_specs()
        }
    }
}

fn accepted_route_strategy_profiles() -> [NativeProjectRouteProposalProfileArg; 2] {
    [
        NativeProjectRouteProposalProfileArg::Default,
        NativeProjectRouteProposalProfileArg::AuthoredCopperPriority,
    ]
}

fn default_route_proposal_selection_specs() -> Vec<RouteProposalSelectionCandidateSpec> {
    vec![
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidate,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalDogleg,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalTwoBend,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraph,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::AuthoredCopperPlusOneGap,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateTwoVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateThreeVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateFourVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateFiveVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateSixVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateAuthoredViaChain,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphTwoVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate:
                NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphThreeVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate:
                NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFourVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate:
                NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFiveVia,
            policy: None,
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphSixVia,
            policy: None,
        },
    ]
}

fn authored_copper_priority_route_proposal_selection_specs()
-> Vec<RouteProposalSelectionCandidateSpec> {
    let mut specs = vec![
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            policy: Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::Plain),
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            policy: Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware),
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            policy: Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware),
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            policy: Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware),
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            policy: Some(
                NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware,
            ),
        },
        RouteProposalSelectionCandidateSpec {
            candidate: NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            policy: Some(
                NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware,
            ),
        },
    ];
    specs.extend(default_route_proposal_selection_specs());
    specs
}

fn route_proposal_profile_name(profile: NativeProjectRouteProposalProfileArg) -> &'static str {
    match profile {
        NativeProjectRouteProposalProfileArg::Default => "default",
        NativeProjectRouteProposalProfileArg::AuthoredCopperPriority => "authored-copper-priority",
    }
}

fn route_proposal_profile_from_name(name: &str) -> Result<NativeProjectRouteProposalProfileArg> {
    match name {
        "default" => Ok(NativeProjectRouteProposalProfileArg::Default),
        "authored-copper-priority" => {
            Ok(NativeProjectRouteProposalProfileArg::AuthoredCopperPriority)
        }
        _ => anyhow::bail!("unsupported route proposal profile name {name}"),
    }
}

fn route_strategy_profile_distinction(
    profile: NativeProjectRouteProposalProfileArg,
) -> &'static str {
    match profile {
        NativeProjectRouteProposalProfileArg::Default => {
            "baseline profile: preserves the accepted selector family order exactly"
        }
        NativeProjectRouteProposalProfileArg::AuthoredCopperPriority => {
            "reuse-priority profile: prepends the accepted authored-copper-graph policy family ahead of the unchanged default order"
        }
    }
}

fn route_proposal_selection_spec_name(spec: &RouteProposalSelectionCandidateSpec) -> String {
    if let Some(policy) = spec.policy {
        format!(
            "{}:{}",
            route_apply_candidate_name(spec.candidate),
            route_authored_copper_graph_policy_name(policy)
        )
    } else {
        route_apply_candidate_name(spec.candidate).to_string()
    }
}

fn route_authored_copper_graph_policy_name(
    policy: NativeRoutePathCandidateAuthoredCopperGraphPolicy,
) -> String {
    match policy {
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::Plain => "plain".to_string(),
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware => "zone_aware".to_string(),
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware => {
            "obstacle_aware".to_string()
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware => {
            "zone_obstacle_aware".to_string()
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware => {
            "zone_obstacle_topology_aware".to_string()
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware => {
            "zone_obstacle_topology_layer_balance_aware".to_string()
        }
    }
}

fn route_apply_candidate_name(candidate: NativeProjectRouteApplyCandidateArg) -> &'static str {
    match candidate {
        NativeProjectRouteApplyCandidateArg::RoutePathCandidate => "route-path-candidate",
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateVia => "route-path-candidate-via",
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateTwoVia => {
            "route-path-candidate-two-via"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateThreeVia => {
            "route-path-candidate-three-via"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFourVia => {
            "route-path-candidate-four-via"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFiveVia => {
            "route-path-candidate-five-via"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateSixVia => {
            "route-path-candidate-six-via"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateAuthoredViaChain => {
            "route-path-candidate-authored-via-chain"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalDogleg => {
            "route-path-candidate-orthogonal-dogleg"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalTwoBend => {
            "route-path-candidate-orthogonal-two-bend"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraph => {
            "route-path-candidate-orthogonal-graph"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphVia => {
            "route-path-candidate-orthogonal-graph-via"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphTwoVia => {
            "route-path-candidate-orthogonal-graph-two-via"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphThreeVia => {
            "route-path-candidate-orthogonal-graph-three-via"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFourVia => {
            "route-path-candidate-orthogonal-graph-four-via"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFiveVia => {
            "route-path-candidate-orthogonal-graph-five-via"
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphSixVia => {
            "route-path-candidate-orthogonal-graph-six-via"
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperPlusOneGap => {
            "authored-copper-plus-one-gap"
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph => "authored-copper-graph",
    }
}

pub(crate) fn load_route_proposal_artifact(
    artifact_path: &Path,
) -> Result<LoadedRouteProposalArtifact> {
    let contents = std::fs::read_to_string(artifact_path).with_context(|| {
        format!(
            "failed to read route proposal artifact {}",
            artifact_path.display()
        )
    })?;
    let value = serde_json::from_str::<serde_json::Value>(&contents).with_context(|| {
        format!(
            "failed to parse route proposal artifact {}",
            artifact_path.display()
        )
    })?;

    let kind = value.get("kind").and_then(serde_json::Value::as_str);
    if let Some(kind) = kind
        && kind != ROUTE_PROPOSAL_ARTIFACT_KIND
    {
        bail!(
            "unsupported route proposal artifact kind '{}' in {}",
            kind,
            artifact_path.display()
        );
    }

    let version = match value.get("version") {
        Some(version) => {
            let raw = version.as_u64().ok_or_else(|| {
                anyhow::Error::msg(format!(
                    "invalid route proposal artifact version in {}",
                    artifact_path.display()
                ))
            })?;
            u32::try_from(raw).map_err(|_| {
                anyhow::Error::msg(format!(
                    "invalid route proposal artifact version in {}",
                    artifact_path.display()
                ))
            })?
        }
        None => 0,
    };

    match version {
        ROUTE_PROPOSAL_ARTIFACT_VERSION => {
            let artifact =
                serde_json::from_value::<RouteProposalArtifact>(value).with_context(|| {
                    format!(
                        "failed to parse route proposal artifact {}",
                        artifact_path.display()
                    )
                })?;
            if artifact.kind != ROUTE_PROPOSAL_ARTIFACT_KIND {
                bail!(
                    "unsupported route proposal artifact kind '{}' in {}",
                    artifact.kind,
                    artifact_path.display()
                );
            }
            Ok(LoadedRouteProposalArtifact {
                artifact_path: artifact_path.to_path_buf(),
                source_version: ROUTE_PROPOSAL_ARTIFACT_VERSION,
                artifact,
            })
        }
        _ => bail!(
            "unsupported route proposal artifact version {} in {}",
            version,
            artifact_path.display()
        ),
    }
}

pub(crate) fn inspect_route_proposal_artifact(
    artifact_path: &Path,
) -> Result<NativeProjectRouteProposalArtifactInspectionView> {
    let loaded = load_route_proposal_artifact(artifact_path)?;
    let segment_evidence = orthogonal_graph_route_proposal_artifact_inspection_segment_evidence(
        &loaded.artifact.actions,
    );
    Ok(NativeProjectRouteProposalArtifactInspectionView {
        artifact_path: loaded.artifact_path.display().to_string(),
        kind: loaded.artifact.kind,
        source_version: loaded.source_version,
        version: loaded.artifact.version,
        migration_applied: false,
        project_uuid: loaded.artifact.project_uuid.to_string(),
        project_name: loaded.artifact.project_name,
        contract: loaded.artifact.contract,
        actions: loaded.artifact.actions.len(),
        draw_track_actions: loaded
            .artifact
            .actions
            .iter()
            .filter(|action| action.proposal_action == "draw_track")
            .count(),
        selected_path_bend_count: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.selected_path_bend_count)
            .unwrap_or(0),
        selected_path_point_count: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.selected_path_point_count)
            .unwrap_or(0),
        selected_path_segment_count: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.selected_path_segment_count)
            .unwrap_or(0),
        segment_evidence,
    })
}

fn review_route_proposal_artifact(
    artifact_path: &Path,
) -> Result<NativeProjectRouteProposalReviewView> {
    let loaded = load_route_proposal_artifact(artifact_path)?;
    let segment_evidence = orthogonal_graph_route_proposal_artifact_inspection_segment_evidence(
        &loaded.artifact.actions,
    );
    Ok(NativeProjectRouteProposalReviewView {
        action: "review_route_proposal".to_string(),
        review_source: "route_proposal_artifact".to_string(),
        status: "saved_route_proposal_artifact_ready".to_string(),
        explanation:
            "reviewing one saved route proposal artifact without consulting live project state"
                .to_string(),
        project_root: None,
        artifact_path: Some(loaded.artifact_path.display().to_string()),
        kind: Some(loaded.artifact.kind.clone()),
        source_version: Some(loaded.source_version),
        version: Some(loaded.artifact.version),
        project_uuid: Some(loaded.artifact.project_uuid.to_string()),
        project_name: Some(loaded.artifact.project_name.clone()),
        net_uuid: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.net_uuid.to_string()),
        from_anchor_pad_uuid: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.from_anchor_pad_uuid.to_string()),
        to_anchor_pad_uuid: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.to_anchor_pad_uuid.to_string()),
        selection_profile: None,
        selection_rule: None,
        selected_candidate: None,
        selected_policy: None,
        contract: loaded.artifact.contract.clone(),
        actions: loaded.artifact.actions.len(),
        draw_track_actions: loaded
            .artifact
            .actions
            .iter()
            .filter(|action| action.proposal_action == "draw_track")
            .count(),
        selected_path_bend_count: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.selected_path_bend_count)
            .unwrap_or(0),
        selected_path_point_count: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.selected_path_point_count)
            .unwrap_or(0),
        selected_path_segment_count: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.selected_path_segment_count)
            .unwrap_or(0),
        segment_evidence,
        proposal_actions: loaded.artifact.actions,
    })
}

pub(crate) fn apply_route_proposal_artifact(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectRouteProposalArtifactApplyView> {
    let loaded = load_route_proposal_artifact(artifact_path)?;
    super::command_project_route_apply::validate_route_proposal_actions(&loaded.artifact.actions)?;

    let first_action = loaded.artifact.actions.first().ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal artifact {} must contain at least one action",
            loaded.artifact_path.display()
        )
    })?;
    let revalidation = analyze_route_proposal_artifact_revalidation(
        root,
        &loaded.artifact.contract,
        first_action,
        &loaded.artifact.actions,
    );
    if let Some(diagnostic) = &revalidation.drift_message {
        bail!(
            "route proposal artifact drifted for contract {}: {}; refresh the proposal before apply",
            loaded.artifact.contract,
            diagnostic
        );
    }
    let _live_actions = revalidation.live_actions?;
    if !revalidation.matches_live {
        bail!(
            "route proposal artifact drifted for contract {}: geometry changed under the same ranked path; refresh the proposal before apply",
            loaded.artifact.contract
        );
    }

    let applied = super::command_project_route_apply::apply_route_proposal_actions(
        root,
        &loaded.artifact.actions,
    )?;

    Ok(NativeProjectRouteProposalArtifactApplyView {
        action: "apply_route_proposal_artifact".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: root.display().to_string(),
        artifact_actions: loaded.artifact.actions.len(),
        applied_actions: applied.len(),
        selected_path_bend_count: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.selected_path_bend_count)
            .unwrap_or(0),
        selected_path_point_count: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.selected_path_point_count)
            .unwrap_or(0),
        selected_path_segment_count: loaded
            .artifact
            .actions
            .first()
            .map(|action| action.selected_path_segment_count)
            .unwrap_or(0),
        applied,
    })
}

pub(crate) fn revalidate_route_proposal_artifact(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectRouteProposalArtifactRevalidationView> {
    let loaded = load_route_proposal_artifact(artifact_path)?;
    let first_action = loaded.artifact.actions.first().ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal artifact {} must contain at least one action",
            loaded.artifact_path.display()
        )
    })?;
    let revalidation = analyze_route_proposal_artifact_revalidation(
        root,
        &loaded.artifact.contract,
        first_action,
        &loaded.artifact.actions,
    );
    let artifact_selected_path_bend_count = loaded
        .artifact
        .actions
        .first()
        .map(|action| action.selected_path_bend_count)
        .unwrap_or(0);
    let artifact_selected_path_point_count = loaded
        .artifact
        .actions
        .first()
        .map(|action| action.selected_path_point_count)
        .unwrap_or(0);
    let artifact_selected_path_segment_count = loaded
        .artifact
        .actions
        .first()
        .map(|action| action.selected_path_segment_count)
        .unwrap_or(0);
    let live_actions_len = revalidation.live_actions.as_ref().ok().map(Vec::len);
    let live_selected_path_bend_count =
        revalidation.live_actions.as_ref().ok().and_then(|actions| {
            actions
                .first()
                .map(|action| action.selected_path_bend_count)
        });
    let live_selected_path_point_count =
        revalidation.live_actions.as_ref().ok().and_then(|actions| {
            actions
                .first()
                .map(|action| action.selected_path_point_count)
        });
    let live_selected_path_segment_count =
        revalidation.live_actions.as_ref().ok().and_then(|actions| {
            actions
                .first()
                .map(|action| action.selected_path_segment_count)
        });
    let segment_evidence = orthogonal_graph_route_proposal_artifact_segment_evidence(
        &loaded.artifact.actions,
        revalidation.live_actions.as_ref().ok().map(Vec::as_slice),
    );
    let live_rebuild_error = revalidation
        .live_actions
        .as_ref()
        .err()
        .map(|error| error.to_string());

    Ok(NativeProjectRouteProposalArtifactRevalidationView {
        action: "revalidate_route_proposal_artifact".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: root.display().to_string(),
        contract: loaded.artifact.contract.clone(),
        artifact_actions: loaded.artifact.actions.len(),
        live_actions: live_actions_len,
        matches_live: revalidation.matches_live,
        drift_kind: revalidation
            .drift_kind
            .map(|kind| kind.as_str().to_string()),
        drift_message: revalidation.drift_message,
        live_rebuild_error,
        selected_path_bend_count: artifact_selected_path_bend_count,
        selected_path_point_count: artifact_selected_path_point_count,
        selected_path_segment_count: artifact_selected_path_segment_count,
        live_selected_path_bend_count,
        live_selected_path_point_count,
        live_selected_path_segment_count,
        segment_evidence,
    })
}

fn rebuild_route_proposal_artifact_live_actions(
    root: &Path,
    contract: &str,
    first_action: &NativeProjectRouteProposalActionView,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    if contract == "m5_route_path_candidate_authored_copper_graph_policy_v1" {
        let policy =
            route_path_candidate_authored_copper_graph_policy_from_reason(&first_action.reason)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "route proposal artifact apply is not supported for contract={} reason={}",
                        contract,
                        first_action.reason
                    )
                })?;
        return build_route_path_candidate_authored_copper_graph_policy_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
            policy,
        );
    }

    match (contract, first_action.reason.as_str()) {
        ("m5_route_path_candidate_v2", ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE) => {
            build_route_path_candidate_proposal_actions(
                root,
                first_action.net_uuid,
                first_action.from_anchor_pad_uuid,
                first_action.to_anchor_pad_uuid,
            )
        }
        ("m5_route_path_candidate_via_v1", ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_VIA) => {
            build_route_path_candidate_via_proposal_actions(
                root,
                first_action.net_uuid,
                first_action.from_anchor_pad_uuid,
                first_action.to_anchor_pad_uuid,
            )
        }
        (
            "m5_route_path_candidate_two_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_TWO_VIA,
        ) => build_route_path_candidate_two_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_three_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_THREE_VIA,
        ) => build_route_path_candidate_three_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_four_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FOUR_VIA,
        ) => build_route_path_candidate_four_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_five_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FIVE_VIA,
        ) => build_route_path_candidate_five_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_six_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_SIX_VIA,
        ) => build_route_path_candidate_six_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_authored_via_chain_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN,
        ) => build_route_path_candidate_authored_via_chain_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_orthogonal_dogleg_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG,
        ) => build_route_path_candidate_orthogonal_dogleg_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_orthogonal_two_bend_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND,
        ) => build_route_path_candidate_orthogonal_two_bend_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_orthogonal_graph_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH,
        ) => build_route_path_candidate_orthogonal_graph_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_orthogonal_graph_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA,
        ) => build_route_path_candidate_orthogonal_graph_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_orthogonal_graph_two_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA,
        ) => build_route_path_candidate_orthogonal_graph_two_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_orthogonal_graph_three_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA,
        ) => build_route_path_candidate_orthogonal_graph_three_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_orthogonal_graph_four_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FOUR_VIA,
        ) => build_route_path_candidate_orthogonal_graph_four_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_orthogonal_graph_five_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FIVE_VIA,
        ) => build_route_path_candidate_orthogonal_graph_five_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_orthogonal_graph_six_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA,
        ) => build_route_path_candidate_orthogonal_graph_six_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_authored_copper_graph_zone_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE,
        ) => build_route_path_candidate_authored_copper_graph_zone_aware_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE,
        ) => build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE,
        ) => build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE_LAYER_BALANCE_AWARE,
        ) => build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_authored_copper_graph_obstacle_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE,
        ) => build_route_path_candidate_authored_copper_graph_obstacle_aware_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        (
            "m5_route_path_candidate_authored_copper_plus_one_gap_v1",
            ROUTE_PROPOSAL_REASON_AUTHORED_COPPER_PLUS_ONE_GAP,
        ) => build_plus_one_gap_route_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        ),
        _ => bail!(
            "route proposal artifact apply is not supported for contract={} reason={}",
            contract,
            first_action.reason
        ),
    }
}

fn analyze_route_proposal_artifact_revalidation(
    root: &Path,
    contract: &str,
    first_action: &NativeProjectRouteProposalActionView,
    artifact_actions: &[NativeProjectRouteProposalActionView],
) -> RouteProposalArtifactRevalidationState {
    let live_actions = rebuild_route_proposal_artifact_live_actions(root, contract, first_action);
    let drift_kind = orthogonal_graph_route_proposal_artifact_drift_kind(
        contract,
        artifact_actions,
        &live_actions,
    );
    let drift_message = drift_kind.map(|kind| {
        let artifact_first = artifact_actions
            .first()
            .expect("route proposal artifact revalidation requires one action");
        render_orthogonal_graph_route_proposal_drift_message(
            kind,
            artifact_first,
            live_actions
                .as_ref()
                .ok()
                .and_then(|actions| actions.first()),
            live_actions.as_ref().err(),
        )
    });
    let matches_live = match &live_actions {
        Ok(actions) => actions == artifact_actions,
        Err(_) => false,
    };

    RouteProposalArtifactRevalidationState {
        live_actions,
        matches_live,
        drift_kind,
        drift_message,
    }
}

fn orthogonal_graph_route_proposal_artifact_drift_kind(
    contract: &str,
    artifact_actions: &[NativeProjectRouteProposalActionView],
    live_actions: &Result<Vec<NativeProjectRouteProposalActionView>>,
) -> Option<OrthogonalGraphArtifactDriftKind> {
    if !is_orthogonal_graph_route_proposal_contract(contract) {
        return None;
    }

    let artifact_first = artifact_actions.first()?;
    match live_actions {
        Ok(live_actions) => {
            let live_first = live_actions.first()?;
            if live_actions == artifact_actions {
                return None;
            }
            if artifact_first.selected_path_bend_count != live_first.selected_path_bend_count
                || artifact_first.selected_path_point_count != live_first.selected_path_point_count
                || artifact_first.selected_path_segment_count
                    != live_first.selected_path_segment_count
                || artifact_first.reused_via_uuids != live_first.reused_via_uuids
            {
                Some(OrthogonalGraphArtifactDriftKind::DeterministicCostWinnerChanged)
            } else {
                Some(OrthogonalGraphArtifactDriftKind::GeometryChanged)
            }
        }
        Err(_) => Some(OrthogonalGraphArtifactDriftKind::CandidateAvailabilityChanged),
    }
}

fn is_orthogonal_graph_route_proposal_contract(contract: &str) -> bool {
    matches!(
        contract,
        "m5_route_path_candidate_orthogonal_graph_v1"
            | "m5_route_path_candidate_orthogonal_graph_via_v1"
            | "m5_route_path_candidate_orthogonal_graph_two_via_v1"
            | "m5_route_path_candidate_orthogonal_graph_three_via_v1"
            | "m5_route_path_candidate_orthogonal_graph_four_via_v1"
            | "m5_route_path_candidate_orthogonal_graph_five_via_v1"
            | "m5_route_path_candidate_orthogonal_graph_six_via_v1"
    )
}

fn render_orthogonal_graph_route_proposal_drift_message(
    drift_kind: OrthogonalGraphArtifactDriftKind,
    artifact_action: &NativeProjectRouteProposalActionView,
    live_action: Option<&NativeProjectRouteProposalActionView>,
    error: Option<&anyhow::Error>,
) -> String {
    match drift_kind {
        OrthogonalGraphArtifactDriftKind::CandidateAvailabilityChanged => format!(
            "candidate availability changed under current authored constraints (artifact bends={}, points={}, segments={}; live rebuild failed: {})",
            artifact_action.selected_path_bend_count,
            artifact_action.selected_path_point_count,
            artifact_action.selected_path_segment_count,
            error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "unknown live rebuild failure".to_string())
        ),
        OrthogonalGraphArtifactDriftKind::DeterministicCostWinnerChanged => {
            let live_action = live_action.expect("cost-winner drift requires a live action");
            format!(
                "deterministic cost winner changed (bends {} -> {}, points {} -> {}, segments {} -> {}, reused_vias {} -> {})",
                artifact_action.selected_path_bend_count,
                live_action.selected_path_bend_count,
                artifact_action.selected_path_point_count,
                live_action.selected_path_point_count,
                artifact_action.selected_path_segment_count,
                live_action.selected_path_segment_count,
                artifact_action.reused_via_uuids.len(),
                live_action.reused_via_uuids.len(),
            )
        }
        OrthogonalGraphArtifactDriftKind::GeometryChanged => {
            let live_action = live_action.expect("geometry drift requires a live action");
            format!(
                "geometry changed under the same ranked path (artifact action {} no longer matches live action {})",
                artifact_action.action_id, live_action.action_id
            )
        }
    }
}

fn orthogonal_graph_route_proposal_artifact_segment_evidence(
    artifact_actions: &[NativeProjectRouteProposalActionView],
    live_actions: Option<&[NativeProjectRouteProposalActionView]>,
) -> Option<Vec<NativeProjectRouteProposalArtifactRevalidationSegmentView>> {
    let artifact_segments = orthogonal_graph_route_proposal_segment_facts(artifact_actions)?;
    let live_segments = live_actions.and_then(orthogonal_graph_route_proposal_segment_facts);
    let live_segment_map = live_segments
        .unwrap_or_default()
        .into_iter()
        .map(|segment| (segment.layer_segment_index, segment))
        .collect::<BTreeMap<_, _>>();
    Some(
        artifact_segments
            .into_iter()
            .map(|artifact_segment| {
                let live_segment = live_segment_map.get(&artifact_segment.layer_segment_index);
                NativeProjectRouteProposalArtifactRevalidationSegmentView {
                    layer_segment_index: artifact_segment.layer_segment_index,
                    layer_segment_count: artifact_segment.layer_segment_count,
                    artifact_layer: artifact_segment.layer,
                    artifact_bend_count: artifact_segment.bend_count,
                    artifact_point_count: artifact_segment.point_count,
                    artifact_track_action_count: artifact_segment.track_action_count,
                    live_layer: live_segment.map(|segment| segment.layer),
                    live_bend_count: live_segment.map(|segment| segment.bend_count),
                    live_point_count: live_segment.map(|segment| segment.point_count),
                    live_track_action_count: live_segment.map(|segment| segment.track_action_count),
                    matches_live: live_segment.is_some_and(|segment| {
                        segment.layer == artifact_segment.layer
                            && segment.bend_count == artifact_segment.bend_count
                            && segment.point_count == artifact_segment.point_count
                            && segment.track_action_count == artifact_segment.track_action_count
                    }),
                }
            })
            .collect(),
    )
}

fn orthogonal_graph_route_proposal_artifact_inspection_segment_evidence(
    artifact_actions: &[NativeProjectRouteProposalActionView],
) -> Option<Vec<NativeProjectRouteProposalArtifactInspectionSegmentView>> {
    Some(
        orthogonal_graph_route_proposal_segment_facts(artifact_actions)?
            .into_iter()
            .map(
                |segment| NativeProjectRouteProposalArtifactInspectionSegmentView {
                    layer_segment_index: segment.layer_segment_index,
                    layer_segment_count: segment.layer_segment_count,
                    layer: segment.layer,
                    bend_count: segment.bend_count,
                    point_count: segment.point_count,
                    track_action_count: segment.track_action_count,
                },
            )
            .collect(),
    )
}

fn orthogonal_graph_route_proposal_segment_facts(
    actions: &[NativeProjectRouteProposalActionView],
) -> Option<Vec<OrthogonalGraphArtifactSegmentFacts>> {
    let mut grouped = BTreeMap::<usize, Vec<&NativeProjectRouteProposalActionView>>::new();
    for action in actions {
        let layer_segment_index = action.selected_path_layer_segment_index?;
        grouped.entry(layer_segment_index).or_default().push(action);
    }
    Some(
        grouped
            .into_iter()
            .map(|(layer_segment_index, grouped_actions)| {
                let first = grouped_actions[0];
                OrthogonalGraphArtifactSegmentFacts {
                    layer_segment_index,
                    layer_segment_count: first.selected_path_layer_segment_count.unwrap_or(0),
                    layer: first.layer,
                    bend_count: first.selected_path_layer_segment_bend_count.unwrap_or(0),
                    point_count: first.selected_path_layer_segment_point_count.unwrap_or(0),
                    track_action_count: grouped_actions.len(),
                }
            })
            .collect(),
    )
}

pub(super) fn build_plus_one_gap_route_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_authored_copper_plus_one_gap(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic plus-one-gap path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let gap_steps = path
        .steps
        .iter()
        .enumerate()
        .filter(|(_, step)| {
            matches!(
                step.kind,
                RoutePathCandidateAuthoredCopperPlusOneGapStepKindView::Gap
            )
        })
        .collect::<Vec<_>>();
    if gap_steps.len() != 1 {
        bail!(
            "route proposal requires exactly one eligible gap, found {} for net {}",
            gap_steps.len(),
            net_uuid
        );
    }
    let (selected_gap_step_index, gap_step) = gap_steps[0];
    let action_id = route_proposal_action_id(
        &report.contract,
        "draw_track",
        ROUTE_PROPOSAL_REASON_AUTHORED_COPPER_PLUS_ONE_GAP,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        gap_step.layer,
        gap_step.from,
        gap_step.to,
        net_class.track_width_nm,
        None,
        &[],
        None,
        None,
        None,
        None,
    );

    Ok(vec![NativeProjectRouteProposalActionView {
        action_id,
        proposal_action: "draw_track".to_string(),
        reason: ROUTE_PROPOSAL_REASON_AUTHORED_COPPER_PLUS_ONE_GAP.to_string(),
        contract: report.contract,
        net_uuid: report.net_uuid,
        net_name: report.net_name,
        from_anchor_pad_uuid: report.from_anchor_pad_uuid,
        to_anchor_pad_uuid: report.to_anchor_pad_uuid,
        layer: gap_step.layer,
        width_nm: net_class.track_width_nm,
        from: gap_step.from,
        to: gap_step.to,
        reused_via_uuid: None,
        reused_via_uuids: Vec::new(),
        reused_object_kind: None,
        reused_object_uuid: None,
        reused_object_from_layer: None,
        reused_object_to_layer: None,
        selected_path_bend_count: 0,
        selected_path_point_count: path.steps.len() + 1,
        selected_path_segment_index: selected_gap_step_index,
        selected_path_segment_count: path.steps.len(),
        selected_path_layer_segment_index: None,
        selected_path_layer_segment_count: None,
        selected_path_layer_segment_bend_count: None,
        selected_path_layer_segment_point_count: None,
    }])
}

pub(super) fn build_route_path_candidate_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic single-layer path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    if path.points.len() < 2 {
        bail!(
            "route proposal requires at least two path points for net {}",
            net_uuid
        );
    }
    let selected_path_segment_count = path.points.len() - 1;
    let actions = path
        .points
        .windows(2)
        .enumerate()
        .map(|(selected_path_segment_index, segment)| {
            let from = segment[0];
            let to = segment[1];
            let action_id = route_proposal_action_id(
                &report.contract,
                "draw_track",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                path.layer,
                from,
                to,
                net_class.track_width_nm,
                None,
                &[],
                None,
                None,
                None,
                None,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "draw_track".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: path.layer,
                width_nm: net_class.track_width_nm,
                from,
                to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 0,
                selected_path_point_count: path.points.len(),
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_orthogonal_dogleg_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_orthogonal_dogleg(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic orthogonal dogleg path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected orthogonal dogleg path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    if path.points.len() < 2 {
        bail!(
            "route proposal requires at least two path points for net {}",
            net_uuid
        );
    }
    let selected_path_segment_count = path.points.len() - 1;
    let actions = path
        .points
        .windows(2)
        .enumerate()
        .map(|(selected_path_segment_index, segment)| {
            let from = segment[0];
            let to = segment[1];
            let action_id = route_proposal_action_id(
                &report.contract,
                "draw_track",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                path.layer,
                from,
                to,
                net_class.track_width_nm,
                None,
                &[],
                None,
                None,
                None,
                None,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "draw_track".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: path.layer,
                width_nm: net_class.track_width_nm,
                from,
                to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 0,
                selected_path_point_count: path.points.len(),
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_orthogonal_two_bend_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_orthogonal_two_bend(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic orthogonal two-bend path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected orthogonal two-bend path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    if path.points.len() < 2 {
        bail!(
            "route proposal requires at least two path points for net {}",
            net_uuid
        );
    }
    let selected_path_segment_count = path.points.len() - 1;
    let actions = path
        .points
        .windows(2)
        .enumerate()
        .map(|(selected_path_segment_index, segment)| {
            let from = segment[0];
            let to = segment[1];
            let action_id = route_proposal_action_id(
                &report.contract,
                "draw_track",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                path.layer,
                from,
                to,
                net_class.track_width_nm,
                None,
                &[],
                None,
                None,
                None,
                None,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "draw_track".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: path.layer,
                width_nm: net_class.track_width_nm,
                from,
                to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 0,
                selected_path_point_count: path.points.len(),
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_orthogonal_graph_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_orthogonal_graph(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic orthogonal graph path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected orthogonal graph path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    if path.points.len() < 2 {
        bail!(
            "route proposal requires at least two path points for net {}",
            net_uuid
        );
    }
    let selected_path_segment_count = path.points.len() - 1;
    let actions = path
        .points
        .windows(2)
        .enumerate()
        .map(|(selected_path_segment_index, segment)| {
            let from = segment[0];
            let to = segment[1];
            let action_id = route_proposal_action_id(
                &report.contract,
                "draw_track",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                path.layer,
                from,
                to,
                net_class.track_width_nm,
                None,
                &[],
                None,
                None,
                None,
                None,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "draw_track".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: path.layer,
                width_nm: net_class.track_width_nm,
                from,
                to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: path.cost.bend_count,
                selected_path_point_count: path.points.len(),
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: Some(0),
                selected_path_layer_segment_count: Some(1),
                selected_path_layer_segment_bend_count: Some(path.cost.bend_count),
                selected_path_layer_segment_point_count: Some(path.points.len()),
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_orthogonal_graph_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_orthogonal_graph_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic orthogonal graph via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected orthogonal graph via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_uuid),
                    &[path.via_uuid],
                    None,
                    None,
                    None,
                    None,
                );
                NativeProjectRouteProposalActionView {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_uuid),
                    reused_via_uuids: vec![path.via_uuid],
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.points.len())
                        .sum(),
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_orthogonal_graph_two_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_orthogonal_graph_two_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic orthogonal graph two-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected orthogonal graph two-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![path.via_a_uuid, path.via_b_uuid];
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_point_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len())
        .sum();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_a_uuid),
                    &reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                NativeProjectRouteProposalActionView {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_a_uuid),
                    reused_via_uuids: reused_via_uuids.clone(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_orthogonal_graph_three_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_orthogonal_graph_three_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic orthogonal graph three-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }
    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected orthogonal graph three-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![path.via_a_uuid, path.via_b_uuid, path.via_c_uuid];
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_point_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len())
        .sum();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_a_uuid),
                    &reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                NativeProjectRouteProposalActionView {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_a_uuid),
                    reused_via_uuids: reused_via_uuids.clone(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_orthogonal_graph_four_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_orthogonal_graph_four_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic orthogonal graph four-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }
    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected orthogonal graph four-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
    ];
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_point_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len())
        .sum();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FOUR_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_a_uuid),
                    &reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                NativeProjectRouteProposalActionView {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FOUR_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_a_uuid),
                    reused_via_uuids: reused_via_uuids.clone(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_orthogonal_graph_five_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_orthogonal_graph_five_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic orthogonal graph five-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }
    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected orthogonal graph five-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
        path.via_e_uuid,
    ];
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_point_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len())
        .sum();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FIVE_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_a_uuid),
                    &reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                NativeProjectRouteProposalActionView {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FIVE_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_a_uuid),
                    reused_via_uuids: reused_via_uuids.clone(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_orthogonal_graph_six_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_orthogonal_graph_six_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic orthogonal graph six-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }
    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected orthogonal graph six-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
        path.via_e_uuid,
        path.via_f_uuid,
    ];
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_point_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len())
        .sum();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_a_uuid),
                    &reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                NativeProjectRouteProposalActionView {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_a_uuid),
                    reused_via_uuids: reused_via_uuids.clone(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic single-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .map(move |pair| (selected_path_segment_index, segment.layer, pair))
        })
        .map(|(selected_path_segment_index, layer, pair)| {
            let from = pair[0];
            let to = pair[1];
            let action_id = route_proposal_action_id(
                &report.contract,
                "draw_track",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_VIA,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                layer,
                from,
                to,
                net_class.track_width_nm,
                Some(path.via_uuid),
                &[path.via_uuid],
                None,
                None,
                None,
                None,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "draw_track".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_VIA.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer,
                width_nm: net_class.track_width_nm,
                from,
                to,
                reused_via_uuid: Some(path.via_uuid),
                reused_via_uuids: vec![path.via_uuid],
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 0,
                selected_path_point_count: path
                    .segments
                    .get(selected_path_segment_index)
                    .map(|segment| segment.points.len())
                    .unwrap_or(0),
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_two_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_two_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic two-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected two-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![path.via_a_uuid, path.via_b_uuid];
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_TWO_VIA,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(super) fn build_route_path_candidate_three_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_three_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic three-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected three-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![path.via_a_uuid, path.via_b_uuid, path.via_c_uuid];
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_THREE_VIA,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(super) fn build_route_path_candidate_four_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_four_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic four-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected four-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
    ];
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FOUR_VIA,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(super) fn build_route_path_candidate_five_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_five_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic five-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected five-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
        path.via_e_uuid,
    ];
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FIVE_VIA,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(super) fn build_route_path_candidate_six_via_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_six_via(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic six-via path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected six-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
        path.via_e_uuid,
        path.via_f_uuid,
    ];
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_SIX_VIA,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(super) fn build_route_path_candidate_authored_via_chain_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_authored_via_chain(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic authored via chain path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected authored via chain path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = path
        .via_chain
        .iter()
        .map(|via| via.via_uuid)
        .collect::<Vec<_>>();
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

fn build_route_path_candidate_authored_copper_graph_zone_aware_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_authored_copper_graph_zone_aware(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic zone-aware authored-copper path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected zone-aware authored-copper path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Track => "track",
                RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Via => "via",
                RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Zone => "zone",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE
                    .to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

fn build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report =
        query_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware(
            root,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic zone-obstacle-aware authored-copper path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected zone-obstacle-aware authored-copper path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Track => {
                    "track"
                }
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Via => "via",
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Zone => "zone",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE
                    .to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

fn build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic topology-aware zone-obstacle-aware authored-copper path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected topology-aware zone-obstacle-aware authored-copper path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Track => "track",
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Via => "via",
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Zone => "zone",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE
                    .to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

fn build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic layer-balance-aware topology-aware zone-obstacle-aware authored-copper path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected layer-balance-aware topology-aware zone-obstacle-aware authored-copper path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Track => "track",
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Via => "via",
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Zone => "zone",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE_LAYER_BALANCE_AWARE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE_LAYER_BALANCE_AWARE
                    .to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

fn build_route_path_candidate_authored_copper_graph_obstacle_aware_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_authored_copper_graph_obstacle_aware(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic obstacle-aware authored-copper path for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected obstacle-aware authored-copper path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Track => "track",
                RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Via => "via",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason:
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE
                        .to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(super) fn build_route_path_candidate_authored_copper_graph_policy_proposal_actions(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    policy: NativeRoutePathCandidateAuthoredCopperGraphPolicy,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let report = query_native_project_route_path_candidate_authored_copper_graph(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        policy,
    )?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        bail!(
            "route proposal requires deterministic authored-copper graph path for net {} between {} and {} under policy {:?}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            policy
        );
    }

    let preflight = query_native_project_route_preflight(root, net_uuid)?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal requires selected authored-copper graph path data for net {} between {} and {} under policy {:?}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            policy
        )
    })?;
    let reason = route_path_candidate_authored_copper_graph_policy_reason(policy);
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track => "track",
                RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via => "via",
                RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Zone => "zone",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                reason,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            NativeProjectRouteProposalActionView {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason: reason.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

fn route_path_candidate_authored_copper_graph_policy_reason(
    policy: NativeRoutePathCandidateAuthoredCopperGraphPolicy,
) -> &'static str {
    match policy {
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::Plain => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_PLAIN
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_AWARE
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_OBSTACLE_AWARE
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_AWARE
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_AWARE
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_LAYER_BALANCE_AWARE
        }
    }
}

fn route_path_candidate_authored_copper_graph_policy_from_reason(
    reason: &str,
) -> Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy> {
    match reason {
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_PLAIN => {
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::Plain)
        }
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_AWARE => {
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware)
        }
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_OBSTACLE_AWARE => {
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware)
        }
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_AWARE => {
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware)
        }
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_AWARE => {
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware)
        }
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_LAYER_BALANCE_AWARE => {
            Some(
                NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware,
            )
        }
        _ => None,
    }
}

fn build_segmented_route_proposal_actions(
    contract: &str,
    reason: &str,
    net_uuid: Uuid,
    net_name: &str,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    width_nm: i64,
    segments: Vec<(i32, &[Point])>,
    reused_via_uuids: &[Uuid],
) -> Vec<NativeProjectRouteProposalActionView> {
    let selected_path_segment_count = segments.len();
    let primary_reused_via_uuid = reused_via_uuids.first().copied();
    segments
        .into_iter()
        .enumerate()
        .flat_map(|(selected_path_segment_index, (layer, points))| {
            points.windows(2).map(move |pair| {
                (
                    selected_path_segment_index,
                    layer,
                    points.len(),
                    pair[0],
                    pair[1],
                )
            })
        })
        .map(
            |(selected_path_segment_index, layer, selected_path_point_count, from, to)| {
                let action_id = route_proposal_action_id(
                    contract,
                    "draw_track",
                    reason,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    width_nm,
                    primary_reused_via_uuid,
                    reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                NativeProjectRouteProposalActionView {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: reason.to_string(),
                    contract: contract.to_string(),
                    net_uuid,
                    net_name: net_name.to_string(),
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    width_nm,
                    from,
                    to,
                    reused_via_uuid: primary_reused_via_uuid,
                    reused_via_uuids: reused_via_uuids.to_vec(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: 0,
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: None,
                    selected_path_layer_segment_count: None,
                    selected_path_layer_segment_bend_count: None,
                    selected_path_layer_segment_point_count: None,
                }
            },
        )
        .collect()
}

fn route_proposal_action_id(
    contract: &str,
    proposal_action: &str,
    reason: &str,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    layer: i32,
    from: Point,
    to: Point,
    width_nm: i64,
    reused_via_uuid: Option<Uuid>,
    reused_via_uuids: &[Uuid],
    reused_object_kind: Option<&str>,
    reused_object_uuid: Option<Uuid>,
    reused_object_from_layer: Option<i32>,
    reused_object_to_layer: Option<i32>,
) -> String {
    let reused_via_uuid_sequence = reused_via_uuids
        .iter()
        .map(Uuid::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let reused_object_kind = reused_object_kind.unwrap_or_default();
    let stable_key = format!(
        "{contract}|{proposal_action}|{reason}|{net_uuid}|{from_anchor_pad_uuid}|{to_anchor_pad_uuid}|{layer}|{}:{}|{}:{}|{width_nm}|{}|{reused_via_uuid_sequence}|{reused_object_kind}|{}|{}|{}",
        from.x,
        from.y,
        to.x,
        to.y,
        reused_via_uuid
            .map(|uuid| uuid.to_string())
            .unwrap_or_default(),
        reused_object_uuid
            .map(|uuid| uuid.to_string())
            .unwrap_or_default(),
        reused_object_from_layer
            .map(|layer| layer.to_string())
            .unwrap_or_default(),
        reused_object_to_layer
            .map(|layer| layer.to_string())
            .unwrap_or_default(),
    );
    compute_source_hash_bytes(stable_key.as_bytes())
}
