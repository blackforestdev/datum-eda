use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use super::board_journal_ops::{apply_board_operation, inverse_board_operation};
use super::journal_operation_hooks::{
    apply_non_core_shard_operation, inverse_non_core_operation, stage_non_core_operation,
};
use super::project_manifest_journal_ops::{
    apply_project_manifest_operation, inverse_project_manifest_operation,
};
use super::rules_journal_ops::{apply_rules_operation, inverse_rules_operation};
use super::schematic_definition_journal_ops::maybe_stage_schematic_definition_operation;
use super::schematic_root_journal_ops::{
    apply_schematic_root_operation, inverse_schematic_root_operation,
};
use super::schematic_sheet_journal_ops::{
    apply_schematic_sheet_operation, inverse_schematic_sheet_operation,
};
use super::{
    DesignModel, EngineError, JOURNAL_RELATIVE_PATH, JournalCursor, Operation, OperationBatch,
    ResolveDiagnostic, SourceShardDirtyState, SourceShardKind, SourceShardRef, TransactionRecord,
    read_json_value, sha256_hex, source_shard_authority_for_kind,
};
use crate::ir::serialization::to_json_deterministic;

pub(super) struct StagedShardWrite {
    pub(super) destination: PathBuf,
    pub(super) staged: Option<PathBuf>,
    pub(super) kind: SourceShardKind,
    pub(super) relative_path: String,
    pub(super) content_hash: String,
    pub(super) delete: bool,
}

pub(super) fn stage_operation_shard_writes(
    project_root: &Path,
    model: &DesignModel,
    batch: &OperationBatch,
) -> Result<Vec<StagedShardWrite>, EngineError> {
    let mut staged = Vec::new();
    if let Some(manifest_shard) = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::ProjectManifest)
    {
        let mut manifest_value = materialized_shard_value(model, manifest_shard)?;
        let mut touched_manifest = false;
        for operation in &batch.operations {
            if let Some(applied) = apply_project_manifest_operation(&mut manifest_value, operation)?
            {
                touched_manifest |= applied;
            }
        }
        if touched_manifest {
            staged.push(stage_shard_write(
                project_root,
                batch,
                manifest_shard,
                &manifest_value,
            )?);
        }
    }

    if let Some(rules_shard) = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::RulesRoot)
    {
        let mut rules_value = materialized_shard_value(model, rules_shard)?;
        let mut touched_rules = false;
        for operation in &batch.operations {
            if apply_rules_operation(&mut rules_value, operation)? {
                touched_rules = true;
            }
        }
        if touched_rules {
            staged.push(stage_shard_write(
                project_root,
                batch,
                rules_shard,
                &rules_value,
            )?);
        }
    }

    if let Some(board_shard) = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
    {
        let mut board_value = materialized_shard_value(model, board_shard)?;
        let mut touched_board = false;
        for operation in &batch.operations {
            if apply_board_operation(&mut board_value, operation)? {
                touched_board = true;
            }
        }
        if touched_board {
            staged.push(stage_shard_write(
                project_root,
                batch,
                board_shard,
                &board_value,
            )?);
        }
    }

    if let Some(schematic_shard) = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::SchematicRoot)
    {
        let mut schematic_value = materialized_shard_value(model, schematic_shard)?;
        let mut touched_schematic = false;
        for operation in &batch.operations {
            if apply_schematic_root_operation(&mut schematic_value, operation)? {
                touched_schematic = true;
            }
        }
        if touched_schematic {
            staged.push(stage_shard_write(
                project_root,
                batch,
                schematic_shard,
                &schematic_value,
            )?);
        }
    }

    for sheet_shard in model
        .source_shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::SchematicSheet)
    {
        let mut sheet_value = materialized_shard_value(model, sheet_shard)?;
        let mut touched_sheet = false;
        for operation in &batch.operations {
            if apply_schematic_sheet_operation(&mut sheet_value, operation)? {
                touched_sheet = true;
            }
        }
        if touched_sheet {
            staged.push(stage_shard_write(
                project_root,
                batch,
                sheet_shard,
                &sheet_value,
            )?);
        }
    }

    for operation in &batch.operations {
        maybe_stage_schematic_sheet_operation(project_root, batch, operation, &mut staged)?;
        maybe_stage_schematic_definition_operation(project_root, batch, operation, &mut staged)?;
        stage_non_core_operation(project_root, batch, operation, &mut staged)?;
    }

    Ok(staged)
}

