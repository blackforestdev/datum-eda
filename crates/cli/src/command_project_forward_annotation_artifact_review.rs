use super::command_project_forward_annotation_artifact::load_forward_annotation_proposal_artifact;
use super::*;
use std::collections::BTreeMap;

pub(crate) fn import_forward_annotation_artifact_review(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactReviewImportView> {
    let mut project = load_native_project(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    if loaded.artifact.project_uuid != project.manifest.uuid {
        bail!(
            "forward-annotation artifact project UUID {} does not match current project UUID {}",
            loaded.artifact.project_uuid,
            project.manifest.uuid
        );
    }

    let live_proposal = query_native_project_forward_annotation_proposal(root)?;
    let live_actions = live_proposal
        .actions
        .into_iter()
        .map(|action| (action.action_id.clone(), action))
        .collect::<BTreeMap<_, _>>();

    let total_artifact_reviews = loaded.artifact.reviews.len();
    let mut imported_reviews = 0usize;
    let mut skipped_missing_live_actions = 0usize;
    for review in loaded.artifact.reviews {
        if let Some(live_action) = live_actions.get(&review.action_id) {
            project.manifest.forward_annotation_review.insert(
                review.action_id.clone(),
                NativeForwardAnnotationReviewRecord {
                    action_id: review.action_id,
                    decision: review.decision,
                    proposal_action: live_action.action.clone(),
                    reference: live_action.reference.clone(),
                    reason: live_action.reason.clone(),
                },
            );
            imported_reviews += 1;
        } else {
            skipped_missing_live_actions += 1;
        }
    }

    write_canonical_json(&project.root.join("project.json"), &project.manifest)?;
    Ok(NativeProjectForwardAnnotationArtifactReviewImportView {
        action: "import_forward_annotation_artifact_review".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: project.root.display().to_string(),
        imported_reviews,
        skipped_missing_live_actions,
        total_artifact_reviews,
    })
}

pub(crate) fn replace_forward_annotation_artifact_review(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactReviewReplaceView> {
    let mut project = load_native_project(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    if loaded.artifact.project_uuid != project.manifest.uuid {
        bail!(
            "forward-annotation artifact project UUID {} does not match current project UUID {}",
            loaded.artifact.project_uuid,
            project.manifest.uuid
        );
    }

    let live_proposal = query_native_project_forward_annotation_proposal(root)?;
    let live_actions = live_proposal
        .actions
        .into_iter()
        .map(|action| (action.action_id.clone(), action))
        .collect::<BTreeMap<_, _>>();

    let total_artifact_reviews = loaded.artifact.reviews.len();
    let removed_existing_reviews = project.manifest.forward_annotation_review.len();
    let mut replacement_reviews = BTreeMap::new();
    let mut replaced_reviews = 0usize;
    let mut skipped_missing_live_actions = 0usize;
    for review in loaded.artifact.reviews {
        if let Some(live_action) = live_actions.get(&review.action_id) {
            replacement_reviews.insert(
                review.action_id.clone(),
                NativeForwardAnnotationReviewRecord {
                    action_id: review.action_id,
                    decision: review.decision,
                    proposal_action: live_action.action.clone(),
                    reference: live_action.reference.clone(),
                    reason: live_action.reason.clone(),
                },
            );
            replaced_reviews += 1;
        } else {
            skipped_missing_live_actions += 1;
        }
    }

    project.manifest.forward_annotation_review = replacement_reviews;
    write_canonical_json(&project.root.join("project.json"), &project.manifest)?;
    Ok(NativeProjectForwardAnnotationArtifactReviewReplaceView {
        action: "replace_forward_annotation_artifact_review".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: project.root.display().to_string(),
        replaced_reviews,
        removed_existing_reviews,
        skipped_missing_live_actions,
        total_artifact_reviews,
    })
}
