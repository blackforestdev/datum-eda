use super::{
    CommitDiff, DesignModel, EngineError, Operation,
    forward_annotation_review_journal_ops::apply_forward_annotation_review_model_operation,
    generated_evidence_journal_ops::apply_generated_evidence_model_operation,
    operation_application_component_instance::apply_component_instance_operation,
    operation_application_object_revision::bump_existing_object,
    operation_application_schematic_definition::apply_schematic_definition_operation,
    operation_application_schematic_instance::apply_schematic_instance_operation,
    operation_application_schematic_waiver::apply_schematic_disposition_operation,
    proposal_journal_ops::apply_proposal_model_operation,
};

pub(super) fn apply_pre_match_operation(
    model: &mut DesignModel,
    operation: &Operation,
    diff: &mut CommitDiff,
) -> Result<bool, EngineError> {
    if apply_project_root_operation(model, operation, diff)? {
        return Ok(true);
    }
    if apply_component_instance_operation(model, diff, operation)? {
        return Ok(true);
    }
    if apply_schematic_disposition_operation(model, diff, operation)? {
        return Ok(true);
    }
    if apply_schematic_definition_operation(model, diff, operation)? {
        return Ok(true);
    }
    if apply_schematic_instance_operation(model, diff, operation)? {
        return Ok(true);
    }
    if apply_proposal_model_operation(model, operation)? {
        return Ok(true);
    }
    if apply_forward_annotation_review_model_operation(model, operation)? {
        return Ok(true);
    }
    apply_generated_evidence_model_operation(model, operation)
}

fn apply_project_root_operation(
    model: &mut DesignModel,
    operation: &Operation,
    diff: &mut CommitDiff,
) -> Result<bool, EngineError> {
    let project_id = match operation {
        Operation::SetProjectName { project_id, name } => {
            model.project.name.clone_from(name);
            project_id
        }
        Operation::InitializeProjectDisplayUnits {
            project_id,
            profile,
            receipt,
        } => {
            model.project.project_display_units = Some(profile.clone());
            model.project.project_units_seed_receipt = Some(receipt.clone());
            project_id
        }
        Operation::SetProjectDisplayUnits {
            project_id,
            profile,
        } => {
            model.project.project_display_units = Some(profile.clone());
            project_id
        }
        Operation::RemoveProjectDisplayUnits { project_id } => {
            model.project.project_display_units = None;
            model.project.project_units_seed_receipt = None;
            project_id
        }
        _ => return Ok(false),
    };
    bump_existing_object(&mut model.objects, *project_id, Some(diff))?;
    Ok(true)
}