fn maybe_stage_schematic_sheet_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    match operation {
        Operation::CreateSchematicSheet {
            relative_path,
            sheet,
            ..
        } => {
            let relative_path = format!("schematic/{relative_path}");
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::SchematicSheet,
                &relative_path,
                sheet,
            )?);
        }
        Operation::DeleteSchematicSheet { relative_path, .. } => {
            let relative_path = format!("schematic/{relative_path}");
            staged.push(StagedShardWrite {
                destination: project_root.join(&relative_path),
                staged: None,
                kind: SourceShardKind::SchematicSheet,
                relative_path,
                content_hash: String::new(),
                delete: true,
            });
        }
        _ => {}
    }
    Ok(())
}

fn stage_shard_write(
    project_root: &Path,
    batch: &OperationBatch,
    shard: &SourceShardRef,
    value: &serde_json::Value,
) -> Result<StagedShardWrite, EngineError> {
    let stage_path = project_root
        .join(".datum/stage")
        .join(batch.batch_id.to_string())
        .join(&shard.relative_path);
    if let Some(parent) = stage_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = to_json_deterministic(value)?;
    let bytes = format!("{json}\n").into_bytes();
    std::fs::write(&stage_path, &bytes)?;
    std::fs::File::open(&stage_path)?.sync_all()?;
    if let Some(parent) = stage_path.parent() {
        sync_directory(parent)?;
    }

    Ok(StagedShardWrite {
        destination: shard.path.clone(),
        staged: Some(stage_path),
        kind: shard.kind.clone(),
        relative_path: shard.relative_path.clone(),
        content_hash: sha256_hex(&bytes),
        delete: false,
    })
}

pub(super) fn stage_new_shard_write(
    project_root: &Path,
    batch: &OperationBatch,
    kind: SourceShardKind,
    relative_path: &str,
    value: &serde_json::Value,
) -> Result<StagedShardWrite, EngineError> {
    let stage_path = project_root
        .join(".datum/stage")
        .join(batch.batch_id.to_string())
        .join(relative_path);
    if let Some(parent) = stage_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = to_json_deterministic(value)?;
    let bytes = format!("{json}\n").into_bytes();
    std::fs::write(&stage_path, &bytes)?;
    std::fs::File::open(&stage_path)?.sync_all()?;
    if let Some(parent) = stage_path.parent() {
        sync_directory(parent)?;
    }
    Ok(StagedShardWrite {
        destination: project_root.join(relative_path),
        staged: Some(stage_path),
        kind,
        relative_path: relative_path.to_string(),
        content_hash: sha256_hex(&bytes),
        delete: false,
    })
}

pub(super) fn promote_staged_shard_writes(
    writes: Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    let mut synced_dirs = BTreeSet::new();
    for write in writes {
        if write.delete {
            match std::fs::remove_file(&write.destination) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        } else if let Some(staged) = &write.staged {
            if let Some(parent) = write.destination.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(staged, &write.destination)?;
        }
        if let Some(parent) = write.destination.parent() {
            synced_dirs.insert(parent.to_path_buf());
        }
    }
    for dir in synced_dirs {
        sync_directory(&dir)?;
    }
    Ok(())
}

pub(super) fn update_staged_source_hashes(
    shards: &mut Vec<SourceShardRef>,
    writes: &[StagedShardWrite],
) {
    for write in writes {
        if write.delete {
            shards.retain(|shard| shard.relative_path != write.relative_path);
            continue;
        }
        if let Some(shard) = shards
            .iter_mut()
            .find(|shard| shard.path == write.destination)
        {
            shard.content_hash = write.content_hash.clone();
        } else {
            shards.push(SourceShardRef {
                shard_id: Uuid::new_v5(
                    &Uuid::NAMESPACE_URL,
                    format!("datum-eda:source-shard:{}", write.relative_path).as_bytes(),
                ),
                kind: write.kind.clone(),
                path: write.destination.clone(),
                relative_path: write.relative_path.clone(),
                authority: source_shard_authority_for_kind(&write.kind),
                dirty_state: SourceShardDirtyState::Clean,
                schema_version: None,
                content_hash: write.content_hash.clone(),
            });
        }
    }
}

