use std::path::Path;

use super::{
    EngineError, Operation, OperationBatch, SourceShardKind,
    journal::{StagedShardWrite, stage_new_shard_write},
};

pub(super) fn maybe_stage_import_map_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    match operation {
        Operation::CreateImportMapShard {
            relative_path,
            shard,
        } => staged.push(stage_new_shard_write(
            project_root,
            batch,
            SourceShardKind::ImportMap,
            relative_path,
            shard,
        )?),
        Operation::DeleteImportMapShard { relative_path, .. } => {
            staged.push(delete_import_map_shard(project_root, relative_path));
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn apply_import_map_shard_operation(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    if shard_kind != &SourceShardKind::ImportMap {
        return Ok(false);
    }
    match operation {
        Operation::CreateImportMapShard { shard, .. } => {
            *value = shard.clone();
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn inverse_import_map_operation(
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) {
    match operation {
        Operation::CreateImportMapShard {
            relative_path,
            shard,
        } => inverse_operations.push(Operation::DeleteImportMapShard {
            relative_path: relative_path.clone(),
            shard: shard.clone(),
        }),
        Operation::DeleteImportMapShard {
            relative_path,
            shard,
        } => inverse_operations.push(Operation::CreateImportMapShard {
            relative_path: relative_path.clone(),
            shard: shard.clone(),
        }),
        _ => {}
    }
}

fn delete_import_map_shard(project_root: &Path, relative_path: &str) -> StagedShardWrite {
    StagedShardWrite {
        destination: project_root.join(relative_path),
        staged: None,
        kind: SourceShardKind::ImportMap,
        relative_path: relative_path.to_string(),
        content_hash: String::new(),
        delete: true,
    }
}
