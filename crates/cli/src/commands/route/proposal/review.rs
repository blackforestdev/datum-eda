//! Read-only producing review exposes existing project and Proposal authority.
use super::*;

pub(super) fn review_selected_native_project_route_proposal(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    profile: NativeProjectRouteProposalProfileArg,
) -> Result<NativeProjectRouteProposalReviewView> {
    let model = ProjectResolver::new(root).resolve()?;
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
    super::super::apply::validate_route_proposal_actions(&actions)?;
    let first_action = actions.first().ok_or_else(|| {
        anyhow::anyhow!(
            "route proposal review candidate {} produced no actions",
            route_proposal::candidate_spec_name(&selected_spec)
        )
    })?;
    let segment_evidence =
        orthogonal_graph_route_proposal_artifact_inspection_segment_evidence(&actions);
    let final_basis = ProjectResolver::new(root).resolve()?;
    if final_basis.project.project_id != model.project.project_id
        || final_basis.model_revision != model.model_revision
    {
        bail!("route review source changed during query; retry on a coherent basis");
    }
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
        project_uuid: Some(model.project.project_id.to_string()),
        producer_revision: Some(model.model_revision.0.clone()),
        producer_proposal: None,
        source_fingerprint: None,
        project_name: Some(model.project.name.clone()),
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

pub(super) fn review_route_proposal_artifact(
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
        producer_revision: loaded
            .artifact
            .proposal
            .as_ref()
            .map(|p| p.prepared_against.0.clone()),
        producer_proposal: loaded.artifact.proposal.clone(),
        source_fingerprint: Some(loaded.source_fingerprint.clone()),
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
