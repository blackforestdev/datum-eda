use std::path::Path;

use super::*;
use crate::NativeProjectRouteApplyCandidateArg;
use crate::NativeProjectRouteApplySelectedView;
use crate::NativeProjectRouteProposalArtifactInspectionSegmentView;
use crate::NativeProjectRouteProposalArtifactRevalidationSegmentView;
use crate::NativeProjectRouteProposalArtifactRevalidationView;
use crate::NativeProjectRouteProposalSelectionCandidateView;
use crate::NativeProjectRouteProposalSelectionView;
use crate::NativeProjectSelectedRouteProposalExportView;
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

const ROUTE_PROPOSAL_ARTIFACT_KIND: &str = "native_route_proposal_artifact";
const ROUTE_PROPOSAL_ARTIFACT_VERSION: u32 = 1;
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
) -> Result<NativeProjectRouteProposalSelectionView> {
    Ok(run_native_project_route_proposal_selection(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?
    .report)
}

pub(crate) fn export_selected_native_project_route_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectSelectedRouteProposalExportView> {
    let selection = run_native_project_route_proposal_selection(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
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
) -> Result<NativeProjectRouteApplySelectedView> {
    let selection = run_native_project_route_proposal_selection(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
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
) -> Result<RouteProposalSelectionOutcome> {
    let specs = route_proposal_selection_specs();
    let selection_rule = format!(
        "select the first successful candidate in this deterministic order: {}",
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

fn route_proposal_selection_specs() -> Vec<RouteProposalSelectionCandidateSpec> {
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
