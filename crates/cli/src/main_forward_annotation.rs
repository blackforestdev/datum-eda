use super::*;

pub(super) fn render_native_forward_annotation_audit_text(
    report: &NativeProjectForwardAnnotationAuditView,
) -> String {
    let mut lines = vec![
        format!("schematic_symbol_count: {}", report.schematic_symbol_count),
        format!("board_component_count: {}", report.board_component_count),
        format!("matched_count: {}", report.matched_count),
        format!(
            "unresolved_symbol_count: {}",
            report.unresolved_symbol_count
        ),
        format!("missing_on_board_count: {}", report.missing_on_board.len()),
        format!(
            "orphaned_on_board_count: {}",
            report.orphaned_on_board.len()
        ),
        format!("value_mismatch_count: {}", report.value_mismatches.len()),
        format!("part_mismatch_count: {}", report.part_mismatches.len()),
    ];
    if !report.missing_on_board.is_empty() {
        lines.push("missing_on_board:".to_string());
        for entry in &report.missing_on_board {
            lines.push(format!(
                "  {} value={} part_uuid={}",
                entry.reference,
                entry.value,
                entry.part_uuid.as_deref().unwrap_or("none")
            ));
        }
    }
    if !report.orphaned_on_board.is_empty() {
        lines.push("orphaned_on_board:".to_string());
        for entry in &report.orphaned_on_board {
            lines.push(format!(
                "  {} value={} part_uuid={}",
                entry.reference, entry.value, entry.part_uuid
            ));
        }
    }
    if !report.value_mismatches.is_empty() {
        lines.push("value_mismatches:".to_string());
        for entry in &report.value_mismatches {
            lines.push(format!(
                "  {} schematic={} board={}",
                entry.reference, entry.schematic_value, entry.board_value
            ));
        }
    }
    if !report.part_mismatches.is_empty() {
        lines.push("part_mismatches:".to_string());
        for entry in &report.part_mismatches {
            lines.push(format!(
                "  {} schematic_part_uuid={} board_part_uuid={}",
                entry.reference, entry.schematic_part_uuid, entry.board_part_uuid
            ));
        }
    }
    lines.join("\n")
}

pub(super) fn render_native_forward_annotation_proposal_text(
    report: &NativeProjectForwardAnnotationProposalView,
) -> String {
    let mut lines = vec![
        format!("total_actions: {}", report.total_actions),
        format!("add_component_actions: {}", report.add_component_actions),
        format!(
            "remove_component_actions: {}",
            report.remove_component_actions
        ),
        format!(
            "update_component_actions: {}",
            report.update_component_actions
        ),
    ];
    if !report.actions.is_empty() {
        lines.push("actions:".to_string());
        for action in &report.actions {
            lines.push(format!(
                "  {} {} id={} reason={}",
                action.action, action.reference, action.action_id, action.reason
            ));
        }
    }
    lines.join("\n")
}

pub(super) fn render_native_forward_annotation_review_text(
    report: &NativeProjectForwardAnnotationReviewView,
) -> String {
    let mut lines = vec![
        format!("domain: {}", report.domain),
        format!("total_reviews: {}", report.total_reviews),
        format!("deferred_actions: {}", report.deferred_actions),
        format!("rejected_actions: {}", report.rejected_actions),
    ];
    for action in &report.actions {
        lines.push(String::new());
        lines.push(format!("action_id: {}", action.action_id));
        lines.push(format!("decision: {}", action.decision));
        lines.push(format!("proposal_action: {}", action.proposal_action));
        lines.push(format!("reference: {}", action.reference));
        lines.push(format!("reason: {}", action.reason));
    }
    lines.join("\n")
}

pub(super) fn render_native_forward_annotation_review_report_text(
    report: &NativeProjectForwardAnnotationReviewReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("action_id: {}", report.action_id),
        format!("decision: {}", report.decision),
        format!("proposal_action: {}", report.proposal_action),
        format!("reference: {}", report.reference),
        format!("reason: {}", report.reason),
    ]
    .join("\n")
}

