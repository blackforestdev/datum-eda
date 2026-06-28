use std::collections::BTreeMap;
use std::path::Path;

use uuid::Uuid;

use super::component_instance_journal_ops::{
    component_instance_operation_write, wrap_payload as wrap_component_instance_payload,
};
use super::production_journal_ops::{production_operation_write, production_relative_path};
use super::relationship_journal_ops::{relationship_operation_write, wrap_payload};
use super::replay_forward_annotation::replay_forward_annotation_review_shard;
use super::replay_generated_evidence::replay_generated_evidence_shards;
use super::replay_objects::refresh_materialized_shard_objects;
use super::replay_pool::replay_pool_shards;
use super::replay_proposal::replay_proposal_shards;
use super::replay_schematic::{
    add_missing_journal_schematic_sheet_shards, replay_schematic_shards,
};
use super::source_shard_ref_builders::source_shard_ref_for_value;
use super::transaction_links::validate_transaction_links;
use super::{
    CommitDiff, DesignModel, DomainObject, EngineError, JournalCursor, ObjectId, ObjectRevision,
    ProjectManifestSummary, ResolveDiagnostic, SourceShardKind, SourceShardRef, TransactionRecord,
    apply_operation, canonical_json_hash, compute_model_revision, read_json_value,
    replay_journal_shard_value, source_shard::dirty_state_for_materialized_shard,
    transaction_journal_path,
};

pub(super) fn validate_and_replay_journal(
    project_root: &Path,
    project_id: &Uuid,
    shards: &mut Vec<SourceShardRef>,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    journal: &[TransactionRecord],
    diagnostics: &mut Vec<ResolveDiagnostic>,
) -> Result<Vec<TransactionRecord>, EngineError> {
    let mut valid = Vec::new();
    let mut current_revision = compute_model_revision(project_id, shards, objects);
    if let Some(first) = journal.first() {
        if first.before_model_revision != current_revision {
            return validate_promoted_journal_tip(
                project_root,
                project_id,
                shards,
                objects,
                journal,
                diagnostics,
            );
        }
    }

    for (index, transaction) in journal.iter().enumerate() {
        if let Err(message) = validate_transaction_links(transaction, &valid) {
            diagnostics.push(ResolveDiagnostic {
                code: "journal_link_mismatch".to_string(),
                message: format!(
                    "transaction {} at index {} has invalid undo/redo links: {}",
                    transaction.transaction_id, index, message
                ),
                path: Some(transaction_journal_path(project_root)),
            });
            break;
        }
        if transaction.before_model_revision != current_revision {
            diagnostics.push(ResolveDiagnostic {
                code: "journal_chain_mismatch".to_string(),
                message: format!(
                    "transaction {} at index {} expected before revision {}, current {}",
                    transaction.transaction_id,
                    index,
                    transaction.before_model_revision.0,
                    current_revision.0
                ),
                path: Some(transaction_journal_path(project_root)),
            });
            break;
        }

        let mut candidate_objects = objects.clone();
        let mut candidate_shards = shards.to_vec();
        let mut candidate_journal = valid.clone();
        candidate_journal.push(transaction.clone());
        if let Err(error) = apply_transaction_operations_to_objects(
            project_id,
            &candidate_shards,
            &mut candidate_objects,
            transaction,
            &candidate_journal,
        ) {
            diagnostics.push(ResolveDiagnostic {
                code: "journal_replay_failed".to_string(),
                message: format!(
                    "transaction {} at index {} could not replay against current source shards: {}",
                    transaction.transaction_id, index, error
                ),
                path: Some(transaction_journal_path(project_root)),
            });
            break;
        }
        if let Err(error) = replay_journal_prefix_to_source_shards(
            project_root,
            &mut candidate_shards,
            &candidate_journal,
        ) {
            diagnostics.push(ResolveDiagnostic {
                code: "journal_replay_failed".to_string(),
                message: format!(
                    "transaction {} at index {} could not materialize source shards: {}",
                    transaction.transaction_id, index, error
                ),
                path: Some(transaction_journal_path(project_root)),
            });
            break;
        }
        add_missing_journal_schematic_sheet_shards(
            project_root,
            &mut candidate_shards,
            &candidate_journal,
        )?;
        replay_proposal_shards(project_root, &mut candidate_shards, &candidate_journal)?;
        let candidate_revision =
            compute_model_revision(project_id, &candidate_shards, &candidate_objects);
        if transaction.after_model_revision != candidate_revision {
            if promoted_production_journal_matches_files(project_root, journal)? {
                return Ok(journal.to_vec());
            }
            if journal_operations_are_production_only(&candidate_journal) {
                objects.clear();
                objects.extend(candidate_objects);
                *shards = candidate_shards;
                current_revision = transaction.after_model_revision.clone();
                valid.push(transaction.clone());
                continue;
            }
            diagnostics.push(ResolveDiagnostic {
                code: "journal_after_revision_mismatch".to_string(),
                message: format!(
                    "transaction {} at index {} claimed after revision {}, computed {}",
                    transaction.transaction_id,
                    index,
                    transaction.after_model_revision.0,
                    candidate_revision.0
                ),
                path: Some(transaction_journal_path(project_root)),
            });
            break;
        }
        objects.clear();
        objects.extend(candidate_objects);
        *shards = candidate_shards;
        current_revision = candidate_revision;
        valid.push(transaction.clone());
    }

    Ok(valid)
}

