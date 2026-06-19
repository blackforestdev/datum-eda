use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCreateReportView {
    pub(crate) project_root: String,
    pub(crate) project_name: String,
    pub(crate) project_uuid: String,
    pub(crate) schematic_uuid: String,
    pub(crate) board_uuid: String,
    pub(crate) files_written: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectValidationIssueView {
    pub(crate) severity: String,
    pub(crate) code: String,
    pub(crate) subject: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectValidationView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) valid: bool,
    pub(crate) schema_compatible: bool,
    pub(crate) required_files_expected: usize,
    pub(crate) required_files_validated: usize,
    pub(crate) checked_sheet_files: usize,
    pub(crate) checked_definition_files: usize,
    pub(crate) issue_count: usize,
    pub(crate) issues: Vec<NativeProjectValidationIssueView>,
}

pub(crate) fn render_native_project_validation_text(
    report: &NativeProjectValidationView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("valid: {}", report.valid),
        format!("schema_compatible: {}", report.schema_compatible),
        format!(
            "required_files_validated: {}/{}",
            report.required_files_validated, report.required_files_expected
        ),
        format!("checked_sheet_files: {}", report.checked_sheet_files),
        format!(
            "checked_definition_files: {}",
            report.checked_definition_files
        ),
        format!("issue_count: {}", report.issue_count),
    ];
    if !report.issues.is_empty() {
        lines.push("issues:".to_string());
        for issue in &report.issues {
            lines.push(format!(
                "  [{}] {} {} :: {}",
                issue.severity, issue.code, issue.subject, issue.message
            ));
        }
    }
    lines.join("\n")
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardTextMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) text_uuid: String,
    pub(crate) text: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) rotation_deg: i32,
    pub(crate) height_nm: i64,
    pub(crate) stroke_width_nm: i64,
    pub(crate) render_intent: String,
    pub(crate) family: String,
    pub(crate) style: String,
    pub(crate) style_class: Option<String>,
    pub(crate) h_align: String,
    pub(crate) v_align: String,
    pub(crate) mirrored: bool,
    pub(crate) keep_upright: bool,
    pub(crate) line_spacing_ratio_ppm: i32,
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) layer: i32,
}

pub(crate) fn render_native_project_board_text_mutation_text(
    report: &NativeProjectBoardTextMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("text_uuid: {}", report.text_uuid),
        format!("text: {}", report.text),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("rotation_deg: {}", report.rotation_deg),
        format!("height_nm: {}", report.height_nm),
        format!("stroke_width_nm: {}", report.stroke_width_nm),
        format!("render_intent: {}", report.render_intent),
        format!("family: {}", report.family),
        format!("style: {}", report.style),
        format!(
            "style_class: {}",
            report.style_class.as_deref().unwrap_or("")
        ),
        format!("h_align: {}", report.h_align),
        format!("v_align: {}", report.v_align),
        format!("mirrored: {}", report.mirrored),
        format!("keep_upright: {}", report.keep_upright),
        format!("line_spacing_ratio_ppm: {}", report.line_spacing_ratio_ppm),
        format!("bold: {}", report.bold),
        format!("italic: {}", report.italic),
        format!("layer: {}", report.layer),
    ]
    .join("\n")
}
