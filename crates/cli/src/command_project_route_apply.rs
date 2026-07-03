use super::*;
use crate::NativeProjectRouteAppliedTrackReportView;
use crate::NativeProjectRouteApplyCandidateArg;
use crate::NativeProjectRouteApplyView;
use crate::cli_args::NativeRoutePathCandidateAuthoredCopperGraphPolicy;
use eda_engine::board::route_proposal::{self, RouteProposalCandidate};

pub(crate) fn apply_native_project_route(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    candidate: NativeProjectRouteApplyCandidateArg,
    policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
) -> Result<NativeProjectRouteApplyView> {
    if policy.is_some() && candidate != NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph {
        bail!("route-apply --policy is supported only for candidate authored-copper-graph");
    }
    let spec = super::engine_route_proposal_candidate(candidate, policy).ok_or_else(|| {
        anyhow::anyhow!("route-apply candidate authored-copper-graph requires --policy")
    })?;
    apply_native_project_route_for_spec(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        spec,
    )
}

pub(crate) fn apply_native_project_route_for_spec(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    spec: RouteProposalCandidate,
) -> Result<NativeProjectRouteApplyView> {
    let actions = super::build_route_proposal_actions_for_spec(
        root,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        spec,
    )?;
    validate_route_proposal_actions(&actions)?;
    let contract = actions
        .first()
        .map(|action| action.contract.clone())
        .ok_or_else(|| anyhow::anyhow!("route-apply requires at least one proposal action"))?;
    let applied = apply_route_proposal_actions(root, &actions)?;
    Ok(NativeProjectRouteApplyView {
        action: "route_apply".to_string(),
        project_root: root.display().to_string(),
        contract,
        proposal_actions: actions.len(),
        applied_actions: applied.len(),
        applied,
    })
}

pub(super) fn validate_route_proposal_actions(
    actions: &[NativeProjectRouteProposalActionView],
) -> Result<()> {
    route_proposal::validate_route_proposal_actions(actions).map_err(anyhow::Error::msg)
}

pub(super) fn apply_route_proposal_actions(
    root: &Path,
    actions: &[NativeProjectRouteProposalActionView],
) -> Result<Vec<NativeProjectRouteAppliedTrackReportView>> {
    let built = super::command_project_route_proposal_substrate::build_accepted_route_proposal(
        root, actions,
    )?;
    super::command_project_route_proposal_substrate::apply_built_route_proposal(root, built)
}
