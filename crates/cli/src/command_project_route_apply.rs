use super::*;
use crate::NativeProjectRouteApplyCandidateArg;
use crate::NativeProjectRouteApplyView;
use crate::cli_args::NativeRoutePathCandidateAuthoredCopperGraphPolicy;

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
    let actions = match candidate {
        NativeProjectRouteApplyCandidateArg::RoutePathCandidate => {
            super::build_route_path_candidate_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateVia => {
            super::build_route_path_candidate_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateTwoVia => {
            super::build_route_path_candidate_two_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateThreeVia => {
            super::build_route_path_candidate_three_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFourVia => {
            super::build_route_path_candidate_four_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFiveVia => {
            super::build_route_path_candidate_five_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateSixVia => {
            super::build_route_path_candidate_six_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateAuthoredViaChain => {
            super::build_route_path_candidate_authored_via_chain_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalDogleg => {
            super::build_route_path_candidate_orthogonal_dogleg_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalTwoBend => {
            super::build_route_path_candidate_orthogonal_two_bend_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraph => {
            super::build_route_path_candidate_orthogonal_graph_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphVia => {
            super::build_route_path_candidate_orthogonal_graph_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphTwoVia => {
            super::build_route_path_candidate_orthogonal_graph_two_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphThreeVia => {
            super::build_route_path_candidate_orthogonal_graph_three_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFourVia => {
            super::build_route_path_candidate_orthogonal_graph_four_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFiveVia => {
            super::build_route_path_candidate_orthogonal_graph_five_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphSixVia => {
            super::build_route_path_candidate_orthogonal_graph_six_via_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperPlusOneGap => {
            super::build_plus_one_gap_route_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph => {
            let policy = policy.ok_or_else(|| {
                anyhow::anyhow!("route-apply candidate authored-copper-graph requires --policy")
            })?;
            super::build_route_path_candidate_authored_copper_graph_policy_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                policy,
            )?
        }
    };
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
    for action in actions {
        if action.proposal_action != "draw_track"
            && action.proposal_action != "reuse_existing_copper_step"
        {
            bail!(
                "route proposal apply is not supported for {} reason={}",
                action.proposal_action,
                action.reason
            );
        }
    }
    Ok(())
}

pub(super) fn apply_route_proposal_actions(
    root: &Path,
    actions: &[NativeProjectRouteProposalActionView],
) -> Result<Vec<NativeProjectBoardTrackMutationReportView>> {
    let mut applied = Vec::new();
    for action in actions {
        if action.proposal_action == "draw_track" {
            applied.push(place_native_project_board_track(
                root,
                action.net_uuid,
                action.from,
                action.to,
                action.width_nm,
                action.layer,
            )?);
        }
    }
    Ok(applied)
}
