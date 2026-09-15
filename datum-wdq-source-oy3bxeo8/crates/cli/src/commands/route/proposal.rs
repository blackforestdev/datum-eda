use std::path::Path;

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
use crate::args::NativeProjectRouteStrategyBatchGatePolicyArg;
use crate::args::NativeRoutePathCandidateAuthoredCopperGraphPolicy;
use crate::*;
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::board::RoutePathCandidateAuthoredCopperGraphPolicy as EngineAuthoredCopperGraphPolicy;
use eda_engine::board::route_proposal::{self, RouteProposalCandidate, RouteProposalProfile};
use eda_engine::board::{
    Net, NetClass, PadShape, PlacedPad, StackupLayer, StackupLayerType, Track, Via,
};
use eda_engine::substrate::ProjectResolver;
use serde_json::Value;

const ROUTE_PROPOSAL_ARTIFACT_KIND: &str = "native_route_proposal_artifact";
const ROUTE_PROPOSAL_ARTIFACT_VERSION: u32 = 1;
const ROUTE_STRATEGY_BATCH_REQUESTS_KIND: &str = "native_route_strategy_batch_requests";
const ROUTE_STRATEGY_BATCH_REQUESTS_VERSION: u32 = 1;
const ROUTE_STRATEGY_BATCH_RESULT_KIND: &str = "native_route_strategy_batch_result_artifact";
const ROUTE_STRATEGY_BATCH_RESULT_VERSION: u32 = 1;
const ROUTE_STRATEGY_CURATED_FIXTURE_SUITE_ID: &str = "m6_route_strategy_curated_fixture_suite_v1";
const ROUTE_STRATEGY_FIXTURE_AUTHORING_BOUNDARY: &str = "generated_fixture_only";
#[rustfmt::skip] const ROUTE_STRATEGY_FIXTURE_WRITE_PATH_POLICY: &str = "direct project-shard writes are restricted to deterministic regression fixture generation";

pub(crate) use eda_engine::board::route_proposal::RouteProposalAction as NativeProjectRouteProposalActionView;

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
    let project = load_native_project_with_resolved_board(root)?;
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
    let built_proposal = super::proposal_substrate::build_accepted_route_proposal(root, &actions)?;
    let artifact = RouteProposalArtifact {
        kind: ROUTE_PROPOSAL_ARTIFACT_KIND.to_string(),
        version: ROUTE_PROPOSAL_ARTIFACT_VERSION,
        project_uuid: project.manifest.uuid,
        project_name: project.manifest.name.clone(),
        contract: actions[0].contract.clone(),
        actions,
        proposal: built_proposal.proposal,
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
    let spec = engine_route_proposal_candidate(candidate, policy).ok_or_else(|| {
        anyhow::anyhow!(
            "export-route-path-proposal candidate authored-copper-graph requires --policy"
        )
    })?;
    build_route_proposal_actions_for_spec(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        spec,
    )
}