pub(super) fn sort_source_shards(shards: &mut [SourceShardRef]) {
    shards.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.relative_path.cmp(&b.relative_path))
    });
}

pub(super) fn inverse_operations_for_batch(
    model: &DesignModel,
    batch: &OperationBatch,
) -> Result<Vec<Operation>, EngineError> {
    let mut inverse_operations = Vec::new();
    if let Some(manifest_shard) = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::ProjectManifest)
    {
        let mut manifest_value = materialized_shard_value(model, manifest_shard)?;
        for operation in &batch.operations {
            inverse_project_manifest_operation(
                &mut manifest_value,
                operation,
                &mut inverse_operations,
            )?;
        }
    }

    if let Some(rules_shard) = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::RulesRoot)
    {
        let mut rules_value = materialized_shard_value(model, rules_shard)?;
        for operation in &batch.operations {
            inverse_rules_operation(&mut rules_value, operation, &mut inverse_operations)?;
        }
    }

    if let Some(board_shard) = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
    {
        let mut board_value = materialized_shard_value(model, board_shard)?;
        for operation in &batch.operations {
            inverse_board_operation(&mut board_value, operation, &mut inverse_operations)?;
        }
    }
    if let Some(schematic_shard) = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::SchematicRoot)
    {
        let mut schematic_value = materialized_shard_value(model, schematic_shard)?;
        for operation in &batch.operations {
            inverse_schematic_root_operation(
                &mut schematic_value,
                operation,
                &mut inverse_operations,
            )?;
        }
    }
    for sheet_shard in model
        .source_shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::SchematicSheet)
    {
        let mut sheet_value = materialized_shard_value(model, sheet_shard)?;
        for operation in &batch.operations {
            inverse_schematic_sheet_operation(
                &mut sheet_value,
                operation,
                &mut inverse_operations,
            )?;
        }
    }
    for operation in &batch.operations {
        inverse_non_core_operation(operation, &mut inverse_operations);
    }
    inverse_operations.reverse();
    Ok(inverse_operations)
}

pub(super) fn replay_journal_shard_value(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    journal: &[TransactionRecord],
) -> Result<bool, EngineError> {
    let mut changed = false;
    for transaction in journal {
        for operation in &transaction.operations {
            if shard_kind == &SourceShardKind::ProjectManifest
                && apply_project_manifest_operation(value, operation)?.unwrap_or(false)
            {
                changed = true;
            }
            if shard_kind == &SourceShardKind::BoardRoot && apply_board_operation(value, operation)?
            {
                changed = true;
            }
            if shard_kind == &SourceShardKind::RulesRoot && apply_rules_operation(value, operation)?
            {
                changed = true;
            }
            if shard_kind == &SourceShardKind::SchematicRoot
                && apply_schematic_root_operation(value, operation)?
            {
                changed = true;
            }
            if shard_kind == &SourceShardKind::SchematicSheet
                && apply_schematic_sheet_operation(value, operation)?
            {
                changed = true;
            }
            if apply_non_core_shard_operation(shard_kind, value, operation)? {
                changed = true;
            }
        }
    }
    Ok(changed)
}

pub(super) fn canonical_json_hash(value: &serde_json::Value) -> Result<String, EngineError> {
    let json = to_json_deterministic(value)?;
    Ok(sha256_hex(format!("{json}\n").as_bytes()))
}

pub fn transaction_journal_path(project_root: &Path) -> PathBuf {
    project_root.join(JOURNAL_RELATIVE_PATH)
}

