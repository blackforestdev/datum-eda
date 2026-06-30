use super::command_project_schematic_symbol_library_materialization::PoolSymbolComponentBinding;
use super::*;
use eda_engine::substrate::{Operation, ProjectResolver, RevisionedRef};

pub(crate) fn component_instance_operation_for_pool_symbol(
    root: &Path,
    symbol_uuid: Uuid,
    binding: &PoolSymbolComponentBinding,
) -> Result<Option<Operation>> {
    let Some(part) = &binding.part else {
        return Ok(None);
    };
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let part_ref = revisioned_ref(&model, part.part_id)?;
    let component_instance_id = Uuid::new_v5(
        &model.project.project_id,
        format!(
            "datum-eda:component-instance:schematic:{}:{symbol_uuid}",
            binding.symbol_id
        )
        .as_bytes(),
    );
    Ok(Some(Operation::CreateComponentInstance {
        component_instance_id,
        component_instance: serde_json::json!({
            "uuid": component_instance_id,
            "object_revision": 0,
            "part_ref": part_ref,
            "placed_symbol_refs": [{
                "object_id": symbol_uuid,
                "object_revision": 0
            }],
            "placed_package_refs": [],
            "placed_symbol_roles": {
                (symbol_uuid.to_string()): {
                    "role": "primary"
                }
            },
            "placed_package_roles": {}
        }),
    }))
}

fn revisioned_ref(
    model: &eda_engine::substrate::DesignModel,
    object_id: Uuid,
) -> Result<RevisionedRef> {
    let object = model
        .objects
        .get(&object_id)
        .with_context(|| format!("component instance target object {object_id} was not found"))?;
    Ok(RevisionedRef {
        object_id,
        object_revision: object.object_revision,
    })
}
