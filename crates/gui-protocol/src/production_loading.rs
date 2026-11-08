//! Production projection reuses the workspace's qualified engine basis for
//! Proposal details instead of resolving the same Project twice per Proposal.
use super::*;
use eda_engine::api::native_write::forward_annotation::proposal_render_deltas;
use eda_engine::substrate::{
    DesignModel, preview_proposal_diff_journaled, validate_proposal_apply,
};

pub(super) fn load_production_status(
    cli: &[String],
    request: &LiveReviewRequest,
) -> Result<ProductionStatus> {
    load(cli, request, None)
}

pub(super) fn load(
    cli: &[String],
    request: &LiveReviewRequest,
    model: Option<&DesignModel>,
) -> Result<ProductionStatus> {
    if request.board_file.is_some() {
        return Ok(ProductionStatus::default());
    }
    let project_root = request.project_root.display().to_string();
    let output_jobs: OutputJobsPayload =
        match run_cli_json(cli, &["project", "query", &project_root, "output-jobs"]) {
            Ok(payload) => payload,
            Err(_) => return Ok(ProductionStatus::default()),
        };
    let manufacturing_plans = run_cli_json(
        cli,
        &["project", "query", &project_root, "manufacturing-plans"],
    )
    .unwrap_or_default();
    let panel_projections = run_cli_json(
        cli,
        &["project", "query", &project_root, "panel-projections"],
    )
    .unwrap_or_default();
    let artifact_list = run_cli_json(cli, &["artifact", "list", &project_root]).unwrap_or_default();
    let proposals = match model {
        Some(model) => serde_json::from_value(serde_json::json!({
            "proposal_count": model.proposals.len(), "proposals": model.proposals,
        }))?,
        None => run_cli_json(cli, &["proposal", "list", &project_root]).unwrap_or_default(),
    };
    let mut status = production_payloads_to_production_status(
        output_jobs,
        artifact_list,
        proposals,
        manufacturing_plans,
        panel_projections,
    );
    if let Some(model) = model {
        attach_model_proposal_details(model, &request.project_root, &mut status)?;
    } else {
        production_proposals::attach_proposal_validation(cli, &project_root, &mut status);
    }
    if let Some(artifact_id) = focused_artifact_id(&status) {
        let args = vec![
            "artifact".to_string(),
            "files".to_string(),
            project_root.clone(),
            "--artifact".to_string(),
            artifact_id,
        ];
        if let Ok(payload) = run_cli_json_owned::<ArtifactFilesPayload>(cli, &args) {
            let mut detail = artifact_files_payload_to_detail(payload);
            if let Some(file) = detail.focused_file.as_ref() {
                detail.focused_preview =
                    load_artifact_file_preview(cli, &project_root, &detail.artifact_id, &file.path)
                        .ok();
            }
            status.focused_artifact = Some(detail);
        }
    }
    Ok(status)
}

fn attach_model_proposal_details(
    model: &DesignModel,
    root: &Path,
    status: &mut ProductionStatus,
) -> Result<()> {
    for proposal in &mut status.proposals {
        let id = proposal.proposal_id.parse()?;
        let validation = validate_proposal_apply(model, id)?;
        proposal.can_apply = Some(validation.can_apply);
        proposal.blocker_codes = validation.blockers.into_iter().map(|b| b.code).collect();
        // Match the CLI's optional preview exactly: use the same journaled
        // prediction and render-delta functions, including their refusals.
        if let Ok(preview) = preview_proposal_diff_journaled(model, root, id)
            && let Ok(deltas) = proposal_render_deltas(&model.proposals[&id].batch)
        {
            proposal.preview = Some(ProductionProposalPreviewSummary {
                prepared_against: preview.prepared_against.0,
                preview_after_model_revision: preview.preview_after_model_revision.0,
                created_count: preview.diff.created.len(),
                modified_count: preview.diff.modified.len(),
                deleted_count: preview.diff.deleted.len(),
                affected_object_count: preview.affected_objects.len(),
                affected_objects: preview
                    .affected_objects
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
                render_deltas: deltas
                    .into_iter()
                    .map(|d| ProductionProposalRenderDeltaSummary {
                        delta_kind: d.delta_kind.into(),
                        object_id: d.object_id,
                        primitive_kind: d.primitive_kind.into(),
                        layer_id: d.layer_id,
                        end_layer_id: d.end_layer_id,
                        width_nm: d.width_nm,
                        drill_nm: d.drill_nm,
                        diameter_nm: d.diameter_nm,
                        path: d
                            .path
                            .into_iter()
                            .map(|p| PointNm { x: p.x, y: p.y })
                            .collect(),
                    })
                    .collect(),
            });
        }
    }
    Ok(())
}
