use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectDrillExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) drill_path: String,
    pub(crate) rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectDrillValidationView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) drill_path: String,
    pub(crate) matches_expected: bool,
    pub(crate) expected_bytes: usize,
    pub(crate) actual_bytes: usize,
    pub(crate) rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectDrillComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) drill_path: String,
    pub(crate) expected_row_count: usize,
    pub(crate) actual_row_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) drift_count: usize,
    pub(crate) matched: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) extra: Vec<String>,
    pub(crate) drift: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectDrillInspectionRowView {
    pub(crate) via_uuid: String,
    pub(crate) net_uuid: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) drill_nm: i64,
    pub(crate) diameter_nm: i64,
    pub(crate) from_layer: i32,
    pub(crate) to_layer: i32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectDrillInspectionView {
    pub(crate) action: String,
    pub(crate) drill_path: String,
    pub(crate) row_count: usize,
    pub(crate) rows: Vec<NativeProjectDrillInspectionRowView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectExcellonDrillExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) drill_path: String,
    pub(crate) via_count: usize,
    pub(crate) component_pad_count: usize,
    pub(crate) hit_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) tools: Vec<NativeProjectExcellonDrillToolView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectExcellonDrillValidationView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) drill_path: String,
    pub(crate) matches_expected: bool,
    pub(crate) expected_bytes: usize,
    pub(crate) actual_bytes: usize,
    pub(crate) via_count: usize,
    pub(crate) component_pad_count: usize,
    pub(crate) hit_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) tools: Vec<NativeProjectExcellonDrillToolView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectExcellonDrillToolView {
    pub(crate) tool: String,
    pub(crate) diameter_mm: String,
    pub(crate) hits: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectExcellonDrillInspectionView {
    pub(crate) action: String,
    pub(crate) drill_path: String,
    pub(crate) metric: bool,
    pub(crate) tool_count: usize,
    pub(crate) hit_count: usize,
    pub(crate) tools: Vec<NativeProjectExcellonDrillToolView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectExcellonDrillHitDriftView {
    pub(crate) diameter_mm: String,
    pub(crate) expected_hits: usize,
    pub(crate) actual_hits: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectExcellonDrillComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) drill_path: String,
    pub(crate) expected_tool_count: usize,
    pub(crate) actual_tool_count: usize,
    pub(crate) expected_hit_count: usize,
    pub(crate) actual_hit_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) hit_drift_count: usize,
    pub(crate) matched: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) extra: Vec<String>,
    pub(crate) hit_drift: Vec<NativeProjectExcellonDrillHitDriftView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectDrillHoleClassBucketView {
    pub(crate) class: String,
    pub(crate) from_layer: i32,
    pub(crate) to_layer: i32,
    pub(crate) via_count: usize,
    pub(crate) component_pad_count: usize,
    pub(crate) hit_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) tools: Vec<NativeProjectExcellonDrillToolView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectDrillHoleClassReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) copper_layer_count: usize,
    pub(crate) via_count: usize,
    pub(crate) component_pad_count: usize,
    pub(crate) hit_count: usize,
    pub(crate) class_count: usize,
    pub(crate) classes: Vec<NativeProjectDrillHoleClassBucketView>,
}

