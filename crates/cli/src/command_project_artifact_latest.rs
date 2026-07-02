use eda_engine::api::native_write::artifacts::latest_journaled_artifact_id;
use eda_engine::substrate::{ArtifactMetadata, DesignModel};
use uuid::Uuid;

pub(crate) fn latest_artifact_id(
    model: &DesignModel,
    artifacts: &[ArtifactMetadata],
) -> Option<Uuid> {
    let existing = artifacts
        .iter()
        .map(|artifact| artifact.artifact_id)
        .collect::<std::collections::BTreeSet<_>>();
    if let Some(latest_journaled) = latest_journaled_artifact_id(model, &existing) {
        return Some(latest_journaled);
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
