use super::*;
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) fn compare_forward_annotation_proposal_artifact(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactComparisonView> {
    let project = load_native_project_with_resolved_board(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let current_proposal = query_native_project_forward_annotation_proposal(root)?;

    let mut current_by_id = BTreeMap::new();
    let mut current_by_identity = BTreeMap::new();
    for action in current_proposal.actions {
        if let Some(key) = forward_annotation_action_identity_key(&action) {
            current_by_identity.insert(key, ());
        }
        current_by_id.insert(action.action_id.clone(), action);
    }

    let review_by_id = loaded
        .artifact
        .reviews
        .iter()
        .map(|review| (review.action_id.clone(), review.decision.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut actions = Vec::new();
    for action in &loaded.artifact.actions {
        let status = if current_by_id.contains_key(&action.action_id) {
            "applicable"
        } else if forward_annotation_action_identity_key(action)
            .is_some_and(|key| current_by_identity.contains_key(&key))
        {
            "drifted"
        } else {
            "stale"
        };
        actions.push(NativeProjectForwardAnnotationArtifactComparisonActionView {
            action_id: action.action_id.clone(),
            proposal_action: action.action.clone(),
            reference: action.reference.clone(),
            reason: action.reason.clone(),
            status: status.to_string(),
            review_decision: review_by_id.get(&action.action_id).cloned(),
        });
    }
    actions.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.proposal_action.cmp(&b.proposal_action))
            .then_with(|| a.reason.cmp(&b.reason))
            .then_with(|| a.action_id.cmp(&b.action_id))
    });

    let applicable_actions = actions
        .iter()
        .filter(|action| action.status == "applicable")
        .count();
    let drifted_actions = actions
        .iter()
        .filter(|action| action.status == "drifted")
        .count();
    let stale_actions = actions
        .iter()
        .filter(|action| action.status == "stale")
        .count();

    Ok(NativeProjectForwardAnnotationArtifactComparisonView {
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: project.root.display().to_string(),
        kind: loaded.artifact.kind,
        artifact_version: loaded.artifact.version,
        current_project_uuid: project.manifest.uuid.to_string(),
        artifact_project_uuid: loaded.artifact.project_uuid.to_string(),
        artifact_actions: actions.len(),
        applicable_actions,
        drifted_actions,
        stale_actions,
        actions,
    })
}

fn forward_annotation_action_identity_key(
    action: &NativeProjectForwardAnnotationProposalActionView,
) -> Option<(String, String, String, String)> {
    let symbol = action.symbol_uuid.as_deref().unwrap_or("");
    let component = action.component_uuid.as_deref().unwrap_or("");
    (!symbol.is_empty() || !component.is_empty()).then(|| {
        (
            action.action.clone(),
            action.reason.clone(),
            symbol.to_string(),
            component.to_string(),
        )
    })
}
