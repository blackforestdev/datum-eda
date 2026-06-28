use super::*;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver, SourceShardKind,
};
use std::collections::BTreeMap;

const FORWARD_ANNOTATION_REVIEW_PATH: &str = ".datum/forward_annotation_review/review.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeForwardAnnotationReviewSidecar {
    schema_version: u32,
    #[serde(default)]
    reviews: BTreeMap<String, NativeForwardAnnotationReviewRecord>,
}

pub(crate) fn load_forward_annotation_review(
    root: &Path,
) -> Result<BTreeMap<String, NativeForwardAnnotationReviewRecord>> {
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if model
        .source_shards
        .iter()
        .any(|shard| shard.kind == SourceShardKind::ForwardAnnotationReview)
    {
        let sidecar: NativeForwardAnnotationReviewSidecar = serde_json::from_value(
            model
                .materialized_source_shard_value(SourceShardKind::ForwardAnnotationReview)
                .context("failed to materialize forward-annotation review sidecar")?,
        )
        .context("failed to parse resolver-materialized forward-annotation review sidecar")?;
        return Ok(sidecar.reviews);
    }
    let manifest: NativeProjectManifest = serde_json::from_value(
        model
            .materialized_source_shard_value(SourceShardKind::ProjectManifest)
            .context("failed to materialize project manifest")?,
    )
    .context("failed to parse resolver-materialized project manifest")?;
    Ok(manifest.forward_annotation_review)
}

pub(crate) fn write_forward_annotation_review(
    root: &Path,
    reviews: &BTreeMap<String, NativeForwardAnnotationReviewRecord>,
) -> Result<()> {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let previous_review = if model
        .source_shards
        .iter()
        .any(|shard| shard.kind == SourceShardKind::ForwardAnnotationReview)
    {
        Some(
            model
                .materialized_source_shard_value(SourceShardKind::ForwardAnnotationReview)
                .context("failed to materialize previous forward-annotation review sidecar")?,
        )
    } else {
        None
    };
    let review = serde_json::to_value(NativeForwardAnnotationReviewSidecar {
        schema_version: 1,
        reviews: reviews.clone(),
    })?;
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-forward-annotation".to_string(),
                source: CommitSource::Cli,
                reason: "update forward-annotation review state".to_string(),
            },
            operations: vec![Operation::SetForwardAnnotationReview {
                relative_path: FORWARD_ANNOTATION_REVIEW_PATH.to_string(),
                previous_review,
                review,
            }],
        },
    )?;
    Ok(())
}

pub(crate) fn clear_forward_annotation_review_sidecar(root: &Path) -> Result<()> {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if !model
        .source_shards
        .iter()
        .any(|shard| shard.kind == SourceShardKind::ForwardAnnotationReview)
    {
        return Ok(());
    }
    let review = model
        .materialized_source_shard_value(SourceShardKind::ForwardAnnotationReview)
        .context("failed to materialize previous forward-annotation review sidecar")?;
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-forward-annotation".to_string(),
                source: CommitSource::Cli,
                reason: "clear forward-annotation review state".to_string(),
            },
            operations: vec![Operation::DeleteForwardAnnotationReview {
                relative_path: FORWARD_ANNOTATION_REVIEW_PATH.to_string(),
                review,
            }],
        },
    )?;
    Ok(())
}
