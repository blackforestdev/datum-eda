use std::path::Path;

use super::{
    EngineError, Operation, OperationBatch, SourceShardKind,
    journal::{StagedShardWrite, stage_new_shard_write},
};

pub(super) fn maybe_stage_schematic_definition_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    match operation {
        Operation::CreateSchematicDefinition {
            relative_path,
            definition,
            ..
        } => {
            let relative_path = format!("schematic/{relative_path}");
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::SchematicDefinition,
                &relative_path,
                definition,
            )?);
        }
        Operation::DeleteSchematicDefinition { relative_path, .. } => {
            let relative_path = format!("schematic/{relative_path}");
            staged.push(StagedShardWrite {
                destination: project_root.join(&relative_path),
                staged: None,
                kind: SourceShardKind::SchematicDefinition,
                relative_path,
                content_hash: String::new(),
                delete: true,
            });
        }
        _ => {}
    }
    Ok(())
}
