use std::path::Path;

use super::schematic_sheet_journal_ops::apply_schematic_sheet_operation;
use super::source_shard_ref_builders::source_shard_ref_for_value;
use super::{EngineError, SourceShardKind, SourceShardRef, TransactionRecord, read_json_value};

pub(super) fn add_missing_journal_schematic_sheet_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    let mut definition_values = Vec::new();
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                super::Operation::CreateSchematicSheet {
                    relative_path,
                    sheet,
                    ..
                } => {
                    let path = format!("schematic/{relative_path}");
                    upsert_replay_value(&mut values, path, sheet.clone());
                    continue;
                }
                super::Operation::DeleteSchematicSheet { relative_path, .. } => {
                    let path = format!("schematic/{relative_path}");
                    values.retain(|(existing, _)| existing != &path);
                    continue;
                }
                super::Operation::CreateSchematicDefinition {
                    relative_path,
                    definition,
                    ..
                } => {
                    let path = format!("schematic/{relative_path}");
                    upsert_replay_value(&mut definition_values, path, definition.clone());
                    continue;
                }
                super::Operation::DeleteSchematicDefinition { relative_path, .. } => {
                    let path = format!("schematic/{relative_path}");
                    definition_values.retain(|(existing, _)| existing != &path);
                    continue;
                }
                _ => {}
            }
            for (_, value) in &mut values {
                apply_schematic_sheet_operation(value, operation)?;
            }
        }
    }
    for (relative_path, value) in values {
        if shards
            .iter()
            .any(|shard| shard.relative_path == relative_path)
        {
            continue;
        }
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::SchematicSheet,
            relative_path,
            &value,
        )?);
    }
    for (relative_path, value) in definition_values {
        if shards
            .iter()
            .any(|shard| shard.relative_path == relative_path)
        {
            continue;
        }
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::SchematicDefinition,
            relative_path,
            &value,
        )?);
    }
    Ok(())
}

pub(super) fn replay_schematic_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    replay_schematic_sheet_shards(project_root, shards, journal)?;
    replay_schematic_definition_shards(project_root, shards, journal)
}

fn replay_schematic_sheet_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::SchematicSheet)
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
                super::Operation::CreateSchematicSheet {
                    relative_path,
                    sheet,
                    ..
                } => {
                    upsert_replay_materialized(
                        &mut values,
                        format!("schematic/{relative_path}"),
                        sheet.clone(),
                    );
                    continue;
                }
                super::Operation::DeleteSchematicSheet { relative_path, .. } => {
                    let path = format!("schematic/{relative_path}");
                    values.retain(|(existing, _, _, _)| existing != &path);
                    continue;
                }
                _ => {}
            }
            for (_, value, touched, original) in &mut values {
                if apply_schematic_sheet_operation(value, operation)? {
                    *touched = true;
                    *original = None;
                }
            }
        }
    }
    replace_replayed_shards(
        project_root,
        shards,
        SourceShardKind::SchematicSheet,
        values,
    )
}

fn replay_schematic_definition_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::SchematicDefinition)
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
                super::Operation::CreateSchematicDefinition {
                    relative_path,
                    definition,
                    ..
                } => upsert_replay_materialized(
                    &mut values,
                    format!("schematic/{relative_path}"),
                    definition.clone(),
                ),
                super::Operation::DeleteSchematicDefinition { relative_path, .. } => {
                    let path = format!("schematic/{relative_path}");
                    values.retain(|(existing, _, _, _)| existing != &path);
                }
                _ => {}
            }
        }
    }
    replace_replayed_shards(
        project_root,
        shards,
        SourceShardKind::SchematicDefinition,
        values,
    )
}

#[rustfmt::skip]
fn upsert_replay_value(values: &mut Vec<(String, serde_json::Value)>, path: String, value: serde_json::Value) {
    if let Some((_, existing)) = values.iter_mut().find(|(existing, _)| existing == &path) { *existing = value; } else { values.push((path, value)); }
}

fn upsert_replay_materialized(
    values: &mut Vec<(String, serde_json::Value, bool, Option<SourceShardRef>)>,
    relative_path: String,
    value: serde_json::Value,
) {
    if let Some((_, existing, touched, original)) = values
        .iter_mut()
        .find(|(existing_path, _, _, _)| existing_path == &relative_path)
    {
        *existing = value;
        *touched = true;
        *original = None;
    } else {
        values.push((relative_path, value, true, None));
    }
}

fn replace_replayed_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    kind: SourceShardKind,
    values: Vec<(String, serde_json::Value, bool, Option<SourceShardRef>)>,
) -> Result<(), EngineError> {
    shards.retain(|shard| shard.kind != kind);
    for (relative_path, value, touched, original) in values {
        if !touched {
            if let Some(shard) = original {
                shards.push(shard);
            }
            continue;
        }
        shards.push(source_shard_ref_for_value(
            project_root,
            kind.clone(),
            relative_path,
            &value,
        )?);
    }
    Ok(())
}
