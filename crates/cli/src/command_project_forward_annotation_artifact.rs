use super::*;

const FORWARD_ANNOTATION_ARTIFACT_KIND: &str = "native_forward_annotation_proposal_artifact";
const FORWARD_ANNOTATION_ARTIFACT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ForwardAnnotationProposalArtifact {
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) project_uuid: Uuid,
    pub(crate) project_name: String,
    pub(crate) actions: Vec<NativeProjectForwardAnnotationProposalActionView>,
    pub(crate) reviews: Vec<NativeProjectForwardAnnotationReviewActionView>,
}

pub(crate) struct LoadedForwardAnnotationProposalArtifact {
    pub(crate) artifact_path: PathBuf,
    pub(crate) source_version: u32,
    pub(crate) artifact: ForwardAnnotationProposalArtifact,
}

pub(crate) fn export_native_project_forward_annotation_proposal(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectForwardAnnotationExportReportView> {
    let project = load_native_project(root)?;
    let proposal = query_native_project_forward_annotation_proposal(root)?;
    let review = query_native_project_forward_annotation_review(root)?;
    let artifact = ForwardAnnotationProposalArtifact {
        kind: FORWARD_ANNOTATION_ARTIFACT_KIND.to_string(),
        version: FORWARD_ANNOTATION_ARTIFACT_VERSION,
        project_uuid: project.manifest.uuid,
        project_name: project.manifest.name.clone(),
        actions: proposal.actions,
        reviews: review.actions,
    };
    write_canonical_json(output_path, &artifact)?;
    Ok(NativeProjectForwardAnnotationExportReportView {
        action: "export_forward_annotation_proposal".to_string(),
        artifact_path: output_path.display().to_string(),
        kind: artifact.kind,
        version: artifact.version,
        project_uuid: artifact.project_uuid.to_string(),
        actions: artifact.actions.len(),
        reviews: artifact.reviews.len(),
    })
}

pub(crate) fn export_native_project_forward_annotation_proposal_selection(
    root: &Path,
    action_ids: &[String],
    output_path: &Path,
) -> Result<NativeProjectForwardAnnotationExportReportView> {
    if action_ids.is_empty() {
        bail!("forward-annotation proposal selection export requires at least one --action-id");
    }

    let project = load_native_project(root)?;
    let proposal = query_native_project_forward_annotation_proposal(root)?;
    let review = query_native_project_forward_annotation_review(root)?;
    let selected_action_ids = action_ids.iter().cloned().collect::<BTreeSet<_>>();
    let actions = proposal
        .actions
        .into_iter()
        .filter(|action| selected_action_ids.contains(&action.action_id))
        .collect::<Vec<_>>();
    if actions.len() != selected_action_ids.len() {
        let exported_action_ids = actions
            .iter()
            .map(|action| action.action_id.as_str())
            .collect::<BTreeSet<_>>();
        let missing = selected_action_ids
            .iter()
            .filter(|action_id| !exported_action_ids.contains(action_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        bail!(
            "forward-annotation proposal action not found for selection export: {}",
            missing.join(", ")
        );
    }

    let reviews = review
        .actions
        .into_iter()
        .filter(|entry| selected_action_ids.contains(&entry.action_id))
        .collect::<Vec<_>>();
    let artifact = ForwardAnnotationProposalArtifact {
        kind: FORWARD_ANNOTATION_ARTIFACT_KIND.to_string(),
        version: FORWARD_ANNOTATION_ARTIFACT_VERSION,
        project_uuid: project.manifest.uuid,
        project_name: project.manifest.name.clone(),
        actions,
        reviews,
    };
    write_canonical_json(output_path, &artifact)?;
    Ok(NativeProjectForwardAnnotationExportReportView {
        action: "export_forward_annotation_proposal_selection".to_string(),
        artifact_path: output_path.display().to_string(),
        kind: artifact.kind,
        version: artifact.version,
        project_uuid: artifact.project_uuid.to_string(),
        actions: artifact.actions.len(),
        reviews: artifact.reviews.len(),
    })
}

pub(crate) fn select_forward_annotation_proposal_artifact(
    artifact_path: &Path,
    action_ids: &[String],
    output_path: &Path,
) -> Result<NativeProjectForwardAnnotationExportReportView> {
    if action_ids.is_empty() {
        bail!("forward-annotation artifact selection requires at least one --action-id");
    }

    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let selected_action_ids = action_ids.iter().cloned().collect::<BTreeSet<_>>();
    let actions = loaded
        .artifact
        .actions
        .into_iter()
        .filter(|action| selected_action_ids.contains(&action.action_id))
        .collect::<Vec<_>>();
    if actions.len() != selected_action_ids.len() {
        let exported_action_ids = actions
            .iter()
            .map(|action| action.action_id.as_str())
            .collect::<BTreeSet<_>>();
        let missing = selected_action_ids
            .iter()
            .filter(|action_id| !exported_action_ids.contains(action_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        bail!(
            "forward-annotation artifact action not found for selection: {}",
            missing.join(", ")
        );
    }

    let reviews = loaded
        .artifact
        .reviews
        .into_iter()
        .filter(|entry| selected_action_ids.contains(&entry.action_id))
        .collect::<Vec<_>>();
    let artifact = ForwardAnnotationProposalArtifact {
        kind: FORWARD_ANNOTATION_ARTIFACT_KIND.to_string(),
        version: FORWARD_ANNOTATION_ARTIFACT_VERSION,
        project_uuid: loaded.artifact.project_uuid,
        project_name: loaded.artifact.project_name,
        actions,
        reviews,
    };
    write_canonical_json(output_path, &artifact)?;
    Ok(NativeProjectForwardAnnotationExportReportView {
        action: "select_forward_annotation_proposal_artifact".to_string(),
        artifact_path: output_path.display().to_string(),
        kind: artifact.kind,
        version: artifact.version,
        project_uuid: artifact.project_uuid.to_string(),
        actions: artifact.actions.len(),
        reviews: artifact.reviews.len(),
    })
}

pub(crate) fn load_forward_annotation_proposal_artifact(
    artifact_path: &Path,
) -> Result<LoadedForwardAnnotationProposalArtifact> {
    let contents = std::fs::read_to_string(artifact_path).with_context(|| {
        format!(
            "failed to read forward-annotation artifact {}",
            artifact_path.display()
        )
    })?;
    let value = serde_json::from_str::<serde_json::Value>(&contents).with_context(|| {
        format!(
            "failed to parse forward-annotation artifact {}",
            artifact_path.display()
        )
    })?;

    let kind = value.get("kind").and_then(serde_json::Value::as_str);
    if let Some(kind) = kind
        && kind != FORWARD_ANNOTATION_ARTIFACT_KIND
    {
        bail!(
            "unsupported forward-annotation artifact kind '{}' in {}",
            kind,
            artifact_path.display()
        );
    }

    let version = match value.get("version") {
        Some(version) => {
            let raw = version.as_u64().ok_or_else(|| {
                anyhow::Error::msg(format!(
                    "invalid forward-annotation artifact version in {}",
                    artifact_path.display()
                ))
            })?;
            u32::try_from(raw).map_err(|_| {
                anyhow::Error::msg(format!(
                    "invalid forward-annotation artifact version in {}",
                    artifact_path.display()
                ))
            })?
        }
        None => 0,
    };

    match version {
        FORWARD_ANNOTATION_ARTIFACT_VERSION => {
            let artifact = serde_json::from_value::<ForwardAnnotationProposalArtifact>(value)
                .with_context(|| {
                    format!(
                        "failed to parse forward-annotation artifact {}",
                        artifact_path.display()
                    )
                })?;
            if artifact.kind != FORWARD_ANNOTATION_ARTIFACT_KIND {
                bail!(
                    "unsupported forward-annotation artifact kind '{}' in {}",
                    artifact.kind,
                    artifact_path.display()
                );
            }
            Ok(LoadedForwardAnnotationProposalArtifact {
                artifact_path: artifact_path.to_path_buf(),
                source_version: FORWARD_ANNOTATION_ARTIFACT_VERSION,
                artifact,
            })
        }
        _ => {
            bail!(
                "unsupported forward-annotation artifact version {} in {}",
                version,
                artifact_path.display()
            );
        }
    }
}

pub(crate) fn inspect_forward_annotation_proposal_artifact(
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactInspectionView> {
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let add_component_actions = loaded
        .artifact
        .actions
        .iter()
        .filter(|action| action.action == "add_component")
        .count();
    let remove_component_actions = loaded
        .artifact
        .actions
        .iter()
        .filter(|action| action.action == "remove_component")
        .count();
    let update_component_actions = loaded
        .artifact
        .actions
        .iter()
        .filter(|action| action.action == "update_component")
        .count();
    let deferred_reviews = loaded
        .artifact
        .reviews
        .iter()
        .filter(|review| review.decision == "deferred")
        .count();
    let rejected_reviews = loaded
        .artifact
        .reviews
        .iter()
        .filter(|review| review.decision == "rejected")
        .count();

    Ok(NativeProjectForwardAnnotationArtifactInspectionView {
        artifact_path: loaded.artifact_path.display().to_string(),
        kind: loaded.artifact.kind,
        source_version: loaded.source_version,
        version: loaded.artifact.version,
        migration_applied: false,
        project_uuid: loaded.artifact.project_uuid.to_string(),
        project_name: loaded.artifact.project_name,
        actions: loaded.artifact.actions.len(),
        reviews: loaded.artifact.reviews.len(),
        add_component_actions,
        remove_component_actions,
        update_component_actions,
        deferred_reviews,
        rejected_reviews,
    })
}

pub(crate) fn compare_forward_annotation_proposal_artifact(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactComparisonView> {
    let project = load_native_project(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let current_proposal = query_native_project_forward_annotation_proposal(root)?;

    let mut current_by_id = BTreeMap::new();
    let mut current_by_reference_and_action = BTreeMap::new();
    for action in current_proposal.actions {
        current_by_reference_and_action.insert(
            (action.reference.clone(), action.action.clone()),
            action.action_id.clone(),
        );
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
        } else if current_by_reference_and_action
            .contains_key(&(action.reference.clone(), action.action.clone()))
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

pub(crate) fn filter_forward_annotation_proposal_artifact(
    root: &Path,
    artifact_path: &Path,
    output_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactFilterView> {
    let project = load_native_project(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let comparison = compare_forward_annotation_proposal_artifact(root, artifact_path)?;
    let applicable_action_ids = comparison
        .actions
        .iter()
        .filter(|action| action.status == "applicable")
        .map(|action| action.action_id.as_str())
        .collect::<BTreeSet<_>>();

    let filtered_artifact = ForwardAnnotationProposalArtifact {
        kind: loaded.artifact.kind,
        version: loaded.artifact.version,
        project_uuid: loaded.artifact.project_uuid,
        project_name: loaded.artifact.project_name,
        actions: loaded
            .artifact
            .actions
            .into_iter()
            .filter(|action| applicable_action_ids.contains(action.action_id.as_str()))
            .collect(),
        reviews: loaded
            .artifact
            .reviews
            .into_iter()
            .filter(|review| applicable_action_ids.contains(review.action_id.as_str()))
            .collect(),
    };
    write_canonical_json(output_path, &filtered_artifact)?;

    Ok(NativeProjectForwardAnnotationArtifactFilterView {
        action: "filter_forward_annotation_proposal_artifact".to_string(),
        input_artifact_path: loaded.artifact_path.display().to_string(),
        output_artifact_path: output_path.display().to_string(),
        project_root: project.root.display().to_string(),
        kind: filtered_artifact.kind,
        version: filtered_artifact.version,
        artifact_actions: comparison.artifact_actions,
        applicable_actions: filtered_artifact.actions.len(),
        filtered_reviews: filtered_artifact.reviews.len(),
    })
}

fn forward_annotation_apply_required_inputs(
    action: &NativeProjectForwardAnnotationProposalActionView,
) -> (&'static str, Vec<String>) {
    match (action.action.as_str(), action.reason.as_str()) {
        ("remove_component", "board_component_missing_in_schematic") => {
            ("self_sufficient", Vec::new())
        }
        ("update_component", "value_mismatch") => ("self_sufficient", Vec::new()),
        ("add_component", _) => {
            let mut required = vec![
                "package_uuid".to_string(),
                "x_nm".to_string(),
                "y_nm".to_string(),
                "layer".to_string(),
            ];
            if action.schematic_part_uuid.is_none() {
                required.push("part_uuid".to_string());
            }
            ("requires_explicit_input", required)
        }
        ("update_component", "part_mismatch") => {
            let mut required = vec!["package_uuid".to_string()];
            if action.schematic_part_uuid.is_none() {
                required.push("part_uuid".to_string());
            }
            ("requires_explicit_input", required)
        }
        _ => ("unsupported", Vec::new()),
    }
}

pub(crate) fn plan_forward_annotation_proposal_artifact_apply(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactApplyPlanView> {
    let comparison = compare_forward_annotation_proposal_artifact(root, artifact_path)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let review_by_id = loaded
        .artifact
        .reviews
        .iter()
        .map(|review| (review.action_id.clone(), review.decision.clone()))
        .collect::<BTreeMap<_, _>>();
    let actions_by_id = loaded
        .artifact
        .actions
        .iter()
        .map(|action| (action.action_id.clone(), action.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut actions = Vec::new();
    for comparison_action in comparison.actions {
        let artifact_action = actions_by_id
            .get(&comparison_action.action_id)
            .ok_or_else(|| anyhow::anyhow!("artifact action missing during apply planning"))?;
        let (execution, required_inputs) = if comparison_action.status == "applicable" {
            let (execution, required_inputs) =
                forward_annotation_apply_required_inputs(artifact_action);
            (execution.to_string(), required_inputs)
        } else {
            ("not_applicable".to_string(), Vec::new())
        };
        actions.push(NativeProjectForwardAnnotationArtifactApplyPlanActionView {
            action_id: comparison_action.action_id,
            proposal_action: comparison_action.proposal_action,
            reference: comparison_action.reference,
            reason: comparison_action.reason,
            applicability: comparison_action.status,
            execution,
            review_decision: review_by_id.get(&artifact_action.action_id).cloned(),
            required_inputs,
        });
    }

    let self_sufficient_actions = actions
        .iter()
        .filter(|action| action.execution == "self_sufficient")
        .count();
    let requires_input_actions = actions
        .iter()
        .filter(|action| action.execution == "requires_explicit_input")
        .count();
    let not_applicable_actions = actions
        .iter()
        .filter(|action| action.execution == "not_applicable")
        .count();

    Ok(NativeProjectForwardAnnotationArtifactApplyPlanView {
        action: "plan_forward_annotation_proposal_artifact_apply".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: root.display().to_string(),
        kind: loaded.artifact.kind,
        artifact_version: loaded.artifact.version,
        artifact_actions: actions.len(),
        self_sufficient_actions,
        requires_input_actions,
        not_applicable_actions,
        actions,
    })
}

pub(crate) fn apply_forward_annotation_proposal_artifact(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactApplyView> {
    let project = load_native_project(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    if loaded.artifact.project_uuid != project.manifest.uuid {
        bail!(
            "forward-annotation artifact project UUID {} does not match current project UUID {}",
            loaded.artifact.project_uuid,
            project.manifest.uuid
        );
    }

    let plan = plan_forward_annotation_proposal_artifact_apply(root, artifact_path)?;
    let non_applicable = plan
        .actions
        .iter()
        .find(|action| action.applicability != "applicable");
    if let Some(action) = non_applicable {
        bail!(
            "forward-annotation artifact apply requires only applicable actions; action {} is {}",
            action.action_id,
            action.applicability
        );
    }
    let input_bound = plan
        .actions
        .iter()
        .find(|action| action.execution != "self_sufficient");
    if let Some(action) = input_bound {
        bail!(
            "forward-annotation artifact apply requires only self-sufficient actions; action {} is {}",
            action.action_id,
            action.execution
        );
    }

    let review_by_id = loaded
        .artifact
        .reviews
        .iter()
        .map(|review| (review.action_id.clone(), review.decision.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut applied = Vec::new();
    let mut skipped = Vec::new();
    for action in loaded.artifact.actions {
        if let Some(review_decision) = review_by_id.get(&action.action_id) {
            let skip_reason = match review_decision.as_str() {
                "deferred" => Some("deferred_by_review"),
                "rejected" => Some("rejected_by_review"),
                _ => None,
            };
            if let Some(skip_reason) = skip_reason {
                skipped.push(NativeProjectForwardAnnotationBatchApplySkippedActionView {
                    action_id: action.action_id.clone(),
                    proposal_action: action.action.clone(),
                    reference: action.reference.clone(),
                    reason: action.reason.clone(),
                    skip_reason: skip_reason.to_string(),
                });
                continue;
            }
        }

        applied.push(execute_native_project_forward_annotation_action(
            root, action, None, None, None, None, None,
        )?);
    }

    let skipped_deferred_actions = skipped
        .iter()
        .filter(|entry| entry.skip_reason == "deferred_by_review")
        .count();
    let skipped_rejected_actions = skipped
        .iter()
        .filter(|entry| entry.skip_reason == "rejected_by_review")
        .count();

    Ok(NativeProjectForwardAnnotationArtifactApplyView {
        action: "apply_forward_annotation_proposal_artifact".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: project.root.display().to_string(),
        artifact_actions: plan.artifact_actions,
        applied_actions: applied.len(),
        skipped_deferred_actions,
        skipped_rejected_actions,
        applied,
        skipped,
    })
}
