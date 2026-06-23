use eda_engine::substrate::{ArtifactMetadata, OutputJobRun};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberSetArtifactView {
    pub(crate) kind: String,
    pub(crate) layer_id: Option<i32>,
    pub(crate) layer_name: Option<String>,
    pub(crate) filename: String,
    pub(crate) output_path: String,
    pub(crate) sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberSetExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) output_dir: String,
    pub(crate) prefix: String,
    pub(crate) written_count: usize,
    pub(crate) artifact_manifest_path: String,
    pub(crate) artifact_metadata: ArtifactMetadata,
    pub(crate) output_job_run: Option<OutputJobRun>,
    pub(crate) output_job_run_path: Option<String>,
    pub(crate) artifacts: Vec<NativeProjectGerberSetArtifactView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberSetValidationView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) output_dir: String,
    pub(crate) prefix: String,
    pub(crate) expected_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) mismatched_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) artifact_id: Option<String>,
    pub(crate) artifact_manifest_path: Option<String>,
    pub(crate) artifact_validation_state: Option<String>,
    pub(crate) artifact_file_hash_mismatch_count: usize,
    pub(crate) matched: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) mismatched: Vec<String>,
    pub(crate) extra: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberSetComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) output_dir: String,
    pub(crate) prefix: String,
    pub(crate) expected_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) mismatched_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) matched: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) mismatched: Vec<String>,
    pub(crate) extra: Vec<String>,
}

pub(crate) fn render_native_project_gerber_set_export_text(
    report: &NativeProjectGerberSetExportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("output_dir: {}", report.output_dir),
        format!("prefix: {}", report.prefix),
        format!("written_count: {}", report.written_count),
        format!("artifact_manifest_path: {}", report.artifact_manifest_path),
        format!("artifact_id: {}", report.artifact_metadata.artifact_id),
        format!(
            "model_revision: {}",
            report.artifact_metadata.model_revision.0
        ),
        format!(
            "validation_state: {:?}",
            report.artifact_metadata.validation_state
        ),
    ];
    if let Some(output_job_run) = &report.output_job_run {
        lines.push(format!("output_job_run: {}", output_job_run.run_id));
    }
    if let Some(output_job_run_path) = &report.output_job_run_path {
        lines.push(format!("output_job_run_path: {output_job_run_path}"));
    }
    if !report.artifacts.is_empty() {
        lines.push("artifacts:".to_string());
        for artifact in &report.artifacts {
            lines.push(format!(
                "  - {}:{} {}",
                artifact.kind, artifact.output_path, artifact.sha256
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_gerber_set_validation_text(
    report: &NativeProjectGerberSetValidationView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("output_dir: {}", report.output_dir),
        format!("prefix: {}", report.prefix),
        format!("expected_count: {}", report.expected_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("mismatched_count: {}", report.mismatched_count),
        format!("extra_count: {}", report.extra_count),
        format!(
            "artifact_file_hash_mismatch_count: {}",
            report.artifact_file_hash_mismatch_count
        ),
    ];
    if let Some(artifact_id) = &report.artifact_id {
        lines.push(format!("artifact_id: {artifact_id}"));
    }
    if let Some(path) = &report.artifact_manifest_path {
        lines.push(format!("artifact_manifest_path: {path}"));
    }
    if let Some(state) = &report.artifact_validation_state {
        lines.push(format!("artifact_validation_state: {state}"));
    }
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for file in &report.matched {
            lines.push(format!("  {file}"));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for file in &report.missing {
            lines.push(format!("  {file}"));
        }
    }
    if !report.mismatched.is_empty() {
        lines.push("mismatched:".to_string());
        for file in &report.mismatched {
            lines.push(format!("  {file}"));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for file in &report.extra {
            lines.push(format!("  {file}"));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_gerber_set_comparison_text(
    report: &NativeProjectGerberSetComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("output_dir: {}", report.output_dir),
        format!("prefix: {}", report.prefix),
        format!("expected_count: {}", report.expected_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("mismatched_count: {}", report.mismatched_count),
        format!("extra_count: {}", report.extra_count),
    ];
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for file in &report.matched {
            lines.push(format!("  {file}"));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for file in &report.missing {
            lines.push(format!("  {file}"));
        }
    }
    if !report.mismatched.is_empty() {
        lines.push("mismatched:".to_string());
        for file in &report.mismatched {
            lines.push(format!("  {file}"));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for file in &report.extra {
            lines.push(format!("  {file}"));
        }
    }
    lines.join("\n")
}