fn journal_operations_are_production_only(journal: &[TransactionRecord]) -> bool {
    !journal.is_empty()
        && journal.iter().all(|transaction| {
            !transaction.operations.is_empty()
                && transaction
                    .operations
                    .iter()
                    .all(|operation| production_operation_write(operation).is_some())
        })
}

fn validate_promoted_journal_tip(
    project_root: &Path,
    project_id: &Uuid,
    shards: &mut Vec<SourceShardRef>,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    journal: &[TransactionRecord],
    diagnostics: &mut Vec<ResolveDiagnostic>,
) -> Result<Vec<TransactionRecord>, EngineError> {
    if let Some(tip) = journal.last() {
        let current_revision = compute_model_revision(project_id, shards, objects);
        if tip.after_model_revision == current_revision {
            let valid = validate_promoted_journal_links(project_root, journal, diagnostics)?;
            if valid.len() == journal.len() {
                install_promoted_journal_state(project_root, shards, objects, journal)?;
            }
            return Ok(valid);
        }
        let mut adjusted_objects = objects.clone();
        let mut adjusted_shards = shards.to_vec();
        apply_promoted_journal_object_diffs(&mut adjusted_objects, journal);
        add_missing_journal_schematic_sheet_shards(project_root, &mut adjusted_shards, journal)?;
        replay_proposal_shards(project_root, &mut adjusted_shards, journal)?;
        replay_generated_evidence_shards(project_root, &mut adjusted_shards, journal)?;
        replay_pool_shards(project_root, &mut adjusted_shards, journal)?;
        let adjusted_revision =
            compute_model_revision(project_id, &adjusted_shards, &adjusted_objects);
        if tip.after_model_revision == adjusted_revision {
            let valid = validate_promoted_journal_links(project_root, journal, diagnostics)?;
            if valid.len() == journal.len() {
                objects.clear();
                objects.extend(adjusted_objects);
                *shards = adjusted_shards;
            }
            return Ok(valid);
        }
    }

    if promoted_production_journal_matches_files(project_root, journal)? {
        for (index, transaction) in journal.iter().enumerate() {
            if let Err(message) = validate_transaction_links(transaction, &journal[..index]) {
                diagnostics.push(ResolveDiagnostic {
                    code: "journal_link_mismatch".to_string(),
                    message: format!(
                        "transaction {} at index {} has invalid undo/redo links: {}",
                        transaction.transaction_id, index, message
                    ),
                    path: Some(transaction_journal_path(project_root)),
                });
                return Ok(journal[..index].to_vec());
            }
            if index > 0 {
                let previous = &journal[index - 1];
                if transaction.before_model_revision != previous.after_model_revision {
                    diagnostics.push(ResolveDiagnostic {
                        code: "journal_chain_mismatch".to_string(),
                        message: format!(
                            "transaction {} at index {} expected before revision {}, previous tip {}",
                            transaction.transaction_id,
                            index,
                            transaction.before_model_revision.0,
                            previous.after_model_revision.0
                        ),
                        path: Some(transaction_journal_path(project_root)),
                    });
                    return Ok(journal[..index].to_vec());
                }
            }
        }
        return Ok(journal.to_vec());
    }

    let mut candidate_objects = objects.clone();
    let mut candidate_shards = shards.to_vec();
    let mut valid = Vec::new();
    for (index, transaction) in journal.iter().enumerate() {
        if let Err(message) = validate_transaction_links(transaction, &valid) {
            diagnostics.push(ResolveDiagnostic {
                code: "journal_link_mismatch".to_string(),
                message: format!(
                    "transaction {} at index {} has invalid undo/redo links: {}",
                    transaction.transaction_id, index, message
                ),
                path: Some(transaction_journal_path(project_root)),
            });
            return Ok(journal[..index].to_vec());
        }
        if index > 0 {
            let previous = &journal[index - 1];
            if transaction.before_model_revision != previous.after_model_revision {
                diagnostics.push(ResolveDiagnostic {
                    code: "journal_chain_mismatch".to_string(),
                    message: format!(
                        "transaction {} at index {} expected before revision {}, previous tip {}",
                        transaction.transaction_id,
                        index,
                        transaction.before_model_revision.0,
                        previous.after_model_revision.0
                    ),
                    path: Some(transaction_journal_path(project_root)),
                });
                return Ok(journal[..index].to_vec());
            }
        }
        let mut candidate_journal = valid.clone();
        candidate_journal.push(transaction.clone());
        if let Err(error) = apply_transaction_operations_to_objects(
            project_id,
            &candidate_shards,
            &mut candidate_objects,
            transaction,
            &candidate_journal,
        ) {
            if is_promoted_replay_idempotency_error(&error) {
                let valid = validate_promoted_journal_links(project_root, journal, diagnostics)?;
                if valid.len() == journal.len() {
                    install_promoted_journal_state(project_root, shards, objects, journal)?;
                }
                return Ok(valid);
            }
            diagnostics.push(ResolveDiagnostic {
                code: "journal_replay_failed".to_string(),
                message: format!(
                    "transaction {} at index {} could not replay against current source shards: {}",
                    transaction.transaction_id, index, error
                ),
                path: Some(transaction_journal_path(project_root)),
            });
            return Ok(journal[..index].to_vec());
        }
        if let Err(error) = replay_journal_prefix_to_source_shards(
            project_root,
            &mut candidate_shards,
            &candidate_journal,
        ) {
            if is_promoted_replay_idempotency_error(&error) {
                let valid = validate_promoted_journal_links(project_root, journal, diagnostics)?;
                if valid.len() == journal.len() {
                    install_promoted_journal_state(project_root, shards, objects, journal)?;
                }
                return Ok(valid);
            }
            diagnostics.push(ResolveDiagnostic {
                code: "journal_replay_failed".to_string(),
                message: format!(
                    "transaction {} at index {} could not materialize source shards: {}",
                    transaction.transaction_id, index, error
                ),
                path: Some(transaction_journal_path(project_root)),
            });
            return Ok(journal[..index].to_vec());
        }
        valid.push(transaction.clone());
    }

    refresh_materialized_shard_objects(project_root, &candidate_shards, &mut candidate_objects)?;
    if let Some(tip) = journal.last() {
        let candidate_revision =
            compute_model_revision(project_id, &candidate_shards, &candidate_objects);
        if tip.after_model_revision != candidate_revision {
            if promoted_production_journal_matches_files(project_root, journal)? {
                return Ok(journal.to_vec());
            }
            diagnostics.push(ResolveDiagnostic {
                code: "journal_after_revision_mismatch".to_string(),
                message: format!(
                    "journal tip {} claimed after revision {}, computed {}",
                    tip.transaction_id, tip.after_model_revision.0, candidate_revision.0
                ),
                path: Some(transaction_journal_path(project_root)),
            });
            return Ok(Vec::new());
        }
    }

    objects.clear();
    objects.extend(candidate_objects);
    *shards = candidate_shards;
    Ok(journal.to_vec())
}

