use serde::Deserialize;
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(super) struct ArtifactFilePreviewPayload {
    pub(super) file: std::path::PathBuf,
    pub(super) preview_kind: String,
    pub(super) hash_matches_metadata: bool,
    #[serde(default)]
    pub(super) primitive_count: usize,
    #[serde(default)]
    pub(super) primitives: Vec<ArtifactFilePreviewPrimitivePayload>,
    #[serde(default)]
    pub(super) inspection: serde_json::Value,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(super) struct ArtifactFilePreviewPrimitivePayload {
    #[serde(default)]
    pub(super) exact_geometry: Option<serde_json::Value>,
    pub(super) kind: String,
    #[serde(default)]
    pub(super) aperture_diameter_nm: Option<i64>,
    #[serde(default)]
    pub(super) aperture_width_nm: Option<i64>,
    #[serde(default)]
    pub(super) aperture_height_nm: Option<i64>,
    #[serde(default)]
    pub(super) tool: Option<String>,
    #[serde(default)]
    pub(super) diameter_mm: Option<String>,
    #[serde(default)]
    pub(super) points: Vec<ArtifactFilePreviewPointPayload>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub(super) struct ArtifactFilePreviewPointPayload {
    pub(super) x_nm: i64,
    pub(super) y_nm: i64,
}
