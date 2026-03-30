use super::*;
use crate::NativeProjectRouteApplyView;

pub(crate) fn apply_native_project_route(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    candidate: &str,
) -> Result<NativeProjectRouteApplyView> {
    let actions = match candidate {
        "authored-copper-plus-one-gap" => {
            super::build_plus_one_gap_route_proposal_actions(
                root,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?
        }
        _ => bail!("unsupported route-apply candidate {}", candidate),
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