fn apply_promoted_journal_object_diffs(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    journal: &[TransactionRecord],
) {
    for transaction in journal {
        for operation in &transaction.operations {
            if let super::Operation::CreateSchematicSheet {
                sheet_id,
                relative_path,
                ..
            } = operation
            {
                objects.entry(*sheet_id).or_insert_with(|| DomainObject {
                    object_id: *sheet_id,
                    object_revision: ObjectRevision(0),
                    source_shard_id: Uuid::new_v5(
                        &Uuid::NAMESPACE_URL,
                        format!("datum-eda:source-shard:schematic/{relative_path}").as_bytes(),
                    ),
                    domain: "schematic".to_string(),
                    kind: "schematic_sheet".to_string(),
                });
            }
            if let super::Operation::CreateSchematicDefinition {
                definition_id,
                relative_path,
                ..
            } = operation
            {
                objects
                    .entry(*definition_id)
                    .or_insert_with(|| DomainObject {
                        object_id: *definition_id,
                        object_revision: ObjectRevision(0),
                        source_shard_id: Uuid::new_v5(
                            &Uuid::NAMESPACE_URL,
                            format!("datum-eda:source-shard:schematic/{relative_path}").as_bytes(),
                        ),
                        domain: "schematic".to_string(),
                        kind: "schematic_definition".to_string(),
                    });
            }
            if let super::Operation::CreatePoolLibraryObject {
                object_id,
                relative_path,
                object_kind,
                ..
            } = operation
            {
                objects.entry(*object_id).or_insert_with(|| DomainObject {
                    object_id: *object_id,
                    object_revision: ObjectRevision(0),
                    source_shard_id: Uuid::new_v5(
                        &Uuid::NAMESPACE_URL,
                        format!("datum-eda:source-shard:{relative_path}").as_bytes(),
                    ),
                    domain: "pool".to_string(),
                    kind: object_kind.clone(),
                });
            }
        }
        for object_id in &transaction.diff.modified {
            if let Some(object) = objects.get_mut(object_id) {
                object.object_revision = ObjectRevision(object.object_revision.0 + 1);
            }
        }
        for object_id in &transaction.diff.deleted {
            objects.remove(object_id);
        }
    }
}

