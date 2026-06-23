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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectNameMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) project_uuid: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRulesMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rule_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rules_object_revision: Option<u64>,
    pub(crate) rule_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSheetMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) schematic_uuid: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) name: String,
    pub(crate) cascaded_objects: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSheetDefinitionMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) schematic_uuid: String,
    pub(crate) definition_uuid: String,
    pub(crate) definition_path: String,
    pub(crate) root_sheet_uuid: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSheetInstanceMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) schematic_uuid: String,
    pub(crate) instance_uuid: String,
    pub(crate) definition_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_sheet_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) port_uuid: Option<String>,
    pub(crate) name: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
}

pub(crate) fn render_native_project_name_mutation_text(
    report: &NativeProjectNameMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("project_uuid: {}", report.project_uuid),
        format!("name: {}", report.name),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_rules_mutation_text(
    report: &NativeProjectRulesMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("rule_count: {}", report.rule_count),
    ];
    if let Some(rule_uuid) = &report.rule_uuid {
        lines.push(format!("rule_uuid: {rule_uuid}"));
    }
    if let Some(revision) = report.rules_object_revision {
        lines.push(format!("rules_object_revision: {revision}"));
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_sheet_mutation_text(
    report: &NativeProjectSheetMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("schematic_uuid: {}", report.schematic_uuid),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("name: {}", report.name),
        format!("cascaded_objects: {}", report.cascaded_objects),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_sheet_definition_mutation_text(
    report: &NativeProjectSheetDefinitionMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("schematic_uuid: {}", report.schematic_uuid),
        format!("definition_uuid: {}", report.definition_uuid),
        format!("definition_path: {}", report.definition_path),
        format!("root_sheet_uuid: {}", report.root_sheet_uuid),
        format!("name: {}", report.name),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_sheet_instance_mutation_text(
    report: &NativeProjectSheetInstanceMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("schematic_uuid: {}", report.schematic_uuid),
        format!("instance_uuid: {}", report.instance_uuid),
        format!("definition_uuid: {}", report.definition_uuid),
    ];
    if let Some(parent_sheet_uuid) = &report.parent_sheet_uuid {
        lines.push(format!("parent_sheet_uuid: {parent_sheet_uuid}"));
    }
    if let Some(port_uuid) = &report.port_uuid {
        lines.push(format!("port_uuid: {port_uuid}"));
    }
    lines.extend([
        format!("name: {}", report.name),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ]);
    lines.join("\n")
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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardNameMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) board_uuid: String,
    pub(crate) name: String,
}

pub(crate) fn render_native_project_board_name_mutation_text(
    report: &NativeProjectBoardNameMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("board_uuid: {}", report.board_uuid),
        format!("name: {}", report.name),
    ]
    .join("\n")
}
