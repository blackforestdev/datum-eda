use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionArtifactSummary {
    pub artifact_id: String,
    pub kind: String,
    pub project_id: Option<String>,
    pub model_revision: Option<String>,
    pub output_job: Option<String>,
    pub variant: Option<String>,
    pub generator_version: Option<String>,
    pub output_dir: Option<String>,
    pub validation_state: Option<String>,
    pub file_count: usize,
    pub files: Vec<ProductionArtifactFileSummary>,
    pub production_projection_count: usize,
    pub production_projections: Vec<ProductionArtifactProjectionSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionArtifactFileSummary {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionArtifactProjectionSummary {
    pub projection_kind: String,
    pub projection_contract: String,
    pub model_revision: String,
    pub byte_count: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionArtifactDetail {
    pub artifact_id: String,
    pub kind: String,
    pub output_dir: Option<String>,
    pub validation_state: String,
    pub file_count: usize,
    pub files: Vec<ProductionArtifactFileSummary>,
    pub focused_file: Option<ProductionArtifactFileSummary>,
    pub focused_preview: Option<ProductionArtifactFilePreviewSummary>,
    pub production_projection_count: usize,
    pub production_projections: Vec<ProductionArtifactProjectionSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionArtifactFilePreviewSummary {
    pub file: String,
    pub preview_kind: String,
    pub hash_matches_metadata: bool,
    pub primitive_count: usize,
    pub primitives: Vec<ProductionArtifactPreviewPrimitive>,
    pub geometry_count: Option<usize>,
    pub hit_count: Option<usize>,
    pub row_count: Option<usize>,
    pub csv_columns: Vec<String>,
    pub csv_rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionArtifactPreviewPrimitive {
    pub kind: String,
    pub aperture_diameter_nm: Option<i64>,
    pub aperture_width_nm: Option<i64>,
    pub aperture_height_nm: Option<i64>,
    pub tool: Option<String>,
    pub diameter_mm: Option<String>,
    pub points: Vec<ProductionArtifactPreviewPoint>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ProductionArtifactPreviewPoint {
    pub x_nm: i64,
    pub y_nm: i64,
}
