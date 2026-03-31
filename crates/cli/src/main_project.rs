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