fn install_promoted_journal_state(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    apply_promoted_journal_object_diffs(objects, journal);
    add_missing_journal_schematic_sheet_shards(project_root, shards, journal)?;
    replay_generated_evidence_shards(project_root, shards, journal)?;
    replay_pool_shards(project_root, shards, journal)?;
    refresh_materialized_shard_objects(project_root, shards, objects)
}

fn validate_promoted_journal_links(
    project_root: &Path,
    journal: &[TransactionRecord],
    diagnostics: &mut Vec<ResolveDiagnostic>,
) -> Result<Vec<TransactionRecord>, EngineError> {
    for (index, transaction) in journal.iter().enumerate() {
        if let Err(message) = validate_transaction_links(transaction, &journal[..index]) {
            diagnostics.push(ResolveDiagnostic {
                code: "journal_link_mismatch".to_string(),
                message: format!(
                    "transaction {} at index {} has invalid undo/redo links: {}",
                    transaction.transaction_id, index, message
                ),
                path: Some(transaction_journal_path(project_root)),
            });
            return Ok(journal[..index].to_vec());
        }
        if index > 0 {
            let previous = &journal[index - 1];
            if transaction.before_model_revision != previous.after_model_revision {
                diagnostics.push(ResolveDiagnostic {
                    code: "journal_chain_mismatch".to_string(),
                    message: format!(
                        "transaction {} at index {} expected before revision {}, previous tip {}",
                        transaction.transaction_id,
                        index,
                        transaction.before_model_revision.0,
                        previous.after_model_revision.0
                    ),
                    path: Some(transaction_journal_path(project_root)),
                });
                return Ok(journal[..index].to_vec());
            }
        }
    }
    Ok(journal.to_vec())
}

fn is_promoted_replay_idempotency_error(error: &EngineError) -> bool {
    matches!(error, EngineError::Validation(message) if message.contains("already exists"))
}

