use super::command_project_schematic_symbol_library_materialization::PoolSymbolComponentBinding;
use super::*;
use eda_engine::api::native_write::component_instances::build_placed_symbol_component_instance_op;
use eda_engine::substrate::{Operation, ProjectResolver};

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
    Ok(Some(build_placed_symbol_component_instance_op(
        &model,
        symbol_uuid,
        binding.symbol_id,
        part.part_id,
    )?))
}
