use super::*;
use eda_engine::api::native_write::forward_annotation::{
    build_clear_forward_annotation_review, build_set_forward_annotation_review,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::{ProjectResolver, SourceShardKind};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeForwardAnnotationReviewSidecar {
    schema_version: u32,
    #[serde(default)]
    reviews: BTreeMap<String, NativeForwardAnnotationReviewRecord>,
}

fn forward_annotation_provenance(reason: &str) -> Result<WriteProvenance> {
    Ok(WriteProvenance::new(
        "datum-eda-forward-annotation",
        cli_commit_source()?,
        reason,
    ))
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
    let prepared = build_set_forward_annotation_review(
        &model,
        forward_annotation_provenance("update forward-annotation review state")?,
        previous_review,
        review,
    )?;
    commit_prepared(&mut model, root, prepared)?;
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
    let prepared = build_clear_forward_annotation_review(
        &model,
        forward_annotation_provenance("clear forward-annotation review state")?,
        review,
    )?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(())
}
