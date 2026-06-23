use super::*;
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
    let path = root.join(FORWARD_ANNOTATION_REVIEW_PATH);
    if path.exists() {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sidecar: NativeForwardAnnotationReviewSidecar = serde_json::from_str(&text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        return Ok(sidecar.reviews);
    }
    Ok(load_native_project(root)?
        .manifest
        .forward_annotation_review)
}

pub(crate) fn write_forward_annotation_review(
    root: &Path,
    reviews: &BTreeMap<String, NativeForwardAnnotationReviewRecord>,
) -> Result<()> {
    let path = root.join(FORWARD_ANNOTATION_REVIEW_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    write_canonical_json(
        &path,
        &NativeForwardAnnotationReviewSidecar {
            schema_version: 1,
            reviews: reviews.clone(),
        },
    )
}
