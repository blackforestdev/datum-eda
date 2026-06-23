use super::{
    CommitDiff, DesignModel, EngineError, Operation, operation_application::apply_operation,
    variant::propagate_variant_population_to_component_instances,
};

pub(super) fn apply_operations(
    model: &mut DesignModel,
    operations: &[Operation],
    diff: &mut CommitDiff,
) -> Result<(), EngineError> {
    for operation in operations {
        apply_operation(model, operation, diff)?;
    }
    propagate_variant_population_to_component_instances(
        &mut model.variant_populations,
        &model.component_instances,
    );
    Ok(())
}
