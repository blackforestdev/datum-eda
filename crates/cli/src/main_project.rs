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
