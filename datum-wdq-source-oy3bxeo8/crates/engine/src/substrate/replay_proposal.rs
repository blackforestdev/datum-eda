use std::path::Path;

use super::proposal_journal_ops::{proposal_operation_write, proposal_relative_path};
use super::source_shard_ref_builders::source_shard_ref_for_value;
use super::{EngineError, SourceShardKind, SourceShardRef, TransactionRecord, read_json_value};

pub(super) fn replay_proposal_shards(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut values = Vec::new();
    for shard in shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::ProposalMetadata)
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
            let Some((proposal_id, relative_path, next_value)) =
                proposal_operation_write(operation)
            else {
                continue;
            };
            let expected_relative_path = proposal_relative_path(proposal_id);
            if relative_path != expected_relative_path {
                return Err(EngineError::Validation(format!(
                    "proposal metadata {proposal_id} must use `{expected_relative_path}`, got `{relative_path}`"
                )));
            }
            if let Some(next_value) = next_value {
                if let Some((_, value)) = values.iter_mut().find(|(path, _)| path == &relative_path)
                {
                    *value = next_value.clone();
                } else {
                    values.push((relative_path, next_value.clone()));
                }
            } else {
                values.retain(|(path, _)| path != &relative_path);
            }
        }
    }
    shards.retain(|shard| shard.kind != SourceShardKind::ProposalMetadata);
    for (relative_path, value) in values {
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::ProposalMetadata,
            relative_path,
            &value,
        )?);
    }
    Ok(())
}
