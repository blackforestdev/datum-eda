use super::{
    CommitDiff, DesignModel, EngineError, Operation,
    forward_annotation_review_journal_ops::apply_forward_annotation_review_model_operation,
    generated_evidence_journal_ops::apply_generated_evidence_model_operation,
    operation_application_component_instance::apply_component_instance_operation,
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
