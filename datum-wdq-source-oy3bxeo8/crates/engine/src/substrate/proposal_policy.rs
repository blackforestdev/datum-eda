use super::{CommitSource, Operation, OperationBatch, ProposalApplyBlocker};

pub(super) fn direct_commit_proposal_policy_blockers(
    batch: &OperationBatch,
) -> Vec<ProposalApplyBlocker> {
    if !matches!(
        batch.provenance.source,
        CommitSource::Tool | CommitSource::Assistant
    ) {
        return Vec::new();
    }

    batch
        .operations
        .iter()
        .filter_map(|operation| {
            let (code, family) = proposal_required_operation(operation)?;
            Some(ProposalApplyBlocker {
                code: code.to_string(),
                message: format!(
                    "{family} operations from {:?} provenance must be authored as proposals before commit",
                    batch.provenance.source
                ),
            })
        })
        .collect()
}

pub(super) fn format_proposal_policy_blockers(blockers: &[ProposalApplyBlocker]) -> String {
    blockers
        .iter()
        .map(|blocker| format!("{}: {}", blocker.code, blocker.message))
        .collect::<Vec<_>>()
        .join("; ")
}

pub(super) fn proposal_batch_policy_blockers(batch: &OperationBatch) -> Vec<ProposalApplyBlocker> {
    let mut blockers = Vec::new();
    for operation in &batch.operations {
        if matches!(
            operation,
            Operation::CreateProposalMetadata { .. }
                | Operation::SetProposalMetadata { .. }
                | Operation::DeleteProposalMetadata { .. }
        ) {
            blockers.push(ProposalApplyBlocker {
                code: "proposal_metadata_operation_forbidden".to_string(),
                message:
                    "proposal lifecycle metadata is owned by review/apply, not proposal batches"
                        .to_string(),
            });
        }
    }
    blockers
}

fn proposal_required_operation(operation: &Operation) -> Option<(&'static str, &'static str)> {
    if let Some(family) = proposal_required_production_operation_family(operation) {
        return Some((
            "proposal_required_for_automated_production_operation",
            family,
        ));
    }
    if let Some(family) = proposal_required_cross_domain_identity_operation_family(operation) {
        return Some((
            "proposal_required_for_automated_cross_domain_identity_operation",
            family,
        ));
    }
    if let Some(family) = proposal_required_library_operation_family(operation) {
        return Some(("proposal_required_for_automated_library_operation", family));
    }
    proposal_required_generated_evidence_operation_family(operation).map(|family| {
        (
            "proposal_required_for_automated_generated_evidence_operation",
            family,
        )
    })
}

fn proposal_required_production_operation_family(operation: &Operation) -> Option<&'static str> {
    match operation {
        Operation::CreateManufacturingPlan { .. }
        | Operation::SetManufacturingPlan { .. }
        | Operation::DeleteManufacturingPlan { .. } => Some("manufacturing_plan"),
        Operation::CreatePanelProjection { .. }
        | Operation::SetPanelProjection { .. }
        | Operation::DeletePanelProjection { .. } => Some("panel_projection"),
        Operation::CreateOutputJob { .. }
        | Operation::SetOutputJob { .. }
        | Operation::DeleteOutputJob { .. } => Some("output_job"),
        _ => None,
    }
}

fn proposal_required_cross_domain_identity_operation_family(
    operation: &Operation,
) -> Option<&'static str> {
    match operation {
        Operation::CreateComponentInstance { .. }
        | Operation::SetComponentInstance { .. }
        | Operation::DeleteComponentInstance { .. } => Some("component_instance"),
        Operation::CreateRelationship { .. }
        | Operation::SetRelationship { .. }
        | Operation::DeleteRelationship { .. } => Some("relationship"),
        Operation::CreateVariantOverlay { .. }
        | Operation::SetVariantOverlay { .. }
        | Operation::DeleteVariantOverlay { .. } => Some("variant_overlay"),
        _ => None,
    }
}

fn proposal_required_library_operation_family(operation: &Operation) -> Option<&'static str> {
    match operation {
        Operation::CreatePoolPackage { .. } | Operation::DeletePoolPackage { .. } => {
            Some("pool_package")
        }
        Operation::CreatePoolPadstack { .. } | Operation::DeletePoolPadstack { .. } => {
            Some("pool_padstack")
        }
        Operation::CreatePoolLibraryObject { .. }
        | Operation::SetPoolLibraryObject { .. }
        | Operation::DeletePoolLibraryObject { .. } => Some("pool_library_object"),
        Operation::AttachPoolPartModel { .. } | Operation::DetachPoolPartModel { .. } => {
            Some("pool_part_model_attachment")
        }
        _ => None,
    }
}

fn proposal_required_generated_evidence_operation_family(
    operation: &Operation,
) -> Option<&'static str> {
    match operation {
        Operation::SetOutputJobRun { .. } | Operation::DeleteOutputJobRun { .. } => {
            Some("output_job_run")
        }
        Operation::SetArtifactRun { .. } | Operation::DeleteArtifactRun { .. } => {
            Some("artifact_run")
        }
        Operation::SetCheckRun { .. } | Operation::DeleteCheckRun { .. } => Some("check_run"),
        Operation::SetArtifactMetadata { .. } | Operation::DeleteArtifactMetadata { .. } => {
            Some("artifact_metadata")
        }
        Operation::SetZoneFill { .. } | Operation::DeleteZoneFill { .. } => Some("zone_fill"),
        _ => None,
    }
}
