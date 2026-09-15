use std::path::Path;

use super::generated_evidence_journal_ops::{
    apply_generated_evidence_shard_operation, artifact_metadata_relative_path,
    artifact_run_relative_path, check_run_relative_path, output_job_run_relative_path,
};
use super::source_shard_ref_builders::source_shard_ref_for_value;
use super::zone_fill_journal_ops::{apply_zone_fill_shard_operation, zone_fill_relative_path};
use super::{EngineError, SourceShardKind, SourceShardRef, TransactionRecord, read_json_value};

pub(super) fn replay_generated_evidence_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    replay_output_job_run_shards(project_root, shards, journal)?;
    replay_artifact_run_shards(project_root, shards, journal)?;
    replay_check_run_shards(project_root, shards, journal)?;
    replay_artifact_metadata_shards(project_root, shards, journal)?;
    replay_zone_fill_shards(project_root, shards, journal)
}

fn replay_artifact_metadata_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::ArtifactMetadata)
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
                super::Operation::SetArtifactMetadata {
                    artifact_id,
                    artifact_metadata,
                    ..
                } => upsert_replay_value(
                    &mut values,
                    &artifact_metadata_relative_path(*artifact_id),
                    artifact_metadata.clone(),
                ),
                super::Operation::DeleteArtifactMetadata { artifact_id, .. } => {
                    let relative_path = artifact_metadata_relative_path(*artifact_id);
                    values.retain(|(existing, _, _, _)| existing != &relative_path);
                }
                _ => {
                    for (_, value, touched, original) in &mut values {
                        if !*touched && original.is_some() {
                            continue;
                        }
                        if apply_generated_evidence_shard_operation(
                            &SourceShardKind::ArtifactMetadata,
                            value,
                            operation,
                        )? {
                            *touched = true;
                            *original = None;
                        }
                    }
                }
            }
        }
    }
    shards.retain(|shard| shard.kind != SourceShardKind::ArtifactMetadata);
    for (relative_path, value, touched, original) in values {
        if !touched {
            if let Some(shard) = original {
                shards.push(shard);
            }
            continue;
        }
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::ArtifactMetadata,
            relative_path,
            &value,
        )?);
    }
    Ok(())
}

fn replay_check_run_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::CheckRun)
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
                super::Operation::SetCheckRun {
                    check_run_id,
                    check_run,
                    ..
                } => upsert_replay_value(
                    &mut values,
                    &check_run_relative_path(*check_run_id),
                    check_run.clone(),
                ),
                super::Operation::DeleteCheckRun { check_run_id, .. } => {
                    let relative_path = check_run_relative_path(*check_run_id);
                    values.retain(|(existing, _, _, _)| existing != &relative_path);
                }
                _ => {
                    for (_, value, touched, original) in &mut values {
                        if !*touched && original.is_some() {
                            continue;
                        }
                        if apply_generated_evidence_shard_operation(
                            &SourceShardKind::CheckRun,
                            value,
                            operation,
                        )? {
                            *touched = true;
                            *original = None;
                        }
                    }
                }
            }
        }
    }
    shards.retain(|shard| shard.kind != SourceShardKind::CheckRun);
    for (relative_path, value, touched, original) in values {
        if !touched {
            if let Some(shard) = original {
                shards.push(shard);
            }
            continue;
        }
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::CheckRun,
            relative_path,
            &value,
        )?);
    }
    Ok(())
}

fn replay_artifact_run_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::ArtifactRun)
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
                super::Operation::SetArtifactRun {
                    run_id,
                    artifact_run,
                    ..
                } => upsert_replay_value(
                    &mut values,
                    &artifact_run_relative_path(*run_id),
                    artifact_run.clone(),
                ),
                super::Operation::DeleteArtifactRun { run_id, .. } => {
                    let relative_path = artifact_run_relative_path(*run_id);
                    values.retain(|(existing, _, _, _)| existing != &relative_path);
                }
                _ => {
                    for (_, value, touched, original) in &mut values {
                        if !*touched && original.is_some() {
                            continue;
                        }
                        if apply_generated_evidence_shard_operation(
                            &SourceShardKind::ArtifactRun,
                            value,
                            operation,
                        )? {
                            *touched = true;
                            *original = None;
                        }
                    }
                }
            }
        }
    }
    shards.retain(|shard| shard.kind != SourceShardKind::ArtifactRun);
    for (relative_path, value, touched, original) in values {
        if !touched {
            if let Some(shard) = original {
                shards.push(shard);
            }
            continue;
        }
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::ArtifactRun,
            relative_path,
            &value,
        )?);
    }
    Ok(())
}

fn replay_output_job_run_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::OutputJobRun)
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
                super::Operation::SetOutputJobRun {
                    run_id,
                    output_job_run,
                    ..
                } => upsert_replay_value(
                    &mut values,
                    &output_job_run_relative_path(*run_id),
                    output_job_run.clone(),
                ),
                super::Operation::DeleteOutputJobRun { run_id, .. } => {
                    let relative_path = output_job_run_relative_path(*run_id);
                    values.retain(|(existing, _, _, _)| existing != &relative_path);
                }
                _ => {
                    for (_, value, touched, original) in &mut values {
                        if !*touched && original.is_some() {
                            continue;
                        }
                        if apply_generated_evidence_shard_operation(
                            &SourceShardKind::OutputJobRun,
                            value,
                            operation,
                        )? {
                            *touched = true;
                            *original = None;
                        }
                    }
                }
            }
        }
    }
    shards.retain(|shard| shard.kind != SourceShardKind::OutputJobRun);
    for (relative_path, value, touched, original) in values {
        if !touched {
            if let Some(shard) = original {
                shards.push(shard);
            }
            continue;
        }
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::OutputJobRun,
            relative_path,
            &value,
        )?);
    }
    Ok(())
}

fn replay_zone_fill_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::ZoneFill)
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
                super::Operation::SetZoneFill {
                    zone_id, zone_fill, ..
                } => upsert_replay_value(
                    &mut values,
                    &zone_fill_relative_path(*zone_id),
                    zone_fill.clone(),
                ),
                super::Operation::DeleteZoneFill { zone_id, .. } => {
                    let relative_path = zone_fill_relative_path(*zone_id);
                    values.retain(|(existing, _, _, _)| existing != &relative_path);
                }
                _ => {
                    for (_, value, touched, original) in &mut values {
                        if !*touched && original.is_some() {
                            continue;
                        }
                        if apply_zone_fill_shard_operation(
                            &SourceShardKind::ZoneFill,
                            value,
                            operation,
                        )? {
                            *touched = true;
                            *original = None;
                        }
                    }
                }
            }
        }
    }
    shards.retain(|shard| shard.kind != SourceShardKind::ZoneFill);
    for (relative_path, value, touched, original) in values {
        if !touched {
            if let Some(shard) = original {
                shards.push(shard);
            }
            continue;
        }
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::ZoneFill,
            relative_path,
            &value,
        )?);
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