pub(super) fn materialized_shard_value(
    model: &DesignModel,
    shard: &SourceShardRef,
) -> Result<serde_json::Value, EngineError> {
    let mut value = match read_json_value(&shard.path) {
        Ok(value) => value,
        Err(EngineError::Io(error))
            if error.kind() == std::io::ErrorKind::NotFound
                && shard.kind == SourceShardKind::SchematicSheet =>
        {
            reconstruct_schematic_sheet_value(&shard.relative_path, &model.journal)?
        }
        Err(error) => return Err(error),
    };
    if canonical_json_hash(&value)? == shard.content_hash {
        return Ok(value);
    }
    replay_journal_shard_value(&shard.kind, &mut value, &model.journal)?;
    Ok(value)
}

fn reconstruct_schematic_sheet_value(
    relative_path: &str,
    journal: &[TransactionRecord],
) -> Result<serde_json::Value, EngineError> {
    let mut value = None;
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::CreateSchematicSheet {
                    relative_path: operation_path,
                    sheet,
                    ..
                } if format!("schematic/{operation_path}") == relative_path => {
                    value = Some(sheet.clone());
                }
                Operation::DeleteSchematicSheet {
                    relative_path: operation_path,
                    ..
                } if format!("schematic/{operation_path}") == relative_path => {
                    value = None;
                }
                _ => {}
            }
        }
    }
    value.ok_or_else(|| {
        EngineError::Validation(format!(
            "missing schematic sheet shard {relative_path} has no journal create record"
        ))
    })
}

fn journal_cursor_path(project_root: &Path) -> PathBuf {
    project_root.join(".datum/journal/cursor.json")
}

pub(super) fn read_transaction_journal(
    project_root: &Path,
) -> (Vec<TransactionRecord>, Vec<ResolveDiagnostic>) {
    let path = transaction_journal_path(project_root);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (Vec::new(), Vec::new());
        }
        Err(error) => {
            return (
                Vec::new(),
                vec![ResolveDiagnostic {
                    code: "journal_read_error".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                }],
            );
        }
    };

    let mut seen = BTreeMap::<Uuid, String>::new();
    let mut transactions = Vec::new();
    let mut diagnostics = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let transaction = match serde_json::from_str::<TransactionRecord>(line) {
            Ok(transaction) => transaction,
            Err(error) => {
                diagnostics.push(ResolveDiagnostic {
                    code: "journal_parse_error".to_string(),
                    message: format!("line {}: {error}", line_index + 1),
                    path: Some(path.clone()),
                });
                break;
            }
        };
        let canonical = match to_json_deterministic(&transaction) {
            Ok(canonical) => canonical,
            Err(error) => {
                diagnostics.push(ResolveDiagnostic {
                    code: "journal_canonicalization_error".to_string(),
                    message: format!("line {}: {error}", line_index + 1),
                    path: Some(path.clone()),
                });
                break;
            }
        };
        if let Some(existing) = seen.get(&transaction.transaction_id) {
            let code = if existing == &canonical {
                "journal_duplicate_transaction_skipped"
            } else {
                "journal_transaction_id_conflict"
            };
            diagnostics.push(ResolveDiagnostic {
                code: code.to_string(),
                message: format!(
                    "line {}: duplicate transaction {}",
                    line_index + 1,
                    transaction.transaction_id
                ),
                path: Some(path.clone()),
            });
            if existing != &canonical {
                break;
            }
        } else {
            seen.insert(transaction.transaction_id, canonical);
            transactions.push(transaction);
        }
    }
    (transactions, diagnostics)
}

pub(super) fn read_journal_cursor(
    project_root: &Path,
    journal_len: usize,
) -> (JournalCursor, Vec<ResolveDiagnostic>) {
    let path = journal_cursor_path(project_root);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (
                JournalCursor {
                    applied_transaction_count: journal_len,
                },
                Vec::new(),
            );
        }
        Err(error) => {
            return (
                JournalCursor {
                    applied_transaction_count: journal_len,
                },
                vec![ResolveDiagnostic {
                    code: "journal_cursor_read_error".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                }],
            );
        }
    };

    let cursor = match serde_json::from_str::<JournalCursor>(&content) {
        Ok(cursor) => cursor,
        Err(error) => {
            return (
                JournalCursor {
                    applied_transaction_count: journal_len,
                },
                vec![ResolveDiagnostic {
                    code: "journal_cursor_parse_error".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                }],
            );
        }
    };
    if cursor.applied_transaction_count > journal_len {
        return (
            JournalCursor {
                applied_transaction_count: journal_len,
            },
            vec![ResolveDiagnostic {
                code: "journal_cursor_out_of_range".to_string(),
                message: format!(
                    "cursor applied_transaction_count {} exceeds journal length {}",
                    cursor.applied_transaction_count, journal_len
                ),
                path: Some(path),
            }],
        );
    }
    if cursor.applied_transaction_count < journal_len {
        return (
            JournalCursor {
                applied_transaction_count: journal_len,
            },
            vec![ResolveDiagnostic {
                code: "journal_cursor_behind".to_string(),
                message: format!(
                    "cursor applied_transaction_count {} is behind journal length {}",
                    cursor.applied_transaction_count, journal_len
                ),
                path: Some(path),
            }],
        );
    }
    (cursor, Vec::new())
}