pub(super) fn render_native_forward_annotation_apply_text(
    report: &NativeProjectForwardAnnotationApplyReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("action_id: {}", report.action_id),
        format!("proposal_action: {}", report.proposal_action),
        format!("reason: {}", report.reason),
        format!("component_uuid: {}", report.component_report.component_uuid),
        format!("part_uuid: {}", report.component_report.part_uuid),
        format!("package_uuid: {}", report.component_report.package_uuid),
        format!("reference: {}", report.component_report.reference),
        format!("value: {}", report.component_report.value),
        format!("x_nm: {}", report.component_report.x_nm),
        format!("y_nm: {}", report.component_report.y_nm),
        format!("rotation_deg: {}", report.component_report.rotation_deg),
        format!("layer: {}", report.component_report.layer),
        format!("locked: {}", report.component_report.locked),
    ]
    .join("\n")
}

pub(super) fn render_native_forward_annotation_batch_apply_text(
    report: &NativeProjectForwardAnnotationBatchApplyReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("domain: {}", report.domain),
        format!("proposal_actions: {}", report.proposal_actions),
        format!("applied_actions: {}", report.applied_actions),
        format!(
            "skipped_deferred_actions: {}",
            report.skipped_deferred_actions
        ),
        format!(
            "skipped_rejected_actions: {}",
            report.skipped_rejected_actions
        ),
        format!(
            "skipped_requires_input_actions: {}",
            report.skipped_requires_input_actions
        ),
    ];
    for applied in &report.applied {
        lines.push(String::new());
        lines.push(format!("applied_action_id: {}", applied.action_id));
        lines.push(format!("proposal_action: {}", applied.proposal_action));
        lines.push(format!("reference: {}", applied.component_report.reference));
        lines.push(format!("reason: {}", applied.reason));
    }
    for skipped in &report.skipped {
        lines.push(String::new());
        lines.push(format!("skipped_action_id: {}", skipped.action_id));
        lines.push(format!("proposal_action: {}", skipped.proposal_action));
        lines.push(format!("reference: {}", skipped.reference));
        lines.push(format!("reason: {}", skipped.reason));
        lines.push(format!("skip_reason: {}", skipped.skip_reason));
    }
    lines.join("\n")
}

pub(super) fn render_native_forward_annotation_export_text(
    report: &NativeProjectForwardAnnotationExportReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("version: {}", report.version),
        format!("project_uuid: {}", report.project_uuid),
        format!("actions: {}", report.actions),
        format!("reviews: {}", report.reviews),
    ]
    .join("\n")
}

pub(super) fn render_native_forward_annotation_artifact_inspection_text(
    report: &NativeProjectForwardAnnotationArtifactInspectionView,
) -> String {
    [
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("source_version: {}", report.source_version),
        format!("version: {}", report.version),
        format!("migration_applied: {}", report.migration_applied),
        format!("project_uuid: {}", report.project_uuid),
        format!("project_name: {}", report.project_name),
        format!("actions: {}", report.actions),
        format!("reviews: {}", report.reviews),
        format!("add_component_actions: {}", report.add_component_actions),
        format!(
            "remove_component_actions: {}",
            report.remove_component_actions
        ),
        format!(
            "update_component_actions: {}",
            report.update_component_actions
        ),
        format!("deferred_reviews: {}", report.deferred_reviews),
        format!("rejected_reviews: {}", report.rejected_reviews),
    ]
    .join("\n")
}

pub(super) fn render_native_forward_annotation_artifact_validation_text(
    report: &NativeProjectForwardAnnotationArtifactValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("source_version: {}", report.source_version),
        format!("version: {}", report.version),
        format!("migration_applied: {}", report.migration_applied),
        format!("actions: {}", report.actions),
        format!("reviews: {}", report.reviews),
        format!("matches_expected: {}", report.matches_expected),
        format!("canonical_bytes_match: {}", report.canonical_bytes_match),
    ]
    .join("\n")
}

pub(super) fn render_native_forward_annotation_artifact_comparison_text(
    report: &NativeProjectForwardAnnotationArtifactComparisonView,
) -> String {
    let mut lines = vec![
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("kind: {}", report.kind),
        format!("artifact_version: {}", report.artifact_version),
        format!("current_project_uuid: {}", report.current_project_uuid),
        format!("artifact_project_uuid: {}", report.artifact_project_uuid),
        format!("artifact_actions: {}", report.artifact_actions),
        format!("applicable_actions: {}", report.applicable_actions),
        format!("drifted_actions: {}", report.drifted_actions),
        format!("stale_actions: {}", report.stale_actions),
    ];
    for action in &report.actions {
        lines.push(String::new());
        lines.push(format!("action_id: {}", action.action_id));
        lines.push(format!("proposal_action: {}", action.proposal_action));
        lines.push(format!("reference: {}", action.reference));
        lines.push(format!("reason: {}", action.reason));
        lines.push(format!("status: {}", action.status));
        if let Some(review_decision) = &action.review_decision {
            lines.push(format!("review_decision: {}", review_decision));
        }
    }
    lines.join("\n")
}