/// Map the CLI candidate/policy args onto the engine candidate spec.
///
/// `None` marks the single unrepresentable combination — authored-copper-graph
/// without a policy — so each caller can raise its own command-specific
/// missing-policy error (historical error texts differ per command).
pub(crate) fn engine_route_proposal_candidate(
    candidate: NativeProjectRouteApplyCandidateArg,
    policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
) -> Option<RouteProposalCandidate> {
    Some(match candidate {
        NativeProjectRouteApplyCandidateArg::RoutePathCandidate => {
            RouteProposalCandidate::RoutePathCandidate
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateVia => {
            RouteProposalCandidate::RoutePathCandidateVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateTwoVia => {
            RouteProposalCandidate::RoutePathCandidateTwoVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateThreeVia => {
            RouteProposalCandidate::RoutePathCandidateThreeVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFourVia => {
            RouteProposalCandidate::RoutePathCandidateFourVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFiveVia => {
            RouteProposalCandidate::RoutePathCandidateFiveVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateSixVia => {
            RouteProposalCandidate::RoutePathCandidateSixVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateAuthoredViaChain => {
            RouteProposalCandidate::RoutePathCandidateAuthoredViaChain
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalDogleg => {
            RouteProposalCandidate::RoutePathCandidateOrthogonalDogleg
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalTwoBend => {
            RouteProposalCandidate::RoutePathCandidateOrthogonalTwoBend
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraph => {
            RouteProposalCandidate::RoutePathCandidateOrthogonalGraph
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphVia => {
            RouteProposalCandidate::RoutePathCandidateOrthogonalGraphVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphTwoVia => {
            RouteProposalCandidate::RoutePathCandidateOrthogonalGraphTwoVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphThreeVia => {
            RouteProposalCandidate::RoutePathCandidateOrthogonalGraphThreeVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFourVia => {
            RouteProposalCandidate::RoutePathCandidateOrthogonalGraphFourVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFiveVia => {
            RouteProposalCandidate::RoutePathCandidateOrthogonalGraphFiveVia
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphSixVia => {
            RouteProposalCandidate::RoutePathCandidateOrthogonalGraphSixVia
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperPlusOneGap => {
            RouteProposalCandidate::AuthoredCopperPlusOneGap
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph => {
            return policy.map(|policy| {
                RouteProposalCandidate::AuthoredCopperGraph(engine_authored_copper_graph_policy(
                    policy,
                ))
            });
        }
    })
}

fn engine_authored_copper_graph_policy(
    policy: NativeRoutePathCandidateAuthoredCopperGraphPolicy,
) -> EngineAuthoredCopperGraphPolicy {
    match policy {
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::Plain => {
            EngineAuthoredCopperGraphPolicy::Plain
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware => {
            EngineAuthoredCopperGraphPolicy::ZoneAware
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware => {
            EngineAuthoredCopperGraphPolicy::ObstacleAware
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware => {
            EngineAuthoredCopperGraphPolicy::ZoneObstacleAware
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware => {
            EngineAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware => {
            EngineAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware
        }
    }
}

fn engine_route_proposal_profile(
    profile: NativeProjectRouteProposalProfileArg,
) -> RouteProposalProfile {
    match profile {
        NativeProjectRouteProposalProfileArg::Default => RouteProposalProfile::Default,
        NativeProjectRouteProposalProfileArg::AuthoredCopperPriority => {
            RouteProposalProfile::AuthoredCopperPriority
        }
    }
}

/// Resolve the project board and build one candidate's proposal actions
/// through the engine route-proposal kernel.
pub(crate) fn build_route_proposal_actions_for_spec(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    spec: RouteProposalCandidate,
) -> Result<Vec<NativeProjectRouteProposalActionView>> {
    let project = load_native_project_with_resolved_board(root)?;
    let board = build_native_project_board(&project)?;
    route_proposal::build_route_proposal_actions(
        &board,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        spec,
    )
    .map_err(anyhow::Error::msg)
}

struct RouteProposalSelectionOutcome {
    report: NativeProjectRouteProposalSelectionView,
    selected_spec: Option<RouteProposalCandidate>,
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
    let actions = build_route_proposal_actions_for_spec(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selected_spec,
    )?;
    super::apply::validate_route_proposal_actions(&actions)?;
    let first_action = actions.first().ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal review candidate {} produced no actions",
            route_proposal::candidate_spec_name(&selected_spec)
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
        selected_candidate: Some(route_proposal::candidate_name(selected_spec).to_string()),
        selected_policy: route_proposal::candidate_policy_name(selected_spec),
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

#[rustfmt::skip]
fn write_route_strategy_batch_requests_manifest(path: &Path, requests: &[RouteStrategyBatchRequestInput]) -> Result<()> {
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

/// Provenance stamped on every curated route-strategy fixture commit.
fn route_strategy_fixture_provenance() -> Result<WriteProvenance> {
    Ok(WriteProvenance::new(
        "datum-eda-cli",
        cli_commit_source()?,
        "seed route strategy curated fixture board",
    ))
}

/// Commit one curated fixture board spec as a single composed batch through
/// the native-write facade (see `route_proposal::fixtures`): the fixtures
/// exercise the facade + resolver instead of hand-written `board.json`.
fn commit_route_strategy_fixture_board(
    root: &Path,
    board_uuid: Uuid,
    spec: &route_proposal::RouteStrategyFixtureBoardSpec,
) -> Result<()> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared = route_proposal::build_route_strategy_fixture_board_write(
        &model,
        route_strategy_fixture_provenance()?,
        board_uuid,
        spec,
    )?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(())
}

fn route_strategy_fixture_board_uuid(created: &NativeProjectCreateReportView) -> Result<Uuid> {
    Uuid::parse_str(&created.board_uuid).context("fixture project board uuid must parse")
}

/// The single curated net class every fixture uses (byte-identical facts to
/// the historical hand-written fixture `net_classes` entry).
fn route_strategy_fixture_net_class(class_uuid: Uuid) -> NetClass {
    NetClass {
        uuid: class_uuid,
        name: "Default".to_string(),
        clearance: 150_000,
        track_width: 200_000,
        via_drill: 300_000,
        via_diameter: 600_000,
        diffpair_width: 0,
        diffpair_gap: 0,
    }
}

fn route_strategy_fixture_net(uuid: Uuid, name: &str, class: Uuid) -> Net {
    Net {
        uuid,
        name: name.to_string(),
        class,
        controlled_impedance: None,
    }
}

/// One circular anchor pad (parses back to the same `PlacedPad` facts the
/// historical hand-written fixture pads produced).
fn route_strategy_fixture_pad(
    uuid: Uuid,
    package: Uuid,
    net: Uuid,
    position: Point,
    layer: i32,
    diameter: i64,
) -> PlacedPad {
    PlacedPad {
        uuid,
        package,
        name: "1".to_string(),
        net: Some(net),
        position,
        layer,
        copper_layers: Vec::new(),
        shape: PadShape::Circle,
        diameter,
        width: 0,
        height: 0,
        drill: 0,
        rotation: 0,
        roundrect_rratio_ppm: 250_000,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
    }
}

/// Copper(1) / Dielectric / Copper(3) — the cross-layer fixture stackup.
fn route_strategy_fixture_three_layer_stackup() -> Vec<StackupLayer> {
    vec![
        StackupLayer::new(1, "Top Copper", StackupLayerType::Copper, 35_000),
        StackupLayer::new(2, "Core", StackupLayerType::Dielectric, 1_600_000),
        StackupLayer::new(3, "Bottom Copper", StackupLayerType::Copper, 35_000),
    ]
}

fn route_strategy_fixture_outline(width_nm: i64, height_nm: i64) -> Polygon {
    Polygon {
        vertices: vec![
            Point { x: 0, y: 0 },
            Point { x: width_nm, y: 0 },
            Point {
                x: width_nm,
                y: height_nm,
            },
            Point { x: 0, y: height_nm },
        ],
        closed: true,
    }
}

fn seed_curated_route_strategy_same_outcome_fixture(root: &Path) -> Result<(Uuid, Uuid, Uuid)> {
    let created = create_native_project(
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
    let spec = route_proposal::RouteStrategyFixtureBoardSpec {
        stackup_layers: route_strategy_fixture_three_layer_stackup(),
        outline: route_strategy_fixture_outline(5_000_000, 3_000_000),
        net_classes: vec![route_strategy_fixture_net_class(class_uuid)],
        nets: vec![
            route_strategy_fixture_net(target_net_uuid, "SIG", class_uuid),
            route_strategy_fixture_net(other_net_uuid, "GND", class_uuid),
        ],
        pads: vec![
            route_strategy_fixture_pad(
                anchor_a_uuid,
                package_a_uuid,
                target_net_uuid,
                Point {
                    x: 500_000,
                    y: 600_000,
                },
                1,
                450_000,
            ),
            route_strategy_fixture_pad(
                anchor_b_uuid,
                package_b_uuid,
                target_net_uuid,
                Point {
                    x: 4_500_000,
                    y: 2_400_000,
                },
                1,
                450_000,
            ),
        ],
        tracks: Vec::new(),
        vias: Vec::new(),
    };
    commit_route_strategy_fixture_board(root, route_strategy_fixture_board_uuid(&created)?, &spec)?;
    Ok((target_net_uuid, anchor_a_uuid, anchor_b_uuid))
}

fn seed_curated_route_strategy_profile_divergence_fixture(
    root: &Path,
) -> Result<(Uuid, Uuid, Uuid)> {
    let created = create_native_project(
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
    let spec = route_proposal::RouteStrategyFixtureBoardSpec {
        stackup_layers: vec![StackupLayer::new(
            1,
            "Top Copper",
            StackupLayerType::Copper,
            35_000,
        )],
        outline: route_strategy_fixture_outline(4_000_000, 1_000_000),
        net_classes: vec![route_strategy_fixture_net_class(class_uuid)],
        nets: vec![route_strategy_fixture_net(
            target_net_uuid,
            "SIG",
            class_uuid,
        )],
        pads: vec![
            route_strategy_fixture_pad(
                anchor_a_uuid,
                package_a_uuid,
                target_net_uuid,
                Point {
                    x: 500_000,
                    y: 500_000,
                },
                1,
                400_000,
            ),
            route_strategy_fixture_pad(
                anchor_b_uuid,
                package_b_uuid,
                target_net_uuid,
                Point {
                    x: 3_500_000,
                    y: 500_000,
                },
                1,
                400_000,
            ),
        ],
        tracks: vec![Track {
            uuid: authored_track_uuid,
            net: target_net_uuid,
            from: Point {
                x: 500_000,
                y: 500_000,
            },
            to: Point {
                x: 3_500_000,
                y: 500_000,
            },
            width: 200_000,
            layer: 1,
        }],
        vias: Vec::new(),
    };
    commit_route_strategy_fixture_board(root, route_strategy_fixture_board_uuid(&created)?, &spec)?;
    Ok((target_net_uuid, anchor_a_uuid, anchor_b_uuid))
}

fn seed_curated_route_strategy_via_fixture(root: &Path) -> Result<(Uuid, Uuid, Uuid)> {
    let created = create_native_project(root, Some("Route Strategy Curated Via Demo".to_string()))?;

    let target_net_uuid = Uuid::from_u128(0xa10);
    let class_uuid = Uuid::from_u128(0xa11);
    let package_a_uuid = Uuid::from_u128(0xa12);
    let package_b_uuid = Uuid::from_u128(0xa13);
    let anchor_a_uuid = Uuid::from_u128(0xa14);
    let anchor_b_uuid = Uuid::from_u128(0xa15);
    let via_uuid = Uuid::from_u128(0xa16);
    let spec = route_proposal::RouteStrategyFixtureBoardSpec {
        stackup_layers: route_strategy_fixture_three_layer_stackup(),
        outline: route_strategy_fixture_outline(5_000_000, 3_000_000),
        net_classes: vec![route_strategy_fixture_net_class(class_uuid)],
        nets: vec![route_strategy_fixture_net(
            target_net_uuid,
            "SIG",
            class_uuid,
        )],
        pads: vec![
            route_strategy_fixture_pad(
                anchor_a_uuid,
                package_a_uuid,
                target_net_uuid,
                Point {
                    x: 500_000,
                    y: 600_000,
                },
                1,
                450_000,
            ),
            route_strategy_fixture_pad(
                anchor_b_uuid,
                package_b_uuid,
                target_net_uuid,
                Point {
                    x: 4_500_000,
                    y: 2_400_000,
                },
                3,
                450_000,
            ),
        ],
        tracks: Vec::new(),
        vias: vec![Via {
            uuid: via_uuid,
            net: target_net_uuid,
            position: Point {
                x: 2_500_000,
                y: 1_500_000,
            },
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 3,
        }],
    };
    commit_route_strategy_fixture_board(root, route_strategy_fixture_board_uuid(&created)?, &spec)?;
    Ok((target_net_uuid, anchor_a_uuid, anchor_b_uuid))
}

fn seed_curated_route_strategy_no_proposal_fixture(root: &Path) -> Result<(Uuid, Uuid, Uuid)> {
    let (net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid) =
        seed_curated_route_strategy_same_outcome_fixture(root)?;
    // Clear the target net's class to JSON null (the historical fixture's
    // defining no-proposal state) through the facade-guarded SetBoardNet.
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared = route_proposal::build_route_strategy_fixture_net_class_clear(
        &model,
        route_strategy_fixture_provenance()?,
        &route_strategy_fixture_net(net_uuid, "SIG", Uuid::from_u128(0xc202)),
    )?;
    commit_prepared(&mut model, root, prepared)?;
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
        authoring_boundary: ROUTE_STRATEGY_FIXTURE_AUTHORING_BOUNDARY.to_string(),
        write_path_policy: ROUTE_STRATEGY_FIXTURE_WRITE_PATH_POLICY.to_string(),
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

#[rustfmt::skip]
pub(crate) fn capture_route_strategy_curated_baseline(out_dir: &Path, manifest_path_override: Option<&Path>, result_path_override: Option<&Path>) -> Result<NativeProjectRouteStrategyCuratedBaselineCaptureView> {
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
        authoring_boundary: suite.authoring_boundary,
        write_path_policy: suite.write_path_policy,
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
    let actions = build_route_proposal_actions_for_spec(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selected_spec,
    )?;
    let export =
        export_route_proposal_artifact(root, output_path, "export_route_path_proposal", actions)?;
    Ok(NativeProjectSelectedRouteProposalExportView {
        action: "export_route_proposal".to_string(),
        project_root: root.display().to_string(),
        selection_profile: route_proposal_profile_name(profile).to_string(),
        selection_rule: selection.report.selection_rule,
        selected_candidate: route_proposal::candidate_name(selected_spec).to_string(),
        selected_policy: route_proposal::candidate_policy_name(selected_spec),
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
    let apply = super::apply::apply_native_project_route_for_spec(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selected_spec,
    )?;
    Ok(NativeProjectRouteApplySelectedView {
        action: "route_apply_selected".to_string(),
        project_root: root.display().to_string(),
        selection_profile: route_proposal_profile_name(profile).to_string(),
        selection_rule: selection.report.selection_rule,
        selected_candidate: route_proposal::candidate_name(selected_spec).to_string(),
        selected_policy: route_proposal::candidate_policy_name(selected_spec),
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
    let outcome = route_proposal::run_route_proposal_selection(
        engine_route_proposal_profile(profile),
        |spec| {
            build_route_proposal_actions_for_spec(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                *spec,
            )
            .map_err(|error| error.to_string())
        },
    )
    .map_err(anyhow::Error::msg)?;

    let candidates = outcome
        .candidates
        .into_iter()
        .map(
            |candidate| NativeProjectRouteProposalSelectionCandidateView {
                candidate: candidate.candidate,
                policy: candidate.policy,
                selected: candidate.selected,
                contract: candidate.contract,
                actions: candidate.actions,
                selected_path_bend_count: candidate.selected_path_bend_count,
                selected_path_point_count: candidate.selected_path_point_count,
                selected_path_segment_count: candidate.selected_path_segment_count,
                message: candidate.message,
            },
        )
        .collect();

    Ok(RouteProposalSelectionOutcome {
        report: NativeProjectRouteProposalSelectionView {
            action: "route_proposal".to_string(),
            project_root: root.display().to_string(),
            net_uuid: net_uuid.to_string(),
            from_anchor_pad_uuid: from_anchor_pad_uuid.to_string(),
            to_anchor_pad_uuid: to_anchor_pad_uuid.to_string(),
            selection_profile: route_proposal_profile_name(profile).to_string(),
            status: outcome.status,
            selection_rule: outcome.selection_rule,
            attempted_candidates: outcome.attempted_candidates,
            selected_candidate: outcome.selected_candidate,
            selected_policy: outcome.selected_policy,
            selected_contract: outcome.selected_contract,
            selected_actions: outcome.selected_actions,
            selected_path_bend_count: outcome.selected_path_bend_count,
            selected_path_point_count: outcome.selected_path_point_count,
            selected_path_segment_count: outcome.selected_path_segment_count,
            candidates,
        },
        selected_spec: outcome.selected_spec,
    })
}

fn accepted_route_strategy_profiles() -> [NativeProjectRouteProposalProfileArg; 2] {
    [
        NativeProjectRouteProposalProfileArg::Default,
        NativeProjectRouteProposalProfileArg::AuthoredCopperPriority,
    ]
}

fn route_proposal_profile_name(profile: NativeProjectRouteProposalProfileArg) -> &'static str {
    route_proposal::profile_name(engine_route_proposal_profile(profile))
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
    route_proposal::profile_distinction(engine_route_proposal_profile(profile))
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
    super::apply::validate_route_proposal_actions(&loaded.artifact.actions)?;

    let first_action = loaded.artifact.actions.first().ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal artifact {} must contain at least one action",
            loaded.artifact_path.display()
        )
    })?;
    let live_board = load_route_proposal_live_board(root);
    let revalidation = route_proposal::analyze_route_proposal_artifact_revalidation(
        &live_board,
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
    let _live_actions = revalidation.live_actions.map_err(anyhow::Error::msg)?;
    if !revalidation.matches_live {
        bail!(
            "route proposal artifact drifted for contract {}: geometry changed under the same ranked path; refresh the proposal before apply",
            loaded.artifact.contract
        );
    }
    #[rustfmt::skip]
    let has_mutating_actions = loaded.artifact.actions.iter().any(|action| action.proposal_action == "draw_track");
    #[rustfmt::skip]
    let applied = if let Some(proposal) = loaded.artifact.proposal.clone() {
        if proposal.status != eda_engine::substrate::ProposalStatus::Accepted {
            bail!("route proposal artifact {} has proposal status {:?}; expected accepted before apply", loaded.artifact_path.display(), proposal.status);
        }
        super::proposal_substrate::apply_route_proposal(root, &loaded.artifact.actions, proposal)?
    } else if has_mutating_actions {
        bail!("route proposal artifact {} is missing accepted proposal metadata; re-export the proposal before apply", loaded.artifact_path.display());
    } else {
        Vec::new()
    };

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
    let live_board = load_route_proposal_live_board(root);
    let revalidation = route_proposal::analyze_route_proposal_artifact_revalidation(
        &live_board,
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

fn load_route_proposal_live_board(root: &Path) -> std::result::Result<Board, String> {
    load_native_project_with_resolved_board(root)
        .and_then(|project| build_native_project_board(&project))
        .map_err(|error| error.to_string())
}

fn orthogonal_graph_route_proposal_artifact_segment_evidence(
    artifact_actions: &[NativeProjectRouteProposalActionView],
    live_actions: Option<&[NativeProjectRouteProposalActionView]>,
) -> Option<Vec<NativeProjectRouteProposalArtifactRevalidationSegmentView>> {
    Some(
        route_proposal::orthogonal_graph_route_proposal_segment_comparison(
            artifact_actions,
            live_actions,
        )?
        .into_iter()
        .map(
            |segment| NativeProjectRouteProposalArtifactRevalidationSegmentView {
                layer_segment_index: segment.layer_segment_index,
                layer_segment_count: segment.layer_segment_count,
                artifact_layer: segment.artifact_layer,
                artifact_bend_count: segment.artifact_bend_count,
                artifact_point_count: segment.artifact_point_count,
                artifact_track_action_count: segment.artifact_track_action_count,
                live_layer: segment.live_layer,
                live_bend_count: segment.live_bend_count,
                live_point_count: segment.live_point_count,
                live_track_action_count: segment.live_track_action_count,
                matches_live: segment.matches_live,
            },
        )
        .collect(),
    )
}

fn orthogonal_graph_route_proposal_artifact_inspection_segment_evidence(
    artifact_actions: &[NativeProjectRouteProposalActionView],
) -> Option<Vec<NativeProjectRouteProposalArtifactInspectionSegmentView>> {
    Some(
        route_proposal::orthogonal_graph_route_proposal_segment_facts(artifact_actions)?
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

// Phase 5: exec-layer dissolution — variant run() impls (the former
// command_exec destructure-and-forward glue, now inherent methods on the
// clap args structs).

impl ProjectReviewRouteProposalArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
            artifact,
        } = self;
        let report = review_native_project_route_proposal(
            path.as_deref(),
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
            artifact.as_deref(),
        )?;
        let output = render_report(format, &report, render_native_route_proposal_review_text);
        Ok((output, 0))
    }
}

impl ProjectRouteStrategyReportArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            objective,
        } = self;
        let report = report_native_project_route_strategy(
            &path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            objective,
        )?;
        let output = render_report(format, &report, render_native_route_strategy_report_text);
        Ok((output, 0))
    }
}

impl ProjectRouteStrategyCompareArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        } = self;
        let report = compare_native_project_route_strategy(
            &path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
        let output = render_report(format, &report, render_native_route_strategy_compare_text);
        Ok((output, 0))
    }
}

impl ProjectRouteStrategyDeltaArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        } = self;
        let report = report_native_project_route_strategy_delta(
            &path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
        let output = render_report(format, &report, render_native_route_strategy_delta_text);
        Ok((output, 0))
    }
}

impl ProjectWriteRouteStrategyCuratedFixtureSuiteArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { out_dir, manifest } = self;
        let report = write_route_strategy_curated_fixture_suite(&out_dir, manifest.as_deref())?;
        let output = match format {
            OutputFormat::Text => render_native_route_strategy_curated_fixture_suite_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectCaptureRouteStrategyCuratedBaselineArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            out_dir,
            manifest,
            result,
        } = self;
        let report = capture_route_strategy_curated_baseline(
            &out_dir,
            manifest.as_deref(),
            result.as_deref(),
        )?;
        let output = match format {
            OutputFormat::Text => {
                render_native_route_strategy_curated_baseline_capture_text(&report)
            }
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectRouteStrategyBatchEvaluateArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { requests } = self;
        let report = evaluate_native_project_route_strategy_batch(&requests)?;
        let output = render_report(
            format,
            &report,
            render_native_route_strategy_batch_evaluate_text,
        );
        Ok((output, 0))
    }
}

impl ProjectInspectRouteStrategyBatchResultArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path } = self;
        let report = inspect_route_strategy_batch_result(&path)?;
        let output = match format {
            OutputFormat::Text => {
                render_native_route_strategy_batch_result_inspection_text(&report)
            }
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectValidateRouteStrategyBatchResultArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path } = self;
        let report = validate_route_strategy_batch_result(&path)?;
        let output = match format {
            OutputFormat::Text => {
                render_native_route_strategy_batch_result_validation_text(&report)
            }
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectCompareRouteStrategyBatchResultArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { before, after } = self;
        let report = compare_route_strategy_batch_result(&before, &after)?;
        let output = match format {
            OutputFormat::Text => {
                render_native_route_strategy_batch_result_comparison_text(&report)
            }
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectGateRouteStrategyBatchResultArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            before,
            after,
            policy,
        } = self;
        let report = gate_route_strategy_batch_result(&before, &after, policy)?;
        let output = render_report(
            format,
            &report,
            render_native_route_strategy_batch_result_gate_text,
        );
        let exit_code = if report.passed { 0 } else { 2 };
        Ok((output, exit_code))
    }
}

impl ProjectSummarizeRouteStrategyBatchResultsArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            dir,
            artifacts,
            baseline,
            policy,
        } = self;
        let report = summarize_route_strategy_batch_results(
            dir.as_deref(),
            &artifacts,
            baseline.as_deref(),
            policy,
        )?;
        let output = match format {
            OutputFormat::Text => render_native_route_strategy_batch_results_index_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectRouteProposalArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
        } = self;
        let report = select_native_project_route_proposal(
            &path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
        )?;
        let output = render_report(format, &report, render_native_route_proposal_selection_text);
        Ok((output, 0))
    }
}

impl ProjectRouteProposalExplainArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
        } = self;
        let report = explain_native_project_route_proposal(
            &path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
        )?;
        let output = render_report(format, &report, render_native_route_proposal_explain_text);
        Ok((output, 0))
    }
}

impl ProjectExportRouteProposalArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
            out,
        } = self;
        let report = export_selected_native_project_route_proposal(
            &path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
            &out,
        )?;
        let output = render_report(
            format,
            &report,
            render_native_selected_route_proposal_export_text,
        );
        Ok((output, 0))
    }
}

impl ProjectExportRoutePathProposalArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate,
            policy,
            out,
        } = self;
        let report = export_native_project_route_path_proposal(
            &path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate,
            policy,
            &out,
        )?;
        let output = render_report(format, &report, render_native_route_proposal_export_text);
        Ok((output, 0))
    }
}

impl ProjectInspectRouteProposalArtifactArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path } = self;
        let report = inspect_route_proposal_artifact(&path)?;
        let output = match format {
            OutputFormat::Text => render_native_route_proposal_artifact_inspection_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectRevalidateRouteProposalArtifactArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, artifact } = self;
        let report = revalidate_route_proposal_artifact(&path, &artifact)?;
        let output = match format {
            OutputFormat::Text => render_native_route_proposal_artifact_revalidation_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectApplyRouteProposalArtifactArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, artifact } = self;
        let report = apply_route_proposal_artifact(&path, &artifact)?;
        let output = render_report(
            format,
            &report,
            render_native_route_proposal_artifact_apply_text,
        );
        Ok((output, 0))
    }
}

impl ProjectRouteApplySelectedArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
        } = self;
        let report = apply_selected_native_project_route(
            &path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
        )?;
        let output = render_report(format, &report, render_native_route_apply_selected_text);
        Ok((output, 0))
    }
}

impl ProjectRouteApplyArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate,
            policy,
        } = self;
        let report = apply_native_project_route(
            &path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate,
            policy,
        )?;
        let output = render_report(format, &report, render_native_route_apply_text);
        Ok((output, 0))
    }
}
