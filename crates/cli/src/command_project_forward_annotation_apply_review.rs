use super::*;

pub(crate) fn apply_native_project_forward_annotation_action(
    root: &Path,
    action_id: &str,
    package_uuid: Option<Uuid>,
    part_uuid: Option<Uuid>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
    layer: Option<i32>,
) -> Result<NativeProjectForwardAnnotationApplyReportView> {
    let proposal = query_native_project_forward_annotation_proposal(root)?;
    let action = proposal
        .actions
        .into_iter()
        .find(|action| action.action_id == action_id)
        .ok_or_else(|| {
            anyhow::anyhow!("forward-annotation proposal action not found: {action_id}")
        })?;

    execute_native_project_forward_annotation_action(
        root,
        action,
        package_uuid,
        part_uuid,
        x_nm,
        y_nm,
        layer,
    )
}

pub(crate) fn execute_native_project_forward_annotation_action(
    root: &Path,
    action: NativeProjectForwardAnnotationProposalActionView,
    package_uuid: Option<Uuid>,
    part_uuid: Option<Uuid>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
    layer: Option<i32>,
) -> Result<NativeProjectForwardAnnotationApplyReportView> {
    let component_report = match (action.action.as_str(), action.reason.as_str()) {
        ("remove_component", "board_component_missing_in_schematic") => {
            let component_uuid = Uuid::parse_str(
                action
                    .component_uuid
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing component UUID"))?,
            )
            .context("invalid component UUID in forward-annotation proposal")?;
            delete_native_project_board_component(root, component_uuid)?
        }
        ("update_component", "value_mismatch") => {
            let component_uuid = Uuid::parse_str(
                action
                    .component_uuid
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing component UUID"))?,
            )
            .context("invalid component UUID in forward-annotation proposal")?;
            set_native_project_board_component_value(
                root,
                component_uuid,
                action
                    .schematic_value
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing schematic value"))?,
            )?
        }
        ("add_component", _) => {
            let package_uuid = package_uuid.ok_or_else(|| {
                anyhow::anyhow!("forward-annotation add_component apply requires --package <uuid>")
            })?;
            let x_nm = x_nm.ok_or_else(|| {
                anyhow::anyhow!("forward-annotation add_component apply requires --x-nm <i64>")
            })?;
            let y_nm = y_nm.ok_or_else(|| {
                anyhow::anyhow!("forward-annotation add_component apply requires --y-nm <i64>")
            })?;
            let layer = layer.ok_or_else(|| {
                anyhow::anyhow!("forward-annotation add_component apply requires --layer <i32>")
            })?;
            let resolved_part_uuid = match (part_uuid, action.schematic_part_uuid.as_deref()) {
                (Some(part_uuid), _) => part_uuid,
                (None, Some(part_uuid)) => Uuid::parse_str(part_uuid)
                    .context("invalid schematic part UUID in forward-annotation proposal")?,
                (None, None) => {
                    bail!(
                        "forward-annotation add_component apply requires --part <uuid> when the proposal does not carry a resolved schematic part"
                    )
                }
            };
            place_native_project_board_component(
                root,
                resolved_part_uuid,
                package_uuid,
                action.reference.clone(),
                action
                    .schematic_value
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing schematic value"))?,
                Point::new(x_nm, y_nm),
                layer,
            )?
        }
        ("update_component", "part_mismatch") => {
            let component_uuid = Uuid::parse_str(
                action
                    .component_uuid
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing component UUID"))?,
            )
            .context("invalid component UUID in forward-annotation proposal")?;
            let package_uuid = package_uuid.ok_or_else(|| {
                anyhow::anyhow!("forward-annotation part_mismatch apply requires --package <uuid>")
            })?;
            let resolved_part_uuid = match (part_uuid, action.schematic_part_uuid.as_deref()) {
                (Some(part_uuid), _) => part_uuid,
                (None, Some(part_uuid)) => Uuid::parse_str(part_uuid)
                    .context("invalid schematic part UUID in forward-annotation proposal")?,
                (None, None) => {
                    bail!(
                        "forward-annotation part_mismatch apply requires --part <uuid> when the proposal does not carry a resolved schematic part"
                    )
                }
            };
            let _ = set_native_project_board_component_package(root, component_uuid, package_uuid)?;
            set_native_project_board_component_part(root, component_uuid, resolved_part_uuid)?
        }
        _ => bail!(
            "forward-annotation apply is not supported for {} reason={}",
            action.action,
            action.reason
        ),
    };

    Ok(NativeProjectForwardAnnotationApplyReportView {
        action: "apply_forward_annotation_action".to_string(),
        action_id: action.action_id,
        proposal_action: action.action,
        reason: action.reason,
        component_report,
    })
}