fn promoted_production_journal_matches_files(
    project_root: &Path,
    journal: &[TransactionRecord],
) -> Result<bool, EngineError> {
    if journal.is_empty() {
        return Ok(false);
    }
    let mut final_values = BTreeMap::new();
    for transaction in journal {
        for operation in &transaction.operations {
            let Some((kind, object_id, value, delete)) = production_operation_write(operation)
            else {
                return Ok(false);
            };
            if delete {
                final_values.insert(production_relative_path(kind, object_id)?, None);
            } else {
                final_values.insert(
                    production_relative_path(kind, object_id)?,
                    Some(value.clone()),
                );
            }
        }
    }
    for (relative_path, expected_value) in final_values {
        let path = project_root.join(relative_path);
        let Some(expected_value) = expected_value else {
            if !path.exists() {
                continue;
            }
            return Ok(false);
        };
        if !path.exists() {
            return Ok(false);
        }
        let file_value = read_json_value(&path)?;
        if canonical_json_hash(&file_value)? != canonical_json_hash(&expected_value)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn apply_transaction_operations_to_objects(
    project_id: &Uuid,
    shards: &[SourceShardRef],
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    transaction: &TransactionRecord,
    journal_prefix: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut model = DesignModel {
        project: ProjectManifestSummary {
            project_id: *project_id,
            name: String::new(),
            schema_version: None,
        },
        model_revision: compute_model_revision(project_id, shards, objects),
        source_shards: shards.to_vec(),
        objects: objects.clone(),
        component_instances: BTreeMap::new(),
        relationships: BTreeMap::new(),
        relationship_statuses: BTreeMap::new(),
        variants: BTreeMap::new(),
        variant_populations: BTreeMap::new(),
        import_map: BTreeMap::new(),
        zone_fills: BTreeMap::new(),
        manufacturing_plans: BTreeMap::new(),
        panel_projections: BTreeMap::new(),
        output_jobs: BTreeMap::new(),
        output_job_runs: BTreeMap::new(),
        artifact_runs: BTreeMap::new(),
        check_runs: BTreeMap::new(),
        artifact_metadata: BTreeMap::new(),
        proposals: BTreeMap::new(),
        journal: journal_prefix.to_vec(),
        journal_cursor: JournalCursor {
            applied_transaction_count: 0,
        },
        diagnostics: Vec::new(),
    };
    let mut diff = CommitDiff::default();
    for operation in &transaction.operations {
        apply_operation(&mut model, operation, &mut diff)?;
    }
    objects.clear();
    objects.extend(model.objects);
    Ok(())
}

pub(super) fn replay_journal_prefix_to_source_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    for shard in shards.iter_mut().filter(|shard| {
        matches!(
            shard.kind,
            SourceShardKind::ProjectManifest
                | SourceShardKind::SchematicRoot
                | SourceShardKind::BoardRoot
                | SourceShardKind::RulesRoot
                | SourceShardKind::SchematicSheet
        )
    }) {
        let mut value = match read_json_value(&shard.path) {
            Ok(value) => value,
            Err(EngineError::Io(error))
                if error.kind() == std::io::ErrorKind::NotFound
                    && shard.kind == SourceShardKind::SchematicSheet =>
            {
                continue;
            }
            Err(error) => return Err(error),
        };
        if replay_journal_shard_value(&shard.kind, &mut value, journal)? {
            let content_hash = canonical_json_hash(&value)?;
            shard.dirty_state = dirty_state_for_materialized_shard(
                project_root,
                &shard.relative_path,
                &content_hash,
            );
            shard.content_hash = content_hash;
        }
    }
    replay_production_shards(project_root, shards, journal)?;
    replay_schematic_shards(project_root, shards, journal)?;
    replay_generated_evidence_shards(project_root, shards, journal)?;
    replay_pool_shards(project_root, shards, journal)?;
    replay_import_map_shards(project_root, shards, journal)?;
    replay_forward_annotation_review_shard(project_root, shards, journal)?;
    replay_authored_context_shards(project_root, shards, journal)?;
    super::sort_source_shards(shards);
    Ok(())
}

fn replay_production_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut production_values = Vec::new();
    for shard in shards.iter().filter(|shard| {
        matches!(
            shard.kind,
            SourceShardKind::ManufacturingPlan
                | SourceShardKind::PanelProjection
                | SourceShardKind::OutputJob
        )
    }) {
        if !shard.path.exists() {
            continue;
        }
        production_values.push((
            shard.relative_path.clone(),
            (
                shard.kind.clone(),
                serde_json::to_value(read_json_value(&shard.path)?)?,
            ),
        ));
    }
    for transaction in journal {
        for operation in &transaction.operations {
            let Some((kind, object_id, value, delete)) = production_operation_write(operation)
            else {
                continue;
            };
            let relative_path = production_relative_path(kind.clone(), object_id)?;
            if delete {
                production_values.retain(|(path, _)| path != &relative_path);
            } else if let Some((_, entry)) = production_values
                .iter_mut()
                .find(|(path, _)| path == &relative_path)
            {
                *entry = (kind, value.clone());
            } else {
                production_values.push((relative_path, (kind, value.clone())));
            }
        }
    }
    shards.retain(|shard| {
        !matches!(
            shard.kind,
            SourceShardKind::ManufacturingPlan
                | SourceShardKind::PanelProjection
                | SourceShardKind::OutputJob
        )
    });
    for (relative_path, (kind, value)) in production_values {
        shards.push(source_shard_ref_for_value(
            project_root,
            kind,
            relative_path,
            &value,
        )?);
    }
    Ok(())
}

