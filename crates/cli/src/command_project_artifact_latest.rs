use eda_engine::substrate::{ArtifactMetadata, DesignModel, Operation};
use uuid::Uuid;

pub(crate) fn latest_artifact_id(
    model: &DesignModel,
    artifacts: &[ArtifactMetadata],
) -> Option<Uuid> {
    let existing = artifacts
        .iter()
        .map(|artifact| artifact.artifact_id)
        .collect::<std::collections::BTreeSet<_>>();
    let latest_journaled = model
        .journal
        .iter()
        .enumerate()
        .flat_map(|(transaction_index, transaction)| {
            let existing = &existing;
            transaction.operations.iter().enumerate().filter_map(
                move |(operation_index, operation)| match operation {
                    Operation::SetArtifactMetadata { artifact_id, .. }
                        if existing.contains(artifact_id) =>
                    {
                        Some((transaction_index, operation_index, *artifact_id))
                    }
                    _ => None,
                },
            )
        })
        .max_by_key(|(transaction_index, operation_index, artifact_id)| {
            (*transaction_index, *operation_index, *artifact_id)
        })
        .map(|(_, _, artifact_id)| artifact_id);
    if latest_journaled.is_some() {
        return latest_journaled;
    }

    artifacts
        .iter()
        .max_by(|a, b| {
            a.model_revision
                .0
                .cmp(&b.model_revision.0)
                .then_with(|| a.artifact_id.cmp(&b.artifact_id))
        })
        .map(|artifact| artifact.artifact_id)
}