pub(super) fn render_native_forward_annotation_artifact_filter_text(
    report: &NativeProjectForwardAnnotationArtifactFilterView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("input_artifact_path: {}", report.input_artifact_path),
        format!("output_artifact_path: {}", report.output_artifact_path),
        format!("project_root: {}", report.project_root),
        format!("kind: {}", report.kind),
        format!("version: {}", report.version),
        format!("artifact_actions: {}", report.artifact_actions),
        format!("applicable_actions: {}", report.applicable_actions),
        format!("filtered_reviews: {}", report.filtered_reviews),
    ]
    .join("\n")
}

pub(super) fn render_native_forward_annotation_artifact_apply_plan_text(
    report: &NativeProjectForwardAnnotationArtifactApplyPlanView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("kind: {}", report.kind),
        format!("artifact_version: {}", report.artifact_version),
        format!("artifact_actions: {}", report.artifact_actions),
        format!(
            "self_sufficient_actions: {}",
            report.self_sufficient_actions
        ),
        format!("requires_input_actions: {}", report.requires_input_actions),
        format!("not_applicable_actions: {}", report.not_applicable_actions),
    ];
    for action in &report.actions {
        lines.push(String::new());
        lines.push(format!("action_id: {}", action.action_id));
        lines.push(format!("proposal_action: {}", action.proposal_action));
        lines.push(format!("reference: {}", action.reference));
        lines.push(format!("reason: {}", action.reason));
        lines.push(format!("applicability: {}", action.applicability));
        lines.push(format!("execution: {}", action.execution));
        if let Some(review_decision) = &action.review_decision {
            lines.push(format!("review_decision: {}", review_decision));
        }
        for required_input in &action.required_inputs {
            lines.push(format!("required_input: {}", required_input));
        }
    }
    lines.join("\n")
}

pub(super) fn render_native_forward_annotation_artifact_apply_text(
    report: &NativeProjectForwardAnnotationArtifactApplyView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("artifact_actions: {}", report.artifact_actions),
        format!("applied_actions: {}", report.applied_actions),
        format!(
            "skipped_deferred_actions: {}",
            report.skipped_deferred_actions
        ),
        format!(
            "skipped_rejected_actions: {}",
            report.skipped_rejected_actions
        ),
    ];
    for applied in &report.applied {
        lines.push(String::new());
        lines.push(format!("applied_action_id: {}", applied.action_id));
        lines.push(format!("proposal_action: {}", applied.proposal_action));
        lines.push(format!("reason: {}", applied.reason));
        lines.push(format!(
            "component_reference: {}",
            applied.component_report.reference
        ));
    }
    for skipped in &report.skipped {
        lines.push(String::new());
        lines.push(format!("skipped_action_id: {}", skipped.action_id));
        lines.push(format!("proposal_action: {}", skipped.proposal_action));
        lines.push(format!("reference: {}", skipped.reference));
        lines.push(format!("reason: {}", skipped.reason));
        lines.push(format!("skip_reason: {}", skipped.skip_reason));
    }
    lines.join("\n")
}

pub(super) fn render_native_forward_annotation_artifact_review_import_text(
    report: &NativeProjectForwardAnnotationArtifactReviewImportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("total_artifact_reviews: {}", report.total_artifact_reviews),
        format!("imported_reviews: {}", report.imported_reviews),
        format!(
            "skipped_missing_live_actions: {}",
            report.skipped_missing_live_actions
        ),
    ]
    .join("\n")
}

pub(super) fn render_native_forward_annotation_artifact_review_replace_text(
    report: &NativeProjectForwardAnnotationArtifactReviewReplaceView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("total_artifact_reviews: {}", report.total_artifact_reviews),
        format!("replaced_reviews: {}", report.replaced_reviews),
        format!(
            "removed_existing_reviews: {}",
            report.removed_existing_reviews
        ),
        format!(
            "skipped_missing_live_actions: {}",
            report.skipped_missing_live_actions
        ),
    ]
    .join("\n")
}
