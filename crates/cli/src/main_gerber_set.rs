use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberSetArtifactView {
    pub(crate) kind: String,
    pub(crate) layer_id: Option<i32>,
    pub(crate) layer_name: Option<String>,
    pub(crate) filename: String,
    pub(crate) output_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberSetExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) output_dir: String,
    pub(crate) prefix: String,
    pub(crate) written_count: usize,
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
    ];
    if !report.artifacts.is_empty() {
        lines.push("artifacts:".to_string());
        for artifact in &report.artifacts {
            lines.push(format!("  - {}:{}", artifact.kind, artifact.output_path));
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
