use std::path::Path;

use uuid::Uuid;

use super::{
    ArtifactMetadata, ArtifactRun, CheckRun, DesignModel, EngineError, Operation, OperationBatch,
    OutputJobRun, SourceShardKind, TransactionRecord,
    artifact_validation::validate_artifact_metadata,
    check_run::validate_check_run,
    journal::{StagedShardWrite, stage_new_shard_write},
    run_evidence_validation::{
        validate_artifact_run, validate_output_job_run, validate_run_evidence_batch_links,
    },
};

pub(super) fn maybe_stage_generated_evidence_operation(
    project_root: &Path,
    model: &DesignModel,
    batch: &OperationBatch,
    operation: &Operation,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    match operation {
        Operation::SetOutputJobRun {
            run_id,
            output_job_run,
            ..
        } => {
            let run = validated_output_job_run_payload(*run_id, output_job_run)?;
            validate_generated_evidence_scope(
                "output job run",
                Some(run.project_id),
                &run.model_revision,
                model,
            )?;
            validate_run_evidence_batch_links(model, &batch.operations)?;
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::OutputJobRun,
                &output_job_run_relative_path(*run_id),
                output_job_run,
            )?);
        }
        Operation::DeleteOutputJobRun {
            run_id,
            output_job_run,
        } => {
            validated_output_job_run_payload(*run_id, output_job_run)?;
            validate_run_evidence_batch_links(model, &batch.operations)?;
            let relative_path = output_job_run_relative_path(*run_id);
            staged.push(StagedShardWrite {
                destination: project_root.join(&relative_path),
                staged: None,
                kind: SourceShardKind::OutputJobRun,
                relative_path,
                content_hash: String::new(),
                schema_version: None,
                delete: true,
            });
        }
        Operation::SetArtifactRun {
            run_id,
            artifact_run,
            ..
        } => {
            let run = validated_artifact_run_payload(*run_id, artifact_run)?;
            validate_generated_evidence_scope(
                "artifact run",
                Some(run.project_id),
                &run.model_revision,
                model,
            )?;
            validate_run_evidence_batch_links(model, &batch.operations)?;
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::ArtifactRun,
                &artifact_run_relative_path(*run_id),
                artifact_run,
            )?);
        }
        Operation::DeleteArtifactRun {
            run_id,
            artifact_run,
        } => {
            validated_artifact_run_payload(*run_id, artifact_run)?;
            validate_run_evidence_batch_links(model, &batch.operations)?;
            let relative_path = artifact_run_relative_path(*run_id);
            staged.push(StagedShardWrite {
                destination: project_root.join(&relative_path),
                staged: None,
                kind: SourceShardKind::ArtifactRun,
                relative_path,
                content_hash: String::new(),
                schema_version: None,
                delete: true,
            });
        }
        Operation::SetCheckRun {
            check_run_id,
            check_run,
            ..
        } => {
            let run = validated_check_run_payload(*check_run_id, check_run)?;
            validate_generated_evidence_scope(
                "check run",
                Some(run.project_id),
                &run.model_revision,
                model,
            )?;
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::CheckRun,
                &check_run_relative_path(*check_run_id),
                check_run,
            )?);
        }
        Operation::DeleteCheckRun {
            check_run_id,
            check_run,
        } => {
            validated_check_run_payload(*check_run_id, check_run)?;
            let relative_path = check_run_relative_path(*check_run_id);
            staged.push(StagedShardWrite {
                destination: project_root.join(&relative_path),
                staged: None,
                kind: SourceShardKind::CheckRun,
                relative_path,
                content_hash: String::new(),
                schema_version: None,
                delete: true,
            });
        }
        Operation::SetArtifactMetadata {
            artifact_id,
            artifact_metadata,
            ..
        } => {
            let metadata = validated_artifact_metadata_payload(*artifact_id, artifact_metadata)?;
            validate_generated_evidence_scope(
                "artifact metadata",
                Some(metadata.project_id),
                &metadata.model_revision,
                model,
            )?;
            validate_run_evidence_batch_links(model, &batch.operations)?;
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::ArtifactMetadata,
                &artifact_metadata_relative_path(*artifact_id),
                artifact_metadata,
            )?);
        }
        Operation::DeleteArtifactMetadata {
            artifact_id,
            artifact_metadata,
        } => {
            validated_artifact_metadata_payload(*artifact_id, artifact_metadata)?;
            validate_run_evidence_batch_links(model, &batch.operations)?;
            let relative_path = artifact_metadata_relative_path(*artifact_id);
            staged.push(StagedShardWrite {
                destination: project_root.join(&relative_path),
                staged: None,
                kind: SourceShardKind::ArtifactMetadata,
                relative_path,
                content_hash: String::new(),
                schema_version: None,
                delete: true,
            });
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn inverse_generated_evidence_operation(
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) {
    match operation {
        Operation::SetOutputJobRun {
            run_id,
            previous_output_job_run: Some(previous_output_job_run),
            output_job_run,
        } => inverse_operations.push(Operation::SetOutputJobRun {
            run_id: *run_id,
            previous_output_job_run: Some(output_job_run.clone()),
            output_job_run: previous_output_job_run.clone(),
        }),
        Operation::SetOutputJobRun {
            run_id,
            previous_output_job_run: None,
            output_job_run,
        } => inverse_operations.push(Operation::DeleteOutputJobRun {
            run_id: *run_id,
            output_job_run: output_job_run.clone(),
        }),
        Operation::DeleteOutputJobRun {
            run_id,
            output_job_run,
        } => inverse_operations.push(Operation::SetOutputJobRun {
            run_id: *run_id,
            previous_output_job_run: None,
            output_job_run: output_job_run.clone(),
        }),
        Operation::SetArtifactRun {
            run_id,
            previous_artifact_run: Some(previous_artifact_run),
            artifact_run,
        } => inverse_operations.push(Operation::SetArtifactRun {
            run_id: *run_id,
            previous_artifact_run: Some(artifact_run.clone()),
            artifact_run: previous_artifact_run.clone(),
        }),
        Operation::SetArtifactRun {
            run_id,
            previous_artifact_run: None,
            artifact_run,
        } => inverse_operations.push(Operation::DeleteArtifactRun {
            run_id: *run_id,
            artifact_run: artifact_run.clone(),
        }),
        Operation::DeleteArtifactRun {
            run_id,
            artifact_run,
        } => inverse_operations.push(Operation::SetArtifactRun {
            run_id: *run_id,
            previous_artifact_run: None,
            artifact_run: artifact_run.clone(),
        }),
        Operation::SetCheckRun {
            check_run_id,
            previous_check_run: Some(previous_check_run),
            check_run,
        } => inverse_operations.push(Operation::SetCheckRun {
            check_run_id: *check_run_id,
            previous_check_run: Some(check_run.clone()),
            check_run: previous_check_run.clone(),
        }),
        Operation::SetCheckRun {
            check_run_id,
            previous_check_run: None,
            check_run,
        } => inverse_operations.push(Operation::DeleteCheckRun {
            check_run_id: *check_run_id,
            check_run: check_run.clone(),
        }),
        Operation::DeleteCheckRun {
            check_run_id,
            check_run,
        } => inverse_operations.push(Operation::SetCheckRun {
            check_run_id: *check_run_id,
            previous_check_run: None,
            check_run: check_run.clone(),
        }),
        Operation::SetArtifactMetadata {
            artifact_id,
            previous_artifact_metadata: Some(previous_artifact_metadata),
            artifact_metadata,
        } => inverse_operations.push(Operation::SetArtifactMetadata {
            artifact_id: *artifact_id,
            previous_artifact_metadata: Some(artifact_metadata.clone()),
            artifact_metadata: previous_artifact_metadata.clone(),
        }),
        Operation::SetArtifactMetadata {
            artifact_id,
            previous_artifact_metadata: None,
            artifact_metadata,
        } => inverse_operations.push(Operation::DeleteArtifactMetadata {
            artifact_id: *artifact_id,
            artifact_metadata: artifact_metadata.clone(),
        }),
        Operation::DeleteArtifactMetadata {
            artifact_id,
            artifact_metadata,
        } => inverse_operations.push(Operation::SetArtifactMetadata {
            artifact_id: *artifact_id,
            previous_artifact_metadata: None,
            artifact_metadata: artifact_metadata.clone(),
        }),
        _ => {}
    }
}

pub(super) fn apply_generated_evidence_shard_operation(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    let id_field = match shard_kind {
        SourceShardKind::CheckRun => "check_run_id",
        SourceShardKind::ArtifactMetadata => "artifact_id",
        _ => "run_id",
    };
    let Some(current_id) = value.get(id_field).and_then(serde_json::Value::as_str) else {
        return Ok(false);
    };
    match (shard_kind, operation) {
        (
            SourceShardKind::OutputJobRun,
            Operation::SetOutputJobRun {
                run_id,
                output_job_run,
                ..
            },
        ) if current_id == run_id.to_string() => {
            validated_output_job_run_payload(*run_id, output_job_run)?;
            *value = output_job_run.clone();
            Ok(true)
        }
        (SourceShardKind::OutputJobRun, Operation::DeleteOutputJobRun { run_id, .. })
            if current_id == run_id.to_string() =>
        {
            *value = serde_json::Value::Null;
            Ok(true)
        }
        (
            SourceShardKind::ArtifactRun,
            Operation::SetArtifactRun {
                run_id,
                artifact_run,
                ..
            },
        ) if current_id == run_id.to_string() => {
            validated_artifact_run_payload(*run_id, artifact_run)?;
            *value = artifact_run.clone();
            Ok(true)
        }
        (SourceShardKind::ArtifactRun, Operation::DeleteArtifactRun { run_id, .. })
            if current_id == run_id.to_string() =>
        {
            *value = serde_json::Value::Null;
            Ok(true)
        }
        (
            SourceShardKind::CheckRun,
            Operation::SetCheckRun {
                check_run_id,
                check_run,
                ..
            },
        ) if current_id == check_run_id.to_string() => {
            validated_check_run_payload(*check_run_id, check_run)?;
            *value = check_run.clone();
            Ok(true)
        }
        (SourceShardKind::CheckRun, Operation::DeleteCheckRun { check_run_id, .. })
            if current_id == check_run_id.to_string() =>
        {
            *value = serde_json::Value::Null;
            Ok(true)
        }
        (
            SourceShardKind::ArtifactMetadata,
            Operation::SetArtifactMetadata {
                artifact_id,
                artifact_metadata,
                ..
            },
        ) if current_id == artifact_id.to_string() => {
            validated_artifact_metadata_payload(*artifact_id, artifact_metadata)?;
            *value = artifact_metadata.clone();
            Ok(true)
        }
        (
            SourceShardKind::ArtifactMetadata,
            Operation::DeleteArtifactMetadata { artifact_id, .. },
        ) if current_id == artifact_id.to_string() => {
            *value = serde_json::Value::Null;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn apply_output_job_run_journal_to_map(
    journal: &[TransactionRecord],
    output_job_runs: &mut std::collections::BTreeMap<Uuid, OutputJobRun>,
) -> Result<(), EngineError> {
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::SetOutputJobRun {
                    run_id,
                    output_job_run,
                    ..
                } => {
                    let run = validated_output_job_run_payload(*run_id, output_job_run)?;
                    output_job_runs.insert(*run_id, run);
                }
                Operation::DeleteOutputJobRun {
                    run_id,
                    output_job_run,
                } => {
                    validated_output_job_run_payload(*run_id, output_job_run)?;
                    output_job_runs.remove(run_id);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub(super) fn apply_artifact_run_journal_to_map(
    journal: &[TransactionRecord],
    artifact_runs: &mut std::collections::BTreeMap<Uuid, ArtifactRun>,
) -> Result<(), EngineError> {
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::SetArtifactRun {
                    run_id,
                    artifact_run,
                    ..
                } => {
                    let run = validated_artifact_run_payload(*run_id, artifact_run)?;
                    artifact_runs.insert(*run_id, run);
                }
                Operation::DeleteArtifactRun {
                    run_id,
                    artifact_run,
                } => {
                    validated_artifact_run_payload(*run_id, artifact_run)?;
                    artifact_runs.remove(run_id);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub(super) fn apply_check_run_journal_to_map(
    journal: &[TransactionRecord],
    check_runs: &mut std::collections::BTreeMap<Uuid, CheckRun>,
) -> Result<(), EngineError> {
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::SetCheckRun {
                    check_run_id,
                    check_run,
                    ..
                } => {
                    let run = validated_check_run_payload(*check_run_id, check_run)?;
                    check_runs.insert(*check_run_id, run);
                }
                Operation::DeleteCheckRun {
                    check_run_id,
                    check_run,
                } => {
                    validated_check_run_payload(*check_run_id, check_run)?;
                    check_runs.remove(check_run_id);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub(super) fn apply_artifact_metadata_journal_to_map(
    journal: &[TransactionRecord],
    artifact_metadata: &mut std::collections::BTreeMap<Uuid, ArtifactMetadata>,
) -> Result<(), EngineError> {
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::SetArtifactMetadata {
                    artifact_id,
                    artifact_metadata: metadata,
                    ..
                } => {
                    let metadata = validated_artifact_metadata_payload(*artifact_id, metadata)?;
                    artifact_metadata.insert(*artifact_id, metadata);
                }
                Operation::DeleteArtifactMetadata {
                    artifact_id,
                    artifact_metadata: metadata,
                } => {
                    validated_artifact_metadata_payload(*artifact_id, metadata)?;
                    artifact_metadata.remove(artifact_id);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub(super) fn apply_generated_evidence_model_operation(
    model: &mut DesignModel,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::SetOutputJobRun {
            run_id,
            output_job_run,
            ..
        } => {
            let run = validated_output_job_run_payload(*run_id, output_job_run)?;
            model.output_job_runs.insert(*run_id, run);
            Ok(true)
        }
        Operation::DeleteOutputJobRun {
            run_id,
            output_job_run,
        } => {
            validated_output_job_run_payload(*run_id, output_job_run)?;
            model.output_job_runs.remove(run_id);
            Ok(true)
        }
        Operation::SetArtifactRun {
            run_id,
            artifact_run,
            ..
        } => {
            let run = validated_artifact_run_payload(*run_id, artifact_run)?;
            model.artifact_runs.insert(*run_id, run);
            Ok(true)
        }
        Operation::DeleteArtifactRun {
            run_id,
            artifact_run,
        } => {
            validated_artifact_run_payload(*run_id, artifact_run)?;
            model.artifact_runs.remove(run_id);
            Ok(true)
        }
        Operation::SetCheckRun {
            check_run_id,
            check_run,
            ..
        } => {
            let run = validated_check_run_payload(*check_run_id, check_run)?;
            model.check_runs.insert(*check_run_id, run);
            Ok(true)
        }
        Operation::DeleteCheckRun {
            check_run_id,
            check_run,
        } => {
            validated_check_run_payload(*check_run_id, check_run)?;
            model.check_runs.remove(check_run_id);
            Ok(true)
        }
        Operation::SetArtifactMetadata {
            artifact_id,
            artifact_metadata,
            ..
        } => {
            let metadata = validated_artifact_metadata_payload(*artifact_id, artifact_metadata)?;
            model.artifact_metadata.insert(*artifact_id, metadata);
            Ok(true)
        }
        Operation::DeleteArtifactMetadata {
            artifact_id,
            artifact_metadata,
        } => {
            validated_artifact_metadata_payload(*artifact_id, artifact_metadata)?;
            model.artifact_metadata.remove(artifact_id);
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn output_job_run_relative_path(run_id: Uuid) -> String {
    format!(".datum/output_job_runs/{run_id}.json")
}

pub(super) fn artifact_run_relative_path(run_id: Uuid) -> String {
    format!(".datum/artifact_runs/{run_id}.json")
}

pub(super) fn check_run_relative_path(check_run_id: Uuid) -> String {
    format!(".datum/check_runs/{check_run_id}.json")
}

pub(super) fn artifact_metadata_relative_path(artifact_id: Uuid) -> String {
    format!(".datum/artifacts/{artifact_id}.json")
}

pub(super) fn validated_output_job_run_payload(
    expected_run_id: Uuid,
    value: &serde_json::Value,
) -> Result<OutputJobRun, EngineError> {
    let run = serde_json::from_value::<OutputJobRun>(value.clone())?;
    if run.run_id != expected_run_id {
        return Err(EngineError::Validation(format!(
            "output job run payload id {} does not match operation run_id {}",
            run.run_id, expected_run_id
        )));
    }
    validate_output_job_run(&run).map_err(EngineError::Validation)?;
    Ok(run)
}

pub(super) fn validated_artifact_run_payload(
    expected_run_id: Uuid,
    value: &serde_json::Value,
) -> Result<ArtifactRun, EngineError> {
    let run = serde_json::from_value::<ArtifactRun>(value.clone())?;
    if run.run_id != expected_run_id {
        return Err(EngineError::Validation(format!(
            "artifact run payload id {} does not match operation run_id {}",
            run.run_id, expected_run_id
        )));
    }
    validate_artifact_run(&run).map_err(EngineError::Validation)?;
    Ok(run)
}

pub(super) fn validated_check_run_payload(
    expected_check_run_id: Uuid,
    value: &serde_json::Value,
) -> Result<CheckRun, EngineError> {
    let run = serde_json::from_value::<CheckRun>(value.clone())?;
    if run.check_run_id != expected_check_run_id {
        return Err(EngineError::Validation(format!(
            "check run payload id {} does not match operation check_run_id {}",
            run.check_run_id, expected_check_run_id
        )));
    }
    validate_check_run(&run).map_err(EngineError::Validation)?;
    Ok(run)
}

pub(super) fn validated_artifact_metadata_payload(
    expected_artifact_id: Uuid,
    value: &serde_json::Value,
) -> Result<ArtifactMetadata, EngineError> {
    let metadata = serde_json::from_value::<ArtifactMetadata>(value.clone())?;
    if metadata.artifact_id != expected_artifact_id {
        return Err(EngineError::Validation(format!(
            "artifact metadata payload id {} does not match operation artifact_id {}",
            metadata.artifact_id, expected_artifact_id
        )));
    }
    validate_artifact_metadata(&metadata).map_err(EngineError::Validation)?;
    Ok(metadata)
}

pub(super) fn validate_generated_evidence_scope(
    evidence_kind: &str,
    project_id: Option<Uuid>,
    model_revision: &super::ModelRevision,
    model: &DesignModel,
) -> Result<(), EngineError> {
    if let Some(project_id) = project_id
        && project_id != model.project.project_id
    {
        return Err(EngineError::Validation(format!(
            "{evidence_kind} project_id {project_id} does not match current project_id {}",
            model.project.project_id
        )));
    }
    if model_revision != &model.model_revision {
        return Err(EngineError::Validation(format!(
            "{evidence_kind} model_revision {} does not match current model_revision {}",
            model_revision.0, model.model_revision.0
        )));
    }
    Ok(())
}
