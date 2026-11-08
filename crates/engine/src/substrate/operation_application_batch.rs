use super::{
    CommitDiff, DesignModel, EngineError, Operation, operation_application::apply_operation,
    variant::propagate_variant_population_to_component_instances,
};

pub(super) fn apply_operations(
    model: &mut DesignModel,
    operations: &[Operation],
    diff: &mut CommitDiff,
) -> Result<(), EngineError> {
    if model
        .source_shards
        .iter()
        .any(|s| s.kind == super::SourceShardKind::SchematicRoot)
        || !model.electrical_identities.is_empty()
        || operations
            .iter()
            .any(|op| super::electrical_identity_store::write(op).is_some())
    {
        let mut candidate = model.clone();
        for (index, operation) in operations.iter().enumerate() {
            if !super::operation_application_component_instance::apply_component_instance_operation_with_basis(
                &mut candidate, diff, operation, Some(super::component_instance::pool_history::PoolHistoryBasis {model, operations, applied_operations: &operations[..index]}),
            )? {
                apply_operation(&mut candidate, operation, diff)?;
            }
        }
        super::electrical_identity_validation::validate_operations(&candidate, operations)?;
        *model = candidate;
    } else {
        for operation in operations {
            apply_operation(model, operation, diff)?;
        }
    }
    propagate_variant_population_to_component_instances(
        &mut model.variant_populations,
        &model.component_instances,
        &mut model.diagnostics,
    );
    Ok(())
}
