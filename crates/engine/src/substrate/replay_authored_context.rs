use super::component_instance_journal_ops::{
    component_instance_operation_write, wrap_payload as wrap_component_instance_payload,
};
use super::relationship_journal_ops::{relationship_operation_write, wrap_payload};
use super::source_shard_ref_builders::source_shard_ref_for_value;
use super::{EngineError, SourceShardKind, SourceShardRef, TransactionRecord, read_json_value};
use std::path::Path;

pub(super) fn replay_authored_context_shards(
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
                | SourceShardKind::ElectricalIdentity
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
            if let Some((id, record, delete)) = super::electrical_identity_store::write(operation) {
                let path = super::electrical_identity_store::relative_path(id);
                values.retain(|(p, _)| p != &path);
                if !delete {
                    values.push((
                        path,
                        (
                            SourceShardKind::ElectricalIdentity,
                            super::electrical_identity_store::wrapper(record)?,
                        ),
                    ));
                }
                continue;
            }
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
                | SourceShardKind::ElectricalIdentity
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