pub(crate) fn render_native_project_drill_export_text(
    report: &NativeProjectDrillExportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("drill_path: {}", report.drill_path),
        format!("rows: {}", report.rows),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_drill_validation_text(
    report: &NativeProjectDrillValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("drill_path: {}", report.drill_path),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("rows: {}", report.rows),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_drill_comparison_text(
    report: &NativeProjectDrillComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("drill_path: {}", report.drill_path),
        format!("expected_row_count: {}", report.expected_row_count),
        format!("actual_row_count: {}", report.actual_row_count),
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
            lines.push(format!("- {}", entry));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_drill_inspection_text(
    report: &NativeProjectDrillInspectionView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("drill_path: {}", report.drill_path),
        format!("row_count: {}", report.row_count),
    ];
    if !report.rows.is_empty() {
        lines.push("rows:".to_string());
        for row in &report.rows {
            lines.push(format!(
                "- {} {} {} {} {} {} {} {}",
                row.via_uuid,
                row.net_uuid,
                row.x_nm,
                row.y_nm,
                row.drill_nm,
                row.diameter_nm,
                row.from_layer,
                row.to_layer
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_excellon_drill_export_text(
    report: &NativeProjectExcellonDrillExportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("drill_path: {}", report.drill_path),
        format!("via_count: {}", report.via_count),
        format!("component_pad_count: {}", report.component_pad_count),
        format!("hit_count: {}", report.hit_count),
        format!("tool_count: {}", report.tool_count),
    ];
    if !report.tools.is_empty() {
        lines.push("tools:".to_string());
        for tool in &report.tools {
            lines.push(format!(
                "- {} diameter_mm={} hits={}",
                tool.tool, tool.diameter_mm, tool.hits
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_excellon_drill_validation_text(
    report: &NativeProjectExcellonDrillValidationView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("drill_path: {}", report.drill_path),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("via_count: {}", report.via_count),
        format!("component_pad_count: {}", report.component_pad_count),
        format!("hit_count: {}", report.hit_count),
        format!("tool_count: {}", report.tool_count),
    ];
    if !report.tools.is_empty() {
        lines.push("tools:".to_string());
        for tool in &report.tools {
            lines.push(format!(
                "- {} diameter_mm={} hits={}",
                tool.tool, tool.diameter_mm, tool.hits
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_excellon_drill_inspection_text(
    report: &NativeProjectExcellonDrillInspectionView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("drill_path: {}", report.drill_path),
        format!("metric: {}", report.metric),
        format!("tool_count: {}", report.tool_count),
        format!("hit_count: {}", report.hit_count),
    ];
    if !report.tools.is_empty() {
        lines.push("tools:".to_string());
        for tool in &report.tools {
            lines.push(format!(
                "- {} diameter_mm={} hits={}",
                tool.tool, tool.diameter_mm, tool.hits
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_excellon_drill_comparison_text(
    report: &NativeProjectExcellonDrillComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("drill_path: {}", report.drill_path),
        format!("expected_tool_count: {}", report.expected_tool_count),
        format!("actual_tool_count: {}", report.actual_tool_count),
        format!("expected_hit_count: {}", report.expected_hit_count),
        format!("actual_hit_count: {}", report.actual_hit_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
        format!("hit_drift_count: {}", report.hit_drift_count),
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
    if !report.hit_drift.is_empty() {
        lines.push("hit_drift:".to_string());
        for entry in &report.hit_drift {
            lines.push(format!(
                "- diameter_mm={} expected_hits={} actual_hits={}",
                entry.diameter_mm, entry.expected_hits, entry.actual_hits
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_drill_hole_class_report_text(
    report: &NativeProjectDrillHoleClassReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("copper_layer_count: {}", report.copper_layer_count),
        format!("via_count: {}", report.via_count),
        format!("component_pad_count: {}", report.component_pad_count),
        format!("hit_count: {}", report.hit_count),
        format!("class_count: {}", report.class_count),
    ];
    if !report.classes.is_empty() {
        lines.push("classes:".to_string());
        for class in &report.classes {
            lines.push(format!(
                "- class={} span=L{}-L{} via_count={} component_pad_count={} hit_count={} tool_count={}",
                class.class,
                class.from_layer,
                class.to_layer,
                class.via_count,
                class.component_pad_count,
                class.hit_count,
                class.tool_count
            ));
            for tool in &class.tools {
                lines.push(format!(
                    "  tool={} diameter_mm={} hits={}",
                    tool.tool, tool.diameter_mm, tool.hits
                ));
            }
        }
    }
    lines.join("\n")
}
