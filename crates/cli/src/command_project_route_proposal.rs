use std::path::Path;

use super::*;
use crate::NativeProjectRouteApplyCandidateArg;
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
    pub(crate) selected_path_point_count: usize,
    pub(crate) selected_path_segment_index: usize,
    pub(crate) selected_path_segment_count: usize,
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

pub(crate) fn export_native_project_route_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    let project = load_native_project(root)?;
    let actions = build_plus_one_gap_route_proposal_actions(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
    )?;
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
        action: "export_route_proposal".to_string(),
        artifact_path: output_path.display().to_string(),
        kind: artifact.kind,
        version: artifact.version,
        project_uuid: artifact.project_uuid.to_string(),
        contract: artifact.contract,
        actions: artifact.actions.len(),
    })
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
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        candidate,
        policy,
    )
}

fn export_route_path_proposal_artifact_for_candidate(
    root: &Path,
    output_path: &Path,
    action: &str,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    candidate: NativeProjectRouteApplyCandidateArg,
    policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
) -> Result<NativeProjectRouteProposalExportReportView> {
    let actions = build_route_path_proposal_actions(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        candidate,
        policy,
    )?;
    export_route_proposal_artifact(root, output_path, action, actions)
}

pub(crate) fn export_native_project_route_path_candidate_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::RoutePathCandidate,
        None,
    )
}

pub(crate) fn export_native_project_route_path_candidate_via_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_via_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateVia,
        None,
    )
}

pub(crate) fn export_native_project_route_path_candidate_two_via_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_two_via_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateTwoVia,
        None,
    )
}

pub(crate) fn export_native_project_route_path_candidate_three_via_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_three_via_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateThreeVia,
        None,
    )
}

pub(crate) fn export_native_project_route_path_candidate_four_via_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_four_via_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFourVia,
        None,
    )
}

pub(crate) fn export_native_project_route_path_candidate_five_via_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_five_via_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFiveVia,
        None,
    )
}

pub(crate) fn export_native_project_route_path_candidate_six_via_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_six_via_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateSixVia,
        None,
    )
}

pub(crate) fn export_native_project_route_path_candidate_authored_via_chain_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_authored_via_chain_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateAuthoredViaChain,
        None,
    )
}

pub(crate) fn export_native_project_route_path_candidate_authored_copper_graph_zone_aware_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_authored_copper_graph_zone_aware_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
        Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware),
    )
}

pub(crate) fn export_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_authored_copper_graph_zone_obstacle_aware_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
        Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware),
    )
}

pub(crate) fn export_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
        Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware),
    )
}

pub(crate) fn export_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
        Some(
            NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware,
        ),
    )
}

pub(crate) fn export_native_project_route_path_candidate_authored_copper_graph_obstacle_aware_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_authored_copper_graph_obstacle_aware_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
        Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware),
    )
}

pub(crate) fn export_native_project_route_path_candidate_authored_copper_graph_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    policy: NativeRoutePathCandidateAuthoredCopperGraphPolicy,
    output_path: &Path,
) -> Result<NativeProjectRouteProposalExportReportView> {
    export_route_path_proposal_artifact_for_candidate(
        root,
        output_path,
        "export_route_path_candidate_authored_copper_graph_proposal",
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
        Some(policy),
    )
}

fn export_route_proposal_artifact(
    root: &Path,
    output_path: &Path,
    action: &str,
    actions: Vec<NativeProjectRouteProposalActionView>,
) -> Result<NativeProjectRouteProposalExportReportView> {
    let project = load_native_project(root)?;
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
    let live_actions = if loaded.artifact.contract
        == "m5_route_path_candidate_authored_copper_graph_policy_v1"
    {
        let policy =
            route_path_candidate_authored_copper_graph_policy_from_reason(&first_action.reason)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "route proposal artifact apply is not supported for contract={} reason={}",
                        loaded.artifact.contract,
                        first_action.reason
                    )
                })?;
        build_route_path_candidate_authored_copper_graph_policy_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
            policy,
        )?
    } else {
        match (
            loaded.artifact.contract.as_str(),
            first_action.reason.as_str(),
        ) {
        ("m5_route_path_candidate_v2", ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE) => {
            build_route_path_candidate_proposal_actions(
                root,
                first_action.net_uuid,
                first_action.from_anchor_pad_uuid,
                first_action.to_anchor_pad_uuid,
            )?
        }
        ("m5_route_path_candidate_via_v1", ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_VIA) => {
            build_route_path_candidate_via_proposal_actions(
                root,
                first_action.net_uuid,
                first_action.from_anchor_pad_uuid,
                first_action.to_anchor_pad_uuid,
            )?
        }
        (
            "m5_route_path_candidate_two_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_TWO_VIA,
        ) => build_route_path_candidate_two_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_three_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_THREE_VIA,
        ) => build_route_path_candidate_three_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_four_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FOUR_VIA,
        ) => build_route_path_candidate_four_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_five_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FIVE_VIA,
        ) => build_route_path_candidate_five_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_six_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_SIX_VIA,
        ) => build_route_path_candidate_six_via_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_authored_via_chain_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN,
        ) => build_route_path_candidate_authored_via_chain_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_authored_copper_graph_zone_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE,
        ) => build_route_path_candidate_authored_copper_graph_zone_aware_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE,
        ) => build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE,
        ) => build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE_LAYER_BALANCE_AWARE,
        ) => build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_authored_copper_graph_obstacle_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE,
        ) => build_route_path_candidate_authored_copper_graph_obstacle_aware_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        (
            "m5_route_path_candidate_authored_copper_plus_one_gap_v1",
            ROUTE_PROPOSAL_REASON_AUTHORED_COPPER_PLUS_ONE_GAP,
        ) => build_plus_one_gap_route_proposal_actions(
            root,
            first_action.net_uuid,
            first_action.from_anchor_pad_uuid,
            first_action.to_anchor_pad_uuid,
        )?,
        _ => bail!(
            "route proposal artifact apply is not supported for contract={} reason={}",
            loaded.artifact.contract,
            first_action.reason
        ),
        }
    };
    if live_actions != loaded.artifact.actions {
        bail!(
            "route proposal artifact drifted for contract {}; refresh the proposal before apply",
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
        applied,
    })
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
        selected_path_point_count: path.steps.len() + 1,
        selected_path_segment_index: selected_gap_step_index,
        selected_path_segment_count: path.steps.len(),
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
                selected_path_point_count: path.points.len(),
                selected_path_segment_index,
                selected_path_segment_count,
            }
        })
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
                selected_path_point_count: path
                    .segments
                    .get(selected_path_segment_index)
                    .map(|segment| segment.points.len())
                    .unwrap_or(0),
                selected_path_segment_index,
                selected_path_segment_count,
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
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
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
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
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
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
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
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
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
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
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
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
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
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
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