pub(crate) fn apply_native_project_forward_annotation_reviewed(
    root: &Path,
) -> Result<NativeProjectForwardAnnotationBatchApplyReportView> {
    let proposal = query_native_project_forward_annotation_proposal(root)?;
    let project = load_native_project(root)?;
    let review = project.manifest.forward_annotation_review;
    let mut applied = Vec::new();
    let mut skipped = Vec::new();

    for action in proposal.actions {
        if let Some(review_record) = review.get(&action.action_id) {
            let skip_reason = match review_record.decision.as_str() {
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

        match (action.action.as_str(), action.reason.as_str()) {
            ("remove_component", "board_component_missing_in_schematic")
            | ("update_component", "value_mismatch") => {
                applied.push(execute_native_project_forward_annotation_action(
                    root, action, None, None, None, None, None,
                )?);
            }
            ("add_component", _) | ("update_component", "part_mismatch") => {
                skipped.push(NativeProjectForwardAnnotationBatchApplySkippedActionView {
                    action_id: action.action_id.clone(),
                    proposal_action: action.action.clone(),
                    reference: action.reference.clone(),
                    reason: action.reason.clone(),
                    skip_reason: "requires_explicit_input".to_string(),
                });
            }
            _ => {
                skipped.push(NativeProjectForwardAnnotationBatchApplySkippedActionView {
                    action_id: action.action_id.clone(),
                    proposal_action: action.action.clone(),
                    reference: action.reference.clone(),
                    reason: action.reason.clone(),
                    skip_reason: "unsupported_action".to_string(),
                });
            }
        }
    }

    let skipped_deferred_actions = skipped
        .iter()
        .filter(|entry| entry.skip_reason == "deferred_by_review")
        .count();
    let skipped_rejected_actions = skipped
        .iter()
        .filter(|entry| entry.skip_reason == "rejected_by_review")
        .count();
    let skipped_requires_input_actions = skipped
        .iter()
        .filter(|entry| entry.skip_reason == "requires_explicit_input")
        .count();

    Ok(NativeProjectForwardAnnotationBatchApplyReportView {
        action: "apply_forward_annotation_reviewed".to_string(),
        domain: "native_project",
        proposal_actions: applied.len() + skipped.len(),
        applied_actions: applied.len(),
        skipped_deferred_actions,
        skipped_rejected_actions,
        skipped_requires_input_actions,
        applied,
        skipped,
    })
}

pub(crate) fn query_native_project_forward_annotation_review(
    root: &Path,
) -> Result<NativeProjectForwardAnnotationReviewView> {
    let project = load_native_project(root)?;
    let mut actions = project
        .manifest
        .forward_annotation_review
        .values()
        .map(|record| NativeProjectForwardAnnotationReviewActionView {
            action_id: record.action_id.clone(),
            decision: record.decision.clone(),
            proposal_action: record.proposal_action.clone(),
            reference: record.reference.clone(),
            reason: record.reason.clone(),
        })
        .collect::<Vec<_>>();
    actions.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.proposal_action.cmp(&b.proposal_action))
            .then_with(|| a.reason.cmp(&b.reason))
            .then_with(|| a.action_id.cmp(&b.action_id))
    });
    let deferred_actions = actions
        .iter()
        .filter(|action| action.decision == "deferred")
        .count();
    let rejected_actions = actions
        .iter()
        .filter(|action| action.decision == "rejected")
        .count();
    Ok(NativeProjectForwardAnnotationReviewView {
        domain: "native_project",
        total_reviews: actions.len(),
        deferred_actions,
        rejected_actions,
        actions,
    })
}

pub(crate) fn record_native_project_forward_annotation_review(
    root: &Path,
    action_id: &str,
    decision: &str,
) -> Result<NativeProjectForwardAnnotationReviewReportView> {
    if decision != "deferred" && decision != "rejected" {
        bail!("unsupported forward-annotation review decision: {decision}");
    }

    let proposal = query_native_project_forward_annotation_proposal(root)?;
    let action = proposal
        .actions
        .into_iter()
        .find(|action| action.action_id == action_id)
        .ok_or_else(|| {
            anyhow::anyhow!("forward-annotation proposal action not found: {action_id}")
        })?;

    let mut project = load_native_project(root)?;
    project.manifest.forward_annotation_review.insert(
        action.action_id.clone(),
        NativeForwardAnnotationReviewRecord {
            action_id: action.action_id.clone(),
            decision: decision.to_string(),
            proposal_action: action.action.clone(),
            reference: action.reference.clone(),
            reason: action.reason.clone(),
        },
    );
    write_canonical_json(&project.root.join("project.json"), &project.manifest)?;

    Ok(NativeProjectForwardAnnotationReviewReportView {
        action: format!("{decision}_forward_annotation_action"),
        action_id: action.action_id,
        decision: decision.to_string(),
        proposal_action: action.action,
        reference: action.reference,
        reason: action.reason,
    })
}

pub(crate) fn clear_native_project_forward_annotation_review(
    root: &Path,
    action_id: &str,
) -> Result<NativeProjectForwardAnnotationReviewReportView> {
    let mut project = load_native_project(root)?;
    let cleared = project
        .manifest
        .forward_annotation_review
        .remove(action_id)
        .ok_or_else(|| {
            anyhow::anyhow!("forward-annotation review action not found: {action_id}")
        })?;
    write_canonical_json(&project.root.join("project.json"), &project.manifest)?;
    Ok(NativeProjectForwardAnnotationReviewReportView {
        action: "clear_forward_annotation_action_review".to_string(),
        action_id: cleared.action_id,
        decision: cleared.decision,
        proposal_action: cleared.proposal_action,
        reference: cleared.reference,
        reason: cleared.reason,
    })
}
