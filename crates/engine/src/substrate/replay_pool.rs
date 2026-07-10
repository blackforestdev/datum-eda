use std::path::Path;

use super::pool_journal_ops::apply_pool_shard_operation;
use super::source_shard_ref_builders::source_shard_ref_for_value;
use super::{EngineError, SourceShardKind, SourceShardRef, TransactionRecord, read_json_value};

pub(super) fn replay_pool_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::Pool)
    {
        if !shard.path.exists() {
            continue;
        }
        let Ok(value) = read_json_value(&shard.path) else {
            continue;
        };
        values.push((
            shard.relative_path.clone(),
            value,
            false,
            Some(shard.clone()),
        ));
    }
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                super::Operation::CreatePoolPackage {
                    relative_path,
                    package,
                    ..
                } => upsert_replay_value(&mut values, relative_path, package.clone()),
                super::Operation::CreatePoolPadstack {
                    relative_path,
                    padstack,
                    ..
                } => upsert_replay_value(&mut values, relative_path, padstack.clone()),
                super::Operation::CreatePoolLibraryObject {
                    relative_path,
                    object,
                    ..
                } => upsert_replay_value(&mut values, relative_path, object.clone()),
                super::Operation::SetPoolLibraryObject {
                    relative_path,
                    object,
                    ..
                } => upsert_replay_value(&mut values, relative_path, object.clone()),
                super::Operation::DeletePoolPackage { relative_path, .. }
                | super::Operation::DeletePoolPadstack { relative_path, .. }
                | super::Operation::DeletePoolLibraryObject { relative_path, .. } => {
                    values.retain(|(existing, _, _, _)| existing != relative_path);
                }
                super::Operation::AttachPoolPartModel { relative_path, .. }
                | super::Operation::DetachPoolPartModel { relative_path, .. } => {
                    apply_pool_operation_to_replay_value(&mut values, relative_path, operation)?;
                }
                _ => {
                    for (_, value, touched, original) in &mut values {
                        if !*touched && original.is_some() {
                            continue;
                        }
                        if apply_pool_shard_operation(&SourceShardKind::Pool, value, operation)? {
                            *touched = true;
                            *original = None;
                        }
                    }
                }
            }
        }
    }
    shards.retain(|shard| shard.kind != SourceShardKind::Pool);
    for (relative_path, value, touched, original) in values {
        if !touched {
            if let Some(shard) = original {
                shards.push(shard);
            }
            continue;
        }
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::Pool,
            relative_path,
            &value,
        )?);
    }
    Ok(())
}

fn apply_pool_operation_to_replay_value(
    values: &mut [(String, serde_json::Value, bool, Option<SourceShardRef>)],
    relative_path: &str,
    operation: &super::Operation,
) -> Result<(), EngineError> {
    if let Some((_, value, touched, original)) = values
        .iter_mut()
        .find(|(existing_path, _, _, _)| existing_path == relative_path)
    {
        if !*touched && original.is_some() {
            return Ok(());
        }
        if apply_pool_shard_operation(&SourceShardKind::Pool, value, operation)? {
            *touched = true;
            *original = None;
        }
    }
    Ok(())
}

fn upsert_replay_value(
    values: &mut Vec<(String, serde_json::Value, bool, Option<SourceShardRef>)>,
    relative_path: &str,
    value: serde_json::Value,
) {
    if let Some((_, existing, touched, original)) = values
        .iter_mut()
        .find(|(existing_path, _, _, _)| existing_path == relative_path)
    {
        *existing = value;
        *touched = true;
        *original = None;
    } else {
        values.push((relative_path.to_string(), value, true, None));
    }
}