pub(super) fn write_journal_cursor(
    project_root: &Path,
    cursor: &JournalCursor,
) -> Result<(), EngineError> {
    let path = journal_cursor_path(project_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = to_json_deterministic(cursor)?;
    std::fs::write(&path, format!("{json}\n"))?;
    std::fs::File::open(&path)?.sync_all()?;
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}

pub(super) fn append_transaction_journal(
    project_root: &Path,
    transaction: &TransactionRecord,
) -> Result<(), EngineError> {
    let path = transaction_journal_path(project_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        sync_directory(parent)?;
    }

    let line = to_json_deterministic(transaction)?;
    if path.exists() {
        let existing = recover_torn_journal_tail(&path)?;
        let mut existing_transactions = Vec::new();
        for (line_index, existing_line) in existing.lines().enumerate() {
            if existing_line.trim().is_empty() {
                continue;
            }
            let existing_transaction = serde_json::from_str::<TransactionRecord>(existing_line)
                .map_err(|error| {
                    EngineError::Operation(format!(
                        "journal append refused: existing journal parse error on line {}: {error}",
                        line_index + 1
                    ))
                })?;
            let existing_canonical = to_json_deterministic(&existing_transaction)?;
            existing_transactions.push((existing_transaction, existing_canonical));
        }
        for (index, (existing_transaction, existing_canonical)) in
            existing_transactions.iter().enumerate()
        {
            if existing_transaction.transaction_id == transaction.transaction_id {
                if existing_canonical == &line && index + 1 == existing_transactions.len() {
                    return Ok(());
                }
                if existing_canonical == &line {
                    return Err(EngineError::Operation(format!(
                        "journal append refused: transaction {} already exists before journal tip",
                        transaction.transaction_id
                    )));
                }
                return Err(EngineError::Operation(format!(
                    "journal transaction id conflict: {}",
                    transaction.transaction_id
                )));
            }
        }
        if let Some((last_transaction, _)) = existing_transactions.last() {
            if last_transaction.after_model_revision != transaction.before_model_revision {
                return Err(EngineError::Operation(format!(
                    "journal append refused: transaction {} before revision {} does not match journal tip {}",
                    transaction.transaction_id,
                    transaction.before_model_revision.0,
                    last_transaction.after_model_revision.0
                )));
            }
        }
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}

fn recover_torn_journal_tail(path: &Path) -> Result<String, EngineError> {
    let bytes = std::fs::read(path)?;
    let text = String::from_utf8(bytes).map_err(|error| {
        EngineError::Operation(format!(
            "journal append refused: journal is not UTF-8: {error}"
        ))
    })?;
    let Some(last_newline) = text.rfind('\n') else {
        if text.is_empty() {
            return Ok(text);
        }
        truncate_journal_to(path, 0)?;
        return Ok(String::new());
    };
    if last_newline + 1 == text.len() {
        return Ok(text);
    }
    let repaired = text[..=last_newline].to_string();
    truncate_journal_to(path, repaired.len() as u64)?;
    Ok(repaired)
}

fn truncate_journal_to(path: &Path, len: u64) -> Result<(), EngineError> {
    let file = std::fs::OpenOptions::new().write(true).open(path)?;
    file.set_len(len)?;
    file.sync_all()?;
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), EngineError> {
    std::fs::File::open(path)?.sync_all()?;
    Ok(())
}