fn replay_authored_context_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards.iter().filter(|shard| {
        matches!(
            shard.kind,
            SourceShardKind::Relationship
                | SourceShardKind::VariantOverlay
                | SourceShardKind::ComponentInstance
        )
    }) {
        if !shard.path.exists() {
            continue;
        }
        let Ok(value) = read_json_value(&shard.path) else {
            continue;
        };
        values.push((shard.relative_path.clone(), (shard.kind.clone(), value)));
    }
    for transaction in journal {
        for operation in &transaction.operations {
            if let Some((object_id, value, delete)) = component_instance_operation_write(operation)
            {
                let kind = SourceShardKind::ComponentInstance;
                let relative_path =
                    super::operation_application_component_instance::authored_relative_path(
                        object_id,
                    );
                if delete {
                    values.retain(|(path, _)| path != &relative_path);
                } else if let Some((_, entry)) =
                    values.iter_mut().find(|(path, _)| path == &relative_path)
                {
                    *entry = (kind.clone(), wrap_component_instance_payload(value.clone()));
                } else {
                    values.push((
                        relative_path,
                        (kind.clone(), wrap_component_instance_payload(value.clone())),
                    ));
                }
                continue;
            }
            let Some((kind, object_id, value, delete)) = relationship_operation_write(operation)
            else {
                continue;
            };
            let relative_path = super::operation_application_relationship::authored_relative_path(
                kind.clone(),
                object_id,
            )?;
            if delete {
                values.retain(|(path, _)| path != &relative_path);
            } else if let Some((_, entry)) =
                values.iter_mut().find(|(path, _)| path == &relative_path)
            {
                *entry = (kind.clone(), wrap_payload(&kind, value.clone()));
            } else {
                values.push((
                    relative_path,
                    (kind.clone(), wrap_payload(&kind, value.clone())),
                ));
            }
        }
    }
    shards.retain(|shard| {
        !matches!(
            shard.kind,
            SourceShardKind::Relationship
                | SourceShardKind::VariantOverlay
                | SourceShardKind::ComponentInstance
        )
    });
    for (relative_path, (kind, value)) in values {
        shards.push(source_shard_ref_for_value(
            project_root,
            kind,
            relative_path,
            &value,
        )?);
    }
    Ok(())
}

pub(super) fn replay_import_map_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::ImportMap)
    {
        if !shard.path.exists() {
            continue;
        }
        let Ok(value) = read_json_value(&shard.path) else {
            continue;
        };
        values.push((shard.relative_path.clone(), value));
    }
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                super::Operation::CreateImportMapShard {
                    relative_path,
                    shard,
                } => {
                    if let Some((_, value)) =
                        values.iter_mut().find(|(path, _)| path == relative_path)
                    {
                        *value = shard.clone();
                    } else {
                        values.push((relative_path.clone(), shard.clone()));
                    }
                }
                super::Operation::DeleteImportMapShard { relative_path, .. } => {
                    values.retain(|(path, _)| path != relative_path);
                }
                _ => {}
            }
        }
    }
    shards.retain(|shard| shard.kind != SourceShardKind::ImportMap);
    for (relative_path, value) in values {
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::ImportMap,
            relative_path,
            &value,
        )?);
    }
    Ok(())
}
