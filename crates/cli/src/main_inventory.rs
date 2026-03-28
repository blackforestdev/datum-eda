use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBomExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) bom_path: String,
    pub(crate) rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBomDriftView {
    pub(crate) reference: String,
    pub(crate) fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBomComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) bom_path: String,
    pub(crate) expected_count: usize,
    pub(crate) actual_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) drift_count: usize,
    pub(crate) matched: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) extra: Vec<String>,
    pub(crate) drift: Vec<NativeProjectBomDriftView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBomInspectionRowView {
    pub(crate) reference: String,
    pub(crate) value: String,
    pub(crate) part_uuid: String,
    pub(crate) package_uuid: String,
    pub(crate) layer: i32,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) rotation_deg: i32,
    pub(crate) locked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBomInspectionView {
    pub(crate) action: String,
    pub(crate) bom_path: String,
    pub(crate) row_count: usize,
    pub(crate) rows: Vec<NativeProjectBomInspectionRowView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPnpExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) pnp_path: String,
    pub(crate) rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPnpDriftView {
    pub(crate) reference: String,
    pub(crate) fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPnpComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) pnp_path: String,
    pub(crate) expected_count: usize,
    pub(crate) actual_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) drift_count: usize,
    pub(crate) matched: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) extra: Vec<String>,
    pub(crate) drift: Vec<NativeProjectPnpDriftView>,
}

pub(crate) fn render_native_project_bom_export_text(report: &NativeProjectBomExportView) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("bom_path: {}", report.bom_path),
        format!("rows: {}", report.rows),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_bom_comparison_text(
    report: &NativeProjectBomComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("bom_path: {}", report.bom_path),
        format!("expected_count: {}", report.expected_count),
        format!("actual_count: {}", report.actual_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
        format!("drift_count: {}", report.drift_count),
    ];
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for entry in &report.matched {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for entry in &report.missing {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for entry in &report.extra {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.drift.is_empty() {
        lines.push("drift:".to_string());
        for entry in &report.drift {
            lines.push(format!("- {} [{}]", entry.reference, entry.fields.join(", ")));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_bom_inspection_text(
    report: &NativeProjectBomInspectionView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("bom_path: {}", report.bom_path),
        format!("row_count: {}", report.row_count),
    ];
    if !report.rows.is_empty() {
        lines.push("rows:".to_string());
        for row in &report.rows {
            lines.push(format!(
                "- reference={} value={} part_uuid={} package_uuid={} layer={} x_nm={} y_nm={} rotation_deg={} locked={}",
                row.reference,
                row.value,
                row.part_uuid,
                row.package_uuid,
                row.layer,
                row.x_nm,
                row.y_nm,
                row.rotation_deg,
                row.locked
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_pnp_export_text(report: &NativeProjectPnpExportView) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("pnp_path: {}", report.pnp_path),
        format!("rows: {}", report.rows),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_pnp_comparison_text(
    report: &NativeProjectPnpComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("pnp_path: {}", report.pnp_path),
        format!("expected_count: {}", report.expected_count),
        format!("actual_count: {}", report.actual_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
        format!("drift_count: {}", report.drift_count),
    ];
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for entry in &report.matched {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for entry in &report.missing {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for entry in &report.extra {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.drift.is_empty() {
        lines.push("drift:".to_string());
        for entry in &report.drift {
            lines.push(format!("- {} fields={}", entry.reference, entry.fields.join(",")));
        }
    }
    lines.join("\n")
}
